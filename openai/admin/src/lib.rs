use crate::kinode::process::admin::AdminRequest;
use crate::kinode::process::driver::{AdminRequest as DriverAdminRequest};
use kinode_process_lib::{
    await_next_message_body, call_init, our_capabilities, println, spawn, Address, OnExit, Request,
};

const PUBLISHER: &str = "nick1udwig.os";
//const PROCESS_NAME: &str = "driver";
const SCRIPT_NAME: &str = "admin";
const ROUTER_PATH: &str = "llm_provider:nick1udwig.os/pkg/router.wasm";

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
    DriverAdminRequest(DriverAdminRequest),
}

call_init!(init);
fn init(our: Address) {
    let Ok(ref body) = await_next_message_body() else {
        println!("failed to get args!");
        return;
    };
    let package_name = our.package();

    match body.clone().try_into() {
        Ok(Req::AdminRequest(AdminRequest::StartRouter)) => {
            let our_caps = our_capabilities();
            spawn(
                Some("router"),
                ROUTER_PATH,
                OnExit::Restart,
                our_caps,
                vec![],
                false,
            ).unwrap();
        }
        Ok(Req::DriverAdminRequest(DriverAdminRequest::SetLocalDriver(_))) => {
            let driver_process: Address = format!("our@driver:{}", our.package_id())
                .parse()
                .unwrap();
            Request::to(driver_process)
                .body(body.clone())
                .send()
                .unwrap();
        }
        Err(_) => {
            println!("usage:\n{SCRIPT_NAME}:{package_name}:{PUBLISHER} admin_action\ne.g.\n{SCRIPT_NAME}:{package_name}:{PUBLISHER} \"StartRouter\"");
            return;
        }
    }
}
