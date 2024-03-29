use kinode_process_lib::{
    our_capabilities, spawn, Address, OnExit, ProcessId, Request,
};
use std::str::FromStr;
use serde::{Serialize, Deserialize};

use llm_interface::openai::{ChatParams, ChatRequest, LLMRequest, LLMResponse, Message, Provider};

pub fn spawn_openai_pkg(our: Address, openai_key: &str) -> anyhow::Result<OpenaiApi> {
    let openai_pkg_path = format!("{}/pkg/openai.wasm", our.package_id());
    let our_caps = our_capabilities();
    let http_client = ProcessId::from_str("http_client:distro:sys").unwrap();

    let process_id = spawn(
        None,
        &openai_pkg_path,
        OnExit::None,
        our_caps,
        vec![http_client],
        false,
    )?;

    let worker_address = Address {
        node: our.node.clone(),
        process: process_id.clone(),
    };

    Ok(OpenaiApi::new(openai_key.to_string(), worker_address))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenaiApi {
    openai_key: String,
    openai_worker: Address,
}

impl OpenaiApi {
    pub fn new(openai_key: String, openai_worker: Address) -> Self {
        Self {
            openai_key,
            openai_worker,
        }
    }

    pub fn chat(&self, chat_params: ChatParams) -> anyhow::Result<Message> {
        let chat_request = ChatRequest {
            params: chat_params,
            api_key: self.openai_key.clone(),
            provider: Provider::OpenAi,
        };
        let request = LLMRequest::Chat(chat_request);
        let msg = Request::new()
            .target(self.openai_worker.clone())
            .body(request.to_bytes())
            .send_and_await_response(10)??;

        let response = LLMResponse::parse(msg.body())?;
        if let LLMResponse::Chat(chat) = response {
            Ok(chat.to_message_response())
        } else {
            return Err(anyhow::Error::msg("Error querying OpenAI: wrong result"));
        }
    }
}
