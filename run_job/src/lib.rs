use crate::kinode::process::driver::{ClientRequest, ClientResponse, RunJobRequestParams, ToClientRequest};
use kinode_process_lib::{
    await_next_message_body, call_init, get_blob, println, Address, Request,
};

wit_bindgen::generate!({
    path: "target/wit",
    world: "llm-provider-nick1udwig-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});

const PUBLISHER: &str = "nick1udwig.os";
const SCRIPT_NAME: &str = "run_job";

call_init!(init);
fn init(our: Address) {
    let Ok(body) = await_next_message_body() else {
        println!("failed to get args!");
        return;
    };
    let package_name = our.package();
    let body = String::from_utf8(body).unwrap_or_default();
    let Some((model, prompt)) = body.split_once(' ') else {
        println!("usage:\n{SCRIPT_NAME}:{package_name}:{PUBLISHER} model prompt\ne.g.\n{SCRIPT_NAME}:{package_name}:{PUBLISHER} default How much wood could a woodchuck chuck? Be concise.");
        return;
    };

    let driver_process: Address = format!("our@driver:{}", our.package_id()).parse().unwrap();
    let job = RunJobRequestParams {
        model: model.into(),
        prompt: prompt.into(),
        seed: None,
    };
    let result = Request::to(driver_process)
        .body(ClientRequest::RunJob(job))
        .send_and_await_response(5);
    let Ok(Ok(result)) = result else {
        println!("got error getting Response: {result:?}");
        return;
    };
    let Ok(ClientResponse::RunJob(Ok(_))) = result.body().try_into() else {
        println!("got error parsing Response: {result:?}");
        return;
    };

    let Ok(body) = await_next_message_body() else {
        println!("failed to get result");
        return;
    };
    let Ok(ToClientRequest::JobUpdate(_job_update)) = body.try_into() else {
        println!("unexpected message: not JobUpdate");
        return;
    };

    let job_result = get_blob().unwrap_or_default().bytes;
    let job_result = String::from_utf8(job_result).unwrap_or_default();
    println!("{job_result}");

    return;
}
