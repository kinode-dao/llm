use std::collections::{HashMap, VecDeque};

use rand::{Rng, SeedableRng, prelude::SliceRandom};
use rand_pcg::Pcg64;

use crate::kinode::process::router::{
    ClientRequest, ClientResponse,
    DriverRequest, DriverResponse,
    RouterRequest, RouterResponse,
    ToClientRequest, ToClientResponse,
    RunJobRequestParams,
    JobUpdateRequestParams,
    //JobUpdateRequestBlob,
};
use kinode_process_lib::{await_message, call_init, get_typed_state, println, set_state, Address, ProcessId, Request, Response};


const DEFAULT_DRIVER_PROCESS_ID: &str = "driver:llm_provider:nick1udwig.os";
const DEFAULT_QUEUE_RESPONSE_TIMEOUT_SECONDS: u8 = 5;
const DEFAULT_SERVE_TIMEOUT_SECONDS: u16 = 60;

wit_bindgen::generate!({
    path: "target/wit",
    world: "llm-provider-nick1udwig-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, process_macros::SerdeJsonInto)]
#[serde(untagged)]
enum Req {
    ClientRequest(ClientRequest),
    DriverRequest(DriverRequest),
    ToClientRequest(ToClientRequest),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, process_macros::SerdeJsonInto)]
#[serde(untagged)]
enum Res {
    RouterResponse(RouterResponse),
    ToClientResponse(ToClientResponse),
}

#[derive(Debug, serde::Serialize, serde::Deserialize, process_macros::SerdeJsonInto)]
struct State {
    driver_process_id: Option<ProcessId>,
    available_drivers: HashMap<String, String>,  // driver node to model
    outstanding_jobs: HashMap<String, (Address, u64)>,
    job_queue: VecDeque<(Address, u64, RunJobRequestParams)>,
    job_queries: HashMap<u64, JobQuery>,
    rng: Pcg64,
    pub queue_response_timeout_seconds: u8,
    pub serve_timeout_seconds: u16, // TODO
}

#[derive(Debug, serde::Serialize, serde::Deserialize, process_macros::SerdeJsonInto)]
struct JobQuery {
    job: RunJobRequestParams,
    num_rejections: u32,
    num_queried: u32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            driver_process_id: None,
            available_drivers: HashMap::new(),
            outstanding_jobs: HashMap::new(),
            job_queue: VecDeque::new(),
            job_queries: HashMap::new(),
            rng: Pcg64::from_entropy(),
            queue_response_timeout_seconds: DEFAULT_QUEUE_RESPONSE_TIMEOUT_SECONDS,
            serve_timeout_seconds: DEFAULT_SERVE_TIMEOUT_SECONDS,
        }
    }
}

impl State {
    fn save(&self) -> anyhow::Result<()> {
        set_state(&serde_json::to_vec(self)?);
        Ok(())
    }

    fn load() -> Self {
        get_typed_state(|bytes| Ok(bytes.try_into()?))
            .unwrap_or_default()
    }
}

fn permute<T>(mut vec: Vec<T>, rng: &mut Pcg64) -> Vec<T> {
    vec.shuffle(rng);
    vec
}

fn serve_job(
    driver: &Address,
    job_source: &Address,
    job_id: u64,
    job: RunJobRequestParams,
    state: &mut State,
) -> anyhow::Result<()> {
     state.outstanding_jobs.insert(
         driver.node().to_string(),
         (job_source.clone(), job_id.clone()),
     );
     Request::to(driver)
         .body(RouterRequest::RunJob((job_id, job)))
         .inherit(true)
         .expects_response(5)  // TODO
         .send()?;
    state.save()?;
    Ok(())
}

//fn handle_admin_request(
//    our: &Address,
//    message: &Message,
//    state: &mut State,
//) -> anyhow::Result<()> {
//    let source = message.source();
//    if source.node() != our.node() {
//        return Err(anyhow::anyhow!("only our can make AdminRequests; rejecting from {source:?}"));
//    }
//    match serde_json::from_slice(message.body())? {
//        AdminRequest::SetProviderProcess { process_id } => {
//            let process_id = process_id.parse()?;
//            state.provider_process = Some(process_id);
//            state.save()?;
//            Response::new()
//                .body(serde_json::to_vec(&AdminResponse::SetProviderProcess { err: None })?)
//                .send()?;
//        }
//        AdminRequest::SetRollupSequencer { address } => {
//            let address = address.parse()?;
//            state.rollup_sequencer = Some(address);
//            await_chain_state(state)?;
//            Response::new()
//                .body(serde_json::to_vec(&AdminResponse::SetRollupSequencer { err: None })?)
//                .send()?;
//        }
//        AdminRequest::SetContractAddress { address } => {
//            state.contract_address = address;
//            Response::new()
//                .body(serde_json::to_vec(&AdminResponse::SetContractAddress { err: None })?)
//                .send()?;
//        }
//        AdminRequest::CreateDao => {
//            // TODO:
//            // this belong on the FE, along with all other DAO-changing requests
//            // so we can take advantage of already-existing wallet software
//            //init_eth(our, eth_provider, filter, state).unwrap();
//            //Response::new()
//            //    .body(serde_json::to_vec(&AdminResponse::CreateDao { err: None })?)
//            //    .send()?;
//        }
//        AdminRequest::SetDaoId { dao_id } => {
//            state.dao_id = dao_id;
//            init_eth(our, eth_provider, filter, state).unwrap();
//            Response::new()
//                .body(serde_json::to_vec(&AdminResponse::SetDaoId { err: None })?)
//                .send()?;
//        }
//    }
//    Ok(())
//}

fn handle_client_request(
    _our: &Address,
    source: &Address,
    client_request: &ClientRequest,
    state: &mut State,
) -> anyhow::Result<()> {
    match client_request {
        ClientRequest::RunJob(job) => {
            let job_id: u64 = state.rng.gen();
            Response::new()
                .body(ClientResponse::RunJob(Ok(job_id.clone())))
                .send()?;
            let num_hosting_model: Vec<(String, String)> = state.available_drivers
                .iter()
                .filter_map(|(node, model)| {
                    if &job.model != model {
                        None
                    } else {
                        Some((node.clone(), model.clone()))
                    }
                })
                .collect();
            if num_hosting_model.is_empty() {
                // no available drivers -> add to queue
                state.job_queue.push_back((source.clone(), job_id, job.clone()));
                println!("new job added to queue; now have {} queued", state.job_queue.len());
                state.save()?;
                return Ok(());
            }
            // permute available drivers & flood all with query if ready;
            //  first gets job; if none ready, queue
            // TODO: improve algo
            let process_id: ProcessId = state.driver_process_id
                .clone()
                .unwrap_or_else(|| DEFAULT_DRIVER_PROCESS_ID.parse().unwrap());
            state.job_queries.insert(job_id, JobQuery {
                job: job.clone(),
                num_rejections: 0,
                num_queried: num_hosting_model.len() as u32,
            });
            for (member, model) in permute(num_hosting_model, &mut state.rng) {
                if job.model != model {
                    continue;
                }
                let address = Address::new(member.clone(), process_id.clone());
                Request::to(address.clone())
                    .body(RouterRequest::QueryReady)
                    .context(serde_json::to_vec(&(source.clone(), job_id))?)
                    .expects_response(state.queue_response_timeout_seconds as u64)
                    .send()?;
            }
        }
    }
    Ok(())
}

fn handle_driver_request(
    _our: &Address,
    source: &Address,
    driver_request: &DriverRequest,
    state: &mut State,
) -> anyhow::Result<()> {
    match driver_request {
        DriverRequest::SetIsAvailable((is_available, model_name)) => {
            if !is_available {
                state.available_drivers.remove(source.node());
                state.save()?;
            } else {
                if !state.job_queue.is_empty() {
                    let (job_source, job_id, job) = state.job_queue.pop_front().unwrap();
                    serve_job(source, &job_source, job_id, job, state)?;
                } else {
                    state.available_drivers.insert(source.node().to_string(), model_name.clone());
                    state.save()?;
                }
            }
            Response::new()
                .body(DriverResponse::SetIsAvailable)
                .send()?;
        }
    }
    Ok(())
}

fn handle_to_client_request(
    source: &Address,
    to_client_request: &ToClientRequest,
    state: &mut State,
) -> anyhow::Result<()> {
    match to_client_request {
        ToClientRequest::JobUpdate(JobUpdateRequestParams { ref job_id, ref is_final, .. }) => {
            let Some((job_source, expected_job_id)) = state.outstanding_jobs.get(source.node()) else {
                return Err(anyhow::anyhow!("provider sent back {job_id} but no record here"));
            };
            if job_id != expected_job_id {
                println!("job_id != expected_job_id: this should never occur! provider gave us wrong job back");
            }
            Request::to(job_source)
                .body(to_client_request)
                .inherit(true)
                .send()?;
            // TODO: log sigs
            if is_final == &true {
                state.outstanding_jobs.remove(source.node());
                state.save()?;
            }
            Response::new()
                .body(ToClientResponse::JobUpdate)
                .send()?;
        }
    }
    Ok(())
}

fn handle_router_response(
    _our: &Address,
    source: &Address,
    context: &[u8],
    router_response: &RouterResponse,
    state: &mut State,
) -> anyhow::Result<()> {
    match router_response {
        RouterResponse::RunJob(_result) => {
            // TODO: pass on errors to client?
        }
        RouterResponse::QueryReady(is_ready) => {
            // compare to handle_message() send_err case
            let (job_source, job_id): (Address, u64) = serde_json::from_slice(context)?;
            //    message.context().unwrap_or_default()
            //)?;
            let Some(mut job_query) = state.job_queries.remove(&job_id) else {
                // TODO: readd JobTaken again?
                //Request::to(message.source())
                //    .body(serde_json::to_vec(&MemberRequest::JobTaken { job_id })?)
                //    .send()?;
                //state.save()?;
                println!("{source} didnt find job_query for {job_id}");
                return Ok(());
            };
            if !is_ready {
                // TODO: reprimand fake ready member?
                println!("{source} not ready");
                job_query.num_rejections += 1;
                if job_query.num_rejections >= job_query.num_queried {
                    // no one available to serve job
                    // TODO: add stat trackers so we can expose endpoints:
                    //  * how long queue is
                    //  * average time / job
                    //    -> expected time till result
                    state.job_queue.push_back((job_source, job_id.clone(), job_query.job));
                    println!("no ready providers; now have {} queued", state.job_queue.len());
                    state.save()?;
                    return Ok(());
                }
                state.job_queries.insert(job_id, job_query);
                state.save()?;
                return Ok(());
            }
            println!("{source} accepts job");
            serve_job(source, &job_source, job_id, job_query.job, state)?;
        }
    }
    Ok(())
}

fn handle_to_client_response(
    to_client_response: &ToClientResponse,
) -> anyhow::Result<()> {
    match to_client_response {
        ToClientResponse::JobUpdate => {}
    }
    Ok(())
}

fn handle_message(
    our: &Address,
    state: &mut State,
) -> anyhow::Result<()> {
    let message = match await_message() {
        Ok(m) => m,
        Err(send_err) => {
            //println!("SendError\nkind: {:?}\nbody: {:?}", send_err.kind(), serde_json::from_slice::<serde_json::Value>(send_err.message().body()));
            // compare to handle_member_response() MemberResponse::QueryReady case
            let (source, job_id): (Address, u64) = serde_json::from_slice(
                send_err.context().unwrap_or_default()
            )?;
            let Some(mut job_query) = state.job_queries.remove(&job_id) else {
                // provider is offline, so don't inform them
                return Ok(());
            };
            job_query.num_rejections += 1;
            if job_query.num_rejections >= job_query.num_queried {
                // no one available to serve job
                // TODO: add stat trackers so we can expose endpoints:
                //  * how long queue is
                //  * average time / job
                //    -> expected time till result
                state.job_queue.push_back((source, job_id, job_query.job));
                println!("no ready drivers; now have {} queued", state.job_queue.len());
                state.save()?;
                return Ok(());
            }
            state.job_queries.insert(job_id, job_query);
            state.save()?;
            return Ok(());
        }
    };

    if message.is_request() {
        return match message.body().try_into()? {
            Req::ClientRequest(ref client_request) => handle_client_request(
                our,
                message.source(),
                client_request,
                state,
            ),
            Req::DriverRequest(ref driver_request) => handle_driver_request(
                our,
                message.source(),
                driver_request,
                state,
            ),
            Req::ToClientRequest(ref to_client_request) => handle_to_client_request(
                message.source(),
                to_client_request,
                state,
            ),
        };
    }

    match message.body().try_into()? {
        Res::RouterResponse(ref router_response) => handle_router_response(
            our,
            message.source(),
            message.context().unwrap_or_default(),
            router_response,
            state,
        ),
        Res::ToClientResponse(ref to_client_response) => handle_to_client_response(
            to_client_response,
        ),
    }
}

call_init!(init);
fn init(our: Address) {
    println!("{}: begin", our.process());

    let mut state = State::load();

    loop {
        match handle_message(&our, &mut state) {
            Ok(()) => {},
            Err(e) => println!("{}: error: {:?}", our.process(), e),
        };
    }
}
