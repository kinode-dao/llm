use anyhow::Context;
use kinode_process_lib::{
    await_message, call_init, get_blob,
    http::{HttpClientAction, OutgoingHttpRequest},
    println, Address, LazyLoadBlob, ProcessId, Request, Response,
};
use llm_interface::openai::{
    ChatImageRequest, ChatRequest, ChatResponse, EmbeddingRequest, EmbeddingResponse, LLMRequest,
    LLMResponse, OpenAiEmbeddingResponse, Provider,
};
use serde_json::json;
use std::{collections::HashMap, vec};


pub const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
pub const GROQ_BASE_URL: &str = "https://api.groq.com/openai/v1";

// TODO: Put this in helper functions
// TODO: Zena: We should probably derive this through a trait at some point? 
pub fn request_to_context(request: &LLMRequest) -> usize {
    match request {
        LLMRequest::RegisterApiKey(_) => 0,
        LLMRequest::Embedding(_) => 1,
        LLMRequest::OpenaiChat(_) => 2,
        LLMRequest::GroqChat(_) => 3,
        LLMRequest::ChatImage(_) => 4,
    }
}

// ----------------------------------------------------------

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

fn handle_response(context: &[u8]) -> anyhow::Result<()> {
    let a = TestType {
        process_name: "hi".to_string(),
        package_name: "hi".to_string(),
    };

    match context[0] {
        CHAT_CONTEXT_NON_STREAMING => handle_chat_response_non_streaming()?,
        EMBEDDING_CONTEXT => handle_embedding_response()?,
        _ => {}
    }

    Ok(())
}

fn handle_embedding_response() -> anyhow::Result<()> {
    let bytes = get_blob().context("Couldn't get blob")?;
    let openai_embedding =
        serde_json::from_slice::<OpenAiEmbeddingResponse>(bytes.bytes.as_slice())?;
    let embedding = EmbeddingResponse::from_openai_response(openai_embedding);
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
    let request = serde_json::from_slice::<LLMRequest>(body)?;
    match &request {
        LLMRequest::RegisterApiKey(api_key) => todo!(),
        LLMRequest::Embedding(_) => todo!(),
        LLMRequest::OpenaiChat(_) => todo!(),
        LLMRequest::GroqChat(_) => todo!(),
        LLMRequest::ChatImage(_) => todo!(),
        // LLMRequest::Embedding(embedding_request) => handle_embedding_request(embedding_request)?,
        // LLMRequest::Chat(chat_request) => handle_chat_request(chat_request)?,
        // LLMRequest::ChatImage(chat_image_request) => handle_chat_image_request(chat_image_request)?,
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
        url: endpoint.to_string(),
        headers: HashMap::from_iter(vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Authorization".to_string(), format!("Bearer {}", api_key)),
        ]),
    };
    let body_bytes = json!(HttpClientAction::Http(outgoing_request))
        .to_string()
        .as_bytes()
        .to_vec();
    let pretty_content =
        serde_json::to_string_pretty(&params).expect("Failed to pretty print JSON");
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
    let url = match chat_request.provider {
        Provider::OpenAi => OPENAI_BASE_URL,
        Provider::Groq => GROQ_BASE_URL,
    };
    send_request(
        &chat_request.params,
        &format!("{}/chat/completions", url),
        CHAT_CONTEXT_NON_STREAMING,
        &chat_request.api_key,
    )
}

fn handle_chat_image_request_non_streaming(
    chat_image_request: &ChatImageRequest,
) -> anyhow::Result<()> {
    let url = match chat_image_request.provider {
        Provider::OpenAi => OPENAI_BASE_URL,
        Provider::Groq => GROQ_BASE_URL,
    };
    send_request(
        &chat_image_request.params,
        &format!("{}/chat/completions", url),
        CHAT_CONTEXT_NON_STREAMING,
        &chat_image_request.api_key,
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

fn handle_chat_image_request(chat_image_request: &ChatImageRequest) -> anyhow::Result<()> {
    handle_chat_image_request_non_streaming(chat_image_request)
}

fn handle_message() -> anyhow::Result<()> {
    let message = await_message()?;
    if message.is_request() {
        let _ = handle_request(message.body());
    } else {
        let _ = handle_response(
            message
                .context()
                .context("openai_api: Failed to get context")?,
        );
    }

    Ok(())
}

call_init!(init);
fn init(_our: Address) {
    loop {
        match handle_message() {
            Ok(()) => {}
            Err(e) => {
                println!("openai_api: error: {:?}", e);
            }
        };
    }
}
