use anyhow::Context;
use kinode_process_lib::{
    await_message, call_init, get_blob,
    http::{HttpClientAction, OutgoingHttpRequest},
    println, Address, LazyLoadBlob, ProcessId, Request, Response,
};
use llm_interface::openai::{ChatResponse, LLMRequest, LLMResponse, RegisterApiKeyRequest, ClaudeChatResponse};
use serde::Serialize;
use std::{collections::HashMap, vec};

mod structs;
use structs::State;

mod helpers;
use helpers::*;

pub const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
pub const GROQ_BASE_URL: &str = "https://api.groq.com/openai/v1";
pub const CLAUDE_BASE_URL: &str = "https://api.anthropic.com/v1";

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});

enum ApiKeyType {
    OpenAI,
    Groq,
    Claude,
}

fn handle_response(context: &[u8]) -> anyhow::Result<()> {
    match context[0] {
        EMBEDDING_CONTEXT => handle_embedding_response()?,
        OPENAI_CHAT_CONTEXT | GROQ_CHAT_CONTEXT | CHAT_IMAGE_CONTEXT => handle_chat_response()?,
        CLAUDE_CHAT_CONTEXT => handle_claude_chat_response()?,
        _ => {}
    }

    Ok(())
}

fn handle_embedding_response() -> anyhow::Result<()> {
    let bytes = get_blob().context("Couldn't get blob")?;
    let openai_embedding =
        serde_json::from_slice::<OpenAiEmbeddingResponse>(bytes.bytes.as_slice())?;
    let embedding = openai_embedding.to_embedding_response();
    let response = LLMResponse::Embedding(embedding);
    let _ = Response::new().body(serde_json::to_vec(&response)?).send();
    Ok(())
}

fn handle_chat_response() -> anyhow::Result<()> {
    let bytes = get_blob().context("Couldn't get blob")?;
    let chat_response = serde_json::from_slice::<ChatResponse>(bytes.bytes.as_slice())?;
    let llm_response = LLMResponse::Chat(chat_response);
    let _ = Response::new()
        .body(serde_json::to_vec(&llm_response)?)
        .send()?;
    Ok(())
}

fn handle_claude_chat_response() -> anyhow::Result<()> {
    let bytes = get_blob().context("Couldn't get blob")?;
    let claude_response = serde_json::from_slice::<ClaudeChatResponse>(bytes.bytes.as_slice())?;
    let llm_response = LLMResponse::ClaudeChat(claude_response);
    let _ = Response::new()
        .body(serde_json::to_vec(&llm_response)?)
        .send()?;
    Ok(())
}


fn handle_request(body: &[u8], state: &mut Option<State>) -> anyhow::Result<()> {
    let request = serde_json::from_slice::<LLMRequest>(body)?;
    let context = request_to_context(&request);
    match &request {
        LLMRequest::RegisterOpenaiApiKey(api_request) => {
            register_api_key(api_request, state, ApiKeyType::OpenAI)
        }
        LLMRequest::RegisterGroqApiKey(api_request) => {
            register_api_key(api_request, state, ApiKeyType::Groq)
        }
        LLMRequest::RegisterClaudeApiKey(api_request) => {
            register_api_key(api_request, state, ApiKeyType::Claude)
        }
        LLMRequest::Embedding(embedding_request) => {
            let endpoint = format!("{}/embeddings", OPENAI_BASE_URL);
            handle_generic_request(embedding_request, state, context, &endpoint)
        }
        LLMRequest::OpenaiChat(chat_request) => {
            let endpoint = format!("{}/chat/completions", OPENAI_BASE_URL);
            handle_generic_request(chat_request, state, context, &endpoint)
        }
        LLMRequest::GroqChat(chat_request) => {
            let endpoint = format!("{}/chat/completions", GROQ_BASE_URL);
            handle_generic_request(chat_request, state, context, &endpoint)
        }
        LLMRequest::ChatImage(chat_image_request) => {
            let endpoint = format!("{}/chat/completions", OPENAI_BASE_URL);
            handle_generic_request(chat_image_request, state, context, &endpoint)
        }
        LLMRequest::ClaudeChat(chat_request) => {
            let endpoint = format!("{}/messages", CLAUDE_BASE_URL);
            handle_generic_request(chat_request, state, context, &endpoint)
        }
    }
}

fn register_api_key(
    api_request: &RegisterApiKeyRequest,
    state: &mut Option<State>,
    key_type: ApiKeyType,
) -> anyhow::Result<()> {
    let api_key = &api_request.api_key;
    match state {
        Some(_state) => {
            match key_type {
                ApiKeyType::OpenAI => _state.openai_api_key = api_key.to_string(),
                ApiKeyType::Groq => _state.groq_api_key = api_key.to_string(),
                ApiKeyType::Claude => _state.claude_api_key = api_key.to_string(),
            }
            _state.save();
        }
        None => {
            let mut _state = State::default();
            match key_type {
                ApiKeyType::OpenAI => _state.openai_api_key = api_key.to_string(),
                ApiKeyType::Groq => _state.groq_api_key = api_key.to_string(),
                ApiKeyType::Claude => _state.claude_api_key = api_key.to_string(),
            }
            _state.save();
            *state = Some(_state);
        }
    }
    let _ = Response::new()
        .body(serde_json::to_vec(&LLMResponse::Ok)?)
        .send();
    Ok(())
}


fn handle_generic_request<T: Serialize>(
    request_data: &T,
    state: &mut Option<State>,
    context: u8,
    endpoint: &str,
) -> anyhow::Result<()> {
    let api_key = match context {
        OPENAI_CHAT_CONTEXT | EMBEDDING_CONTEXT | CHAT_IMAGE_CONTEXT => state
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("State not initialized"))?
            .openai_api_key
            .clone(),
        GROQ_CHAT_CONTEXT => state
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("State not initialized"))?
            .groq_api_key
            .clone(),
        CLAUDE_CHAT_CONTEXT => state
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("State not initialized"))?
            .claude_api_key
            .clone(),
        _ => return Err(anyhow::anyhow!("Invalid context for API key")),
    };
    let headers = match context {
        CLAUDE_CHAT_CONTEXT => HashMap::from_iter(vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("x-api-key".to_string(), api_key),
            ("anthropic-version".to_string(), "2023-06-01".to_string()),
        ]),
        _ => HashMap::from_iter(vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Authorization".to_string(), format!("Bearer {}", api_key)),
        ]),
    };
    let outgoing_request = OutgoingHttpRequest {
        method: "POST".to_string(),
        version: None,
        url: endpoint.to_string(),
        headers,
    };
    let body = serde_json::to_vec(&HttpClientAction::Http(outgoing_request))?;
    let bytes = serialize_without_none(request_data)?;
    Request::new()
        .target(Address::new(
            "our",
            ProcessId::new(Some("http_client"), "distro", "sys"),
        ))
        .body(body)
        .expects_response(30)
        .context(vec![context])
        .blob(LazyLoadBlob {
            mime: Some("application/json".to_string()),
            bytes,
        })
        .send()?;

    Ok(())
}

fn handle_message(state: &mut Option<State>) -> anyhow::Result<()> {
    let message = await_message()?;
    if message.is_request() {
        handle_request(message.body(), state)
    } else {
        handle_response(
            message
                .context()
                .context("openai_api: Failed to get context")?,
        )
    }
}

call_init!(init);
fn init(_our: Address) {
    let mut state = State::fetch();
    loop {
        match handle_message(&mut state) {
            Ok(()) => {}
            Err(e) => {
                println!("openai_api: error: {:?}", e);
            }
        };
    }
}
