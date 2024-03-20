use anyhow::Context;
use common_types::groq::{
    ChatRequest, ChatResponse, EmbeddingRequest, EmbeddingResponse, LLMRequest, LLMResponse, groqEmbeddingResponse
};
use kinode_process_lib::{
    await_message, call_init, get_blob,
    http::{HttpClientAction, OutgoingHttpRequest},
    println, Address, LazyLoadBlob, ProcessId, Request, Response,
};
use serde_json::json;
use std::{collections::HashMap, vec};

const CHAT_CONTEXT_NON_STREAMING: u8 = 0;
const EMBEDDING_CONTEXT: u8 = 2;

pub const URL: &str = "https://api.groq.com/v1";

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
        EMBEDDING_CONTEXT => handle_embedding_response()?,
        _ => {}
    }

    Ok(())
}

fn handle_embedding_response() -> anyhow::Result<()> {
    let bytes = get_blob().context("Couldn't get blob")?;
    let groq_embedding =
        serde_json::from_slice::<groqEmbeddingResponse>(bytes.bytes.as_slice())?;
    let embedding = EmbeddingResponse::from_groq_response(groq_embedding);
    let response = LLMResponse::Embedding(embedding);
    let _ = Response::new()
        .body(serde_json::to_vec(&response).expect("Failed to serialize response for embedding"))
        .send();
    Ok(())
}

fn handle_chat_response_non_streaming() -> anyhow::Result<()> {
    let bytes = get_blob().context("Couldn't get blob")?;
    let chat_response = serde_json::from_slice::<ChatResponse>(bytes.bytes.as_slice())?;
    let llm_response = LLMResponse::Chat(chat_response);
    let _ = Response::new()
        .body(serde_json::to_vec(&llm_response)?)
        .send()?;
    Ok(())
}

fn handle_request(body: &[u8]) -> anyhow::Result<()> {
    // let body_utf8 = String::from_utf8(body.to_vec()).expect("Failed to convert body to UTF-8 string");
    // println!("body utf is {}", body_utf8);
    let request = LLMRequest::parse(body)?;
    match &request {
        LLMRequest::Embedding(embedding_request) => handle_embedding_request(embedding_request)?,
        LLMRequest::Chat(chat_request) => handle_chat_request(chat_request)?,
    }
    Ok(())
}

fn send_request<T: serde::Serialize>(
    params: &T,
    endpoint: &str,
    context: u8,
    api_key: &str,
) -> anyhow::Result<()> {
    let outgoing_request = OutgoingHttpRequest {
        method: "POST".to_string(),
        version: None,
        url: format!("{}/{}", URL, endpoint),
        headers: HashMap::from_iter(vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Authorization".to_string(), format!("Bearer {}", api_key)),
        ]),
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

fn handle_chat_request_non_streaming(chat_request: &ChatRequest) -> anyhow::Result<()> {
    send_request(
        &chat_request.params,
        "chat/completions",
        CHAT_CONTEXT_NON_STREAMING,
        &chat_request.api_key,
    )
}

fn handle_embedding_request(embedding_request: &EmbeddingRequest) -> anyhow::Result<()> {
    send_request(
        &embedding_request.params,
        "embeddings",
        EMBEDDING_CONTEXT,
        &embedding_request.api_key,
    )
}

fn handle_chat_request(chat_request: &ChatRequest) -> anyhow::Result<()> {
    handle_chat_request_non_streaming(chat_request)
}

fn handle_message() -> anyhow::Result<()> {
    let message = await_message()?;
    if message.is_request() {
        let _ = handle_request(message.body());
    } else {
        let _ = handle_response(
            message
                .context()
                .context("groq_api: Failed to get context")?,
        );
    }

    Ok(())
}

call_init!(init);

fn init(_our: Address) {
    println!("groq_api: begin");

    loop {
        match handle_message() {
            Ok(()) => {}
            Err(e) => {
                println!("groq_api: error: {:?}", e);
            }
        };
    }
}
