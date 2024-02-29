use anyhow::Context;
use common_types::lccp::{
    ChatRequest, ChatResponse, ChatStreamingResponse, EmbeddingRequest, EmbeddingResponse,
    LLMRequest, LLMResponse,
};
use kinode_process_lib::{
    await_message, call_init, get_blob,
    http::{HttpClientAction, OutgoingHttpRequest},
    println, Address, LazyLoadBlob, ProcessId, Request, Response,
};
use serde_json::json;
use std::{collections::HashMap, vec};

// TODO: Zen: This should be in a config
const URL: &str = "http://127.0.0.1:3000";

const CHAT_CONTEXT_NON_STREAMING: u8 = 0;
const CHAT_CONTEXT_STREAMING: u8 = 1;
const EMBEDDING_CONTEXT: u8 = 2;

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

fn handle_response(context: &[u8]) -> anyhow::Result<()> {
    match context[0] {
        CHAT_CONTEXT_NON_STREAMING => handle_chat_response_non_streaming()?,
        // CHAT_CONTEXT_STREAMING => handle_chat_response_streaming()?,
        EMBEDDING_CONTEXT => handle_embedding_response()?,
        _ => {}
    }

    Ok(())
}

fn handle_embedding_response() -> anyhow::Result<()> {
    let bytes = get_blob().context("Couldn't get blob")?;
    let output = serde_json::from_slice::<EmbeddingResponse>(bytes.bytes.as_slice())?;
    let response = LLMResponse::Embedding(output);
    let _ = Response::new()
        .body(serde_json::to_vec(&response).expect("Failed to serialize response for embedding"))
        .send();
    Ok(())
}

fn handle_chat_response_non_streaming() -> anyhow::Result<()> {
    let bytes = get_blob().context("Couldn't get blob")?;
    let response = serde_json::from_slice::<ChatResponse>(bytes.bytes.as_slice())?;
    let response = LLMResponse::Chat(response);
    let _ = Response::new()
        .body(serde_json::to_vec(&response)?)
        .send()?;
    Ok(())
}

fn _handle_chat_response_streaming() -> anyhow::Result<()> {
    let mut acc = String::new();
    if let Some(bytes) = get_blob() {
        println!("lccp: length of received bytes: {}", bytes.bytes.len());
        let response_str = String::from_utf8(bytes.bytes.clone()).expect("Invalid UTF-8");
        acc.push_str(&response_str);

        let acc_clone = acc.clone();
        for chunk in acc_clone.split("\n\n").filter(|e| e.starts_with("data: ")) {
            let json_str = &chunk[6..];
            if let Ok(response) = serde_json::from_str::<ChatStreamingResponse>(json_str) {
                let llm_response = LLMResponse::ChatStreaming(response);
                // println!("lccp: sending response: {:?}", llm_response);
                Response::new()
                    .body(serde_json::to_vec(&llm_response)?)
                    .send()?;
                // Remove the string from acc
                acc = acc.replace(chunk, "");
            }
        }
        // if stop {
        //     break;
        // }
    }
    Ok(())
}

fn handle_request(body: &[u8]) -> anyhow::Result<()> {
    let request = LLMRequest::parse(body)?;
    match &request {
        LLMRequest::Embedding(embedding_params) => handle_embedding_request(embedding_params)?,
        LLMRequest::Chat(chat_params) => handle_chat_request(chat_params)?,
    }
    Ok(())
}
fn send_request<T: serde::Serialize>(
    params: &T,
    endpoint: &str,
    context: u8,
) -> anyhow::Result<()> {
    let outgoing_request = OutgoingHttpRequest {
        method: "POST".to_string(),
        version: None,
        url: format!("{}/{}", URL, endpoint),
        headers: HashMap::from_iter(vec![(
            "Content-Type".to_string(),
            "application/json".to_string(),
        )]),
    };
    let body_bytes = json!(HttpClientAction::Http(outgoing_request))
        .to_string()
        .as_bytes()
        .to_vec();
    let content = serde_json::to_string(params).expect("Failed to serialize params");
    Request::new()
        .target(Address::new(
            "our",
            ProcessId::new(Some("http_client"), "distro", "sys"),
        ))
        .body(body_bytes)
        .expects_response(30)
        .context(vec![context])
        .blob(LazyLoadBlob {
            mime: Some("application/json".to_string()),
            bytes: content.as_bytes().to_vec(),
        })
        .send()?;
    Ok(())
}


fn handle_chat_request_non_streaming(chat_params: &ChatRequest) -> anyhow::Result<()> {
    send_request(chat_params, "completion", CHAT_CONTEXT_NON_STREAMING)
}

fn handle_chat_request_streaming(chat_params: &ChatRequest) -> anyhow::Result<()> {
    send_request(chat_params, "completion", CHAT_CONTEXT_STREAMING)
}

fn handle_embedding_request(embedding_params: &EmbeddingRequest) -> anyhow::Result<()> {
    send_request(embedding_params, "embedding", EMBEDDING_CONTEXT)
}

fn handle_chat_request(chat_params: &ChatRequest) -> anyhow::Result<()> {
    if chat_params.stream.unwrap_or(false) {
        handle_chat_request_streaming(chat_params)
    } else {
        handle_chat_request_non_streaming(chat_params)
    }
}

fn handle_message() -> anyhow::Result<()> {
    let message = await_message()?;
    if message.is_request() {
        let _ = handle_request(message.body());
    } else {
        let context = message.context().context("lccp: Failed to get context")?;
        let _ = handle_response(context);
    }

    Ok(())
}

call_init!(init);

fn init(_our: Address) {
    println!("lccp: begin");

    loop {
        match handle_message() {
            Ok(()) => {}
            Err(e) => {
                println!("lccp: error: {:?}", e);
            }
        };
    }
}

// TODO: Zen: Check out how we send these chatparams with message while just defaulting the fields we don't need
