use crate::kinode::process::driver::{
    AdminRequest, AdminResponse,
    ClientRequest,
    DriverRequest, DriverResponse,
    LocalDriver,
    RouterRequest, RouterResponse,
    ToClientRequest, ToClientResponse,
    RunJobRequestParams,
    JobUpdateRequestParams,
    //JobUpdateRequestBlob,
};
use kinode_process_lib::{
    await_message, call_init, get_typed_state, println, set_state,
    Address, ProcessId, Request, Response,
};
use llm_interface::openai::{ChatRequestBuilder, ChatResponse, LLMRequest, LLMResponse, Message as LLMMessage};

const DEFAULT_ROUTER_PROCESS_ID: &str = "router:llm_provider:nick1udwig.os";
const DEFAULT_ROUTER_NODE: &str = "nick1udwig.os";  // NOTE: this should be changed
const DEFAULT_TIMEOUT_SECONDS: u64 = 60;

wit_bindgen::generate!({
    path: "target/wit",
    world: "llm-provider-nick1udwig-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, process_macros::SerdeJsonInto)]
#[serde(untagged)]
enum Req {
    AdminRequest(AdminRequest),
    ClientRequest(ClientRequest),
    RouterRequest(RouterRequest),
    ToClientRequest(ToClientRequest),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, process_macros::SerdeJsonInto)]
#[serde(untagged)]
enum Res {
    DriverResponse(DriverResponse),
    ToClientResponse(ToClientResponse),
}

#[derive(Debug, serde::Serialize, serde::Deserialize, process_macros::SerdeJsonInto)]
struct State {
    router_process_id: Option<ProcessId>,
    router_node: Option<String>,
    local_driver: Option<LocalDriver>,
    outstanding_job: Option<ProcessId>
}

impl Default for State {
    fn default() -> Self {
        Self {
            router_process_id: None,
            router_node: None,
            local_driver: None,
            outstanding_job: None,
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

fn extract_chat_response_message(chat_response: &ChatResponse) -> anyhow::Result<String> {
    if chat_response.choices.len() != 1 {
        return Err(anyhow::anyhow!("unexpected form of ChatResponse: {chat_response:?}"));
    }
    Ok(chat_response.choices[0].message.content.clone())
}

fn send_to_sidecar(
    our: &Address,
    job: &RunJobRequestParams,
    timeout: Option<u64>,
) -> anyhow::Result<String> {
    let llm_process: Address = format!("{}@openai:{}", our.node(), our.package_id()).parse()?;
    let mut chat_request = ChatRequestBuilder::default();
    chat_request
        .model(job.model.clone())
        .messages(vec![LLMMessage {
            role: "user".into(),
            content: job.prompt.clone(),
        }]);
    if let Some(seed) = job.seed {
        chat_request.seed(Some(seed as i32));
    };
    let chat_request = chat_request.build()?;
    let chat_response = Request::to(llm_process)
        .body(serde_json::to_vec(&LLMRequest::OaiProviderChat(chat_request))?)
        .send_and_await_response(timeout.unwrap_or_else(|| DEFAULT_TIMEOUT_SECONDS))??;
    let LLMResponse::Chat(chat_response) = serde_json::from_slice(chat_response.body())? else {
        return Err(anyhow::anyhow!("unexpected chat response: {chat_response:?}"));
    };
    let chat_response_message = extract_chat_response_message(&chat_response)?;
    return Ok(chat_response_message);
}

fn run_local_job(
    our: &Address,
    source: &Address,
    job: &RunJobRequestParams,
    timeout: Option<u64>,
    state: &mut State,
) -> anyhow::Result<()> {
    if source.node() != our.node() {
        return Err(anyhow::anyhow!(
            "rejecting local ClientRequest from {}; must be from our",
            source.node(),
        ));
    }

    if let Some(ref local_driver) = state.local_driver {
        println!("serving locally: {}", local_driver.model);
        Response::new()
            .body(RouterResponse::RunJob(Ok(0)))
            .send()?;
        let chat_response_message = send_to_sidecar(our, job, timeout)?;
        Request::to(source)
            .body(ToClientRequest::JobUpdate(JobUpdateRequestParams {
                job_id: 0,
                is_final: true,
                signature: None,  // TODO
            }))
            .blob_bytes(chat_response_message)
            .expects_response(5) // TODO
            .send()?;
    } else {
        // serve by passing Request to router
        let router: Address = format!(
            "{}@{}",
            state.router_node.clone().unwrap_or_else(|| DEFAULT_ROUTER_NODE.to_string()),
            state.router_process_id.clone().unwrap_or_else(|| DEFAULT_ROUTER_PROCESS_ID.parse().unwrap()),
        ).parse()?;
        println!("serving via router: {}", router);
        Response::new()
            .body(RouterResponse::RunJob(Ok(0)))
            .send()?;
        Request::to(router)
            .body(ClientRequest::RunJob(job.clone()))
            .send()?;
        state.outstanding_job = Some(source.process.clone());
    }

    Ok(())
}

fn set_is_available(is_available: bool, state: &mut State) -> anyhow::Result<()> {
    let Some(ref local_driver) = state.local_driver else {
        return Err(anyhow::anyhow!("LocalDriver must be set before availability can be set"));
    };
    let router: Address = format!(
        "{}@{}",
        state.router_node
            .clone()
            .unwrap_or_else(|| DEFAULT_ROUTER_NODE.to_string()),
        state.router_process_id
            .clone()
            .unwrap_or_else(|| DEFAULT_ROUTER_PROCESS_ID.parse().unwrap()),
    ).parse()?;
    Request::to(router)
        .body(DriverRequest::SetIsAvailable((
            is_available,
            local_driver.model.clone(),
        )))
        .send()?;
    Ok(())
}

fn run_job(
    our: &Address,
    source: &Address,
    job_id: &u64,
    job: &RunJobRequestParams,
    timeout: Option<u64>,
    state: &mut State,
) -> anyhow::Result<()> {
    let router_node = state.router_node
        .clone()
        .unwrap_or_else(|| DEFAULT_ROUTER_NODE.into());

    if source.node() != router_node {
        let err = format!(
            "rejecting RouterRequest from {}; must be from router {}",
            source.node(),
            router_node,
        );
        Response::new()
            .body(RouterResponse::RunJob(Err(err.clone())))
            .send()?;
        return Err(anyhow::anyhow!(err));
    }
    if !state.local_driver.as_ref().is_some_and(|ld| ld.is_public) {
        let err = "got request from router, but not public";
        Response::new()
            .body(RouterResponse::RunJob(Err(err.to_string())))
            .send()?;
        return Err(anyhow::anyhow!(err));
    }
    let Some(ref local_driver) = state.local_driver else {
        // deny Request: cannot serve it
        let err = "got request from router, but don't have a local llm";
        Response::new()
            .body(RouterResponse::RunJob(Err(err.to_string())))
            .send()?;
        return Err(anyhow::anyhow!(err));
    };

    Response::new()
        .body(RouterResponse::RunJob(Ok(0)))
        .send()?;
    let chat_response_message = send_to_sidecar(our, job, timeout)?;
    Request::to(source)
        .body(ToClientRequest::JobUpdate(JobUpdateRequestParams {
            job_id: job_id.clone(),
            is_final: true,
            signature: None,  // TODO
        }))
        .blob_bytes(chat_response_message)
        .expects_response(5) // TODO
        .send()?;

    state.outstanding_job = None;
    if state.local_driver.as_ref().is_some_and(|ld| ld.is_public) {
        println!("setting ready again");
        set_is_available(true, state)?
    }

    Ok(())
}

fn handle_admin_request(
    our: &Address,
    source: &Address,
    admin_request: &AdminRequest,
    state: &mut State,
) -> anyhow::Result<()> {
    if source.node() != our.node() {
        return Err(anyhow::anyhow!(
            "rejecting AdminRequest from {}; must be from our",
            source.node(),
        ));
    }
    match admin_request {
        AdminRequest::SetLocalDriver(ref local_driver) => {
            state.local_driver = Some(local_driver.clone());
            if local_driver.is_public {
                set_is_available(true, state)?
            }
        }
        AdminRequest::SetRouter(ref router) => {
            if let Some(ref process_id) = router.process_id {
                state.router_process_id = Some(ProcessId {
                    process_name: process_id.process_name.clone(),
                    package_name: process_id.package_name.clone(),
                    publisher_node: process_id.publisher_node.clone(),
                });
            };
            if let Some(ref node) = router.node {
                state.router_node = Some(node.clone());
            };
            if state.local_driver.as_ref().is_some_and(|ld| ld.is_public) {
                set_is_available(true, state)?
            }
        }
    }
    Ok(())
}

fn handle_client_request(
    our: &Address,
    source: &Address,
    client_request: &ClientRequest,
    state: &mut State,
) -> anyhow::Result<()> {
    match client_request {
        ClientRequest::RunJob(ref job) => {
            run_local_job(our, source, job, None, state)?;
        }
    }
    Ok(())
}

fn handle_router_request(
    our: &Address,
    source: &Address,
    router_request: &RouterRequest,
    state: &mut State,
) -> anyhow::Result<()> {
    match router_request {
        RouterRequest::RunJob((ref job_id, ref job)) => {
            run_job(our, source, job_id, job, None, state)?;
        }
        RouterRequest::QueryReady => {
            Response::new()
                .body(RouterResponse::QueryReady(
                    state.local_driver.as_ref().is_some_and(|ld| ld.is_public)
                    && state.outstanding_job.as_ref().is_none()
                ))
                .send()?;
        }
    }
    Ok(())
}

fn handle_to_client_request(
    to_client_request: &ToClientRequest,
    state: &mut State,
) -> anyhow::Result<()> {
    match to_client_request {
        ToClientRequest::JobUpdate(job_update) => {
            let Some(ref requestor) = state.outstanding_job else {
                return Err(anyhow::anyhow!("no outstanding job for {job_update:?}; dropping"));
            };
            let requestor: Address = format!("our@{requestor}").parse()?;
            Request::to(requestor)
                .body(to_client_request)
                .inherit(true)
                .send()?;
            state.outstanding_job = None;
            if state.local_driver.as_ref().is_some_and(|ld| ld.is_public) {
                println!("setting ready again");
                set_is_available(true, state)?
            }
        }
    }
    Ok(())
}

fn handle_driver_response(
    driver_response: &DriverResponse,
    state: &mut State,
) -> anyhow::Result<()> {
    match driver_response {
        DriverResponse::SetIsAvailable => {}
    }
    Ok(())
}

fn handle_to_client_response(
    to_client_response: &ToClientResponse,
    state: &mut State,
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
    let message = await_message()?;

    if message.is_request() {
        return match message.body().try_into()? {
            Req::AdminRequest(ref admin_request) => handle_admin_request(
                our,
                message.source(),
                admin_request,
                state,
            ),
            Req::ClientRequest(ref client_request) => handle_client_request(
                our,
                message.source(),
                client_request,
                state,
            ),
            Req::RouterRequest(ref router_request) => handle_router_request(
                our,
                message.source(),
                router_request,
                state,
            ),
            Req::ToClientRequest(ref to_client_request) => handle_to_client_request(
                to_client_request,
                state,
            ),
        };
    }

    match message.body().try_into()? {
        Res::DriverResponse(ref driver_response) => handle_driver_response(
            driver_response,
            state,
        ),
        Res::ToClientResponse(ref to_client_response) => handle_to_client_response(
            to_client_response,
            state,
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
