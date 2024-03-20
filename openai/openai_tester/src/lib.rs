use kinode_process_lib::{call_init, println, Address, ProcessId, Request};

use common_types::openai::{ChatParams, ChatRequest, EmbeddingParams, EmbeddingRequest, LLMRequest, LLMResponse, Message, Provider};
use kinode_process_lib::await_next_request_body;

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

call_init!(init);

fn init(_our: Address) {
    println!("Tester: begin");
    match run_test() {
        Ok(_) => println!("Tester: Great Success!"),
        Err(e) => println!("Tester: Failure! {:?}", e),
    }
}

fn send_request_and_validate_response<
    T: Into<LLMRequest>,
    F: Fn(LLMResponse) -> anyhow::Result<()>,
>(
    request_params: T,
    response_timeout: u64,
    validate: F,
) -> anyhow::Result<()> {
    let request = request_params.into();
    let msg = Request::new()
        .target(Address::new(
            "our",
            ProcessId::new(Some("openai"), "llm", "kinode"),
        ))
        .body(request.to_bytes())
        .send_and_await_response(response_timeout)??;
    let response = LLMResponse::parse(msg.body())?;
    validate(response)
}

fn test_embedding(api_key: &str) -> anyhow::Result<()> {
    let embedding_params = EmbeddingParams {
        input: "Where is my mind?".to_string(),
        model: "text-embedding-3-small".to_string(),
    };
    let embedding_request = EmbeddingRequest {
        params: embedding_params,
        api_key: api_key.to_string(),
    };
    send_request_and_validate_response(LLMRequest::Embedding(embedding_request), 30, |response| {
        if let LLMResponse::Embedding(embedding_response) = response {
            assert_eq!(embedding_response.embedding.len(), 1536);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Expected Embedding response"))
        }
    })
}

fn test_chat(api_key: &str) -> anyhow::Result<()> {
    let chat_params = ChatParams {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "Hello!".to_string(),
            },
        ],
        ..Default::default()
    };
    let chat_request = ChatRequest {
        params: chat_params,
        api_key: api_key.to_string(),
        provider: Provider::OpenAi,
    };
    send_request_and_validate_response(LLMRequest::Chat(chat_request), 30, |response| {
        if let LLMResponse::Chat(chat_response) = response {
            assert_eq!(chat_response.object, "chat.completion");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Expected Chat response"))
        }
    })
}

fn run_test() -> anyhow::Result<()> {
    let total_start = std::time::Instant::now();
    println!("Openai Tester: Start");

    let Ok(args) = await_next_request_body() else {
        println!("echo: failed to get args, aborting");
        return Ok(());
    };
    let api_key = String::from_utf8_lossy(&args).to_string();

    let embedding_start = std::time::Instant::now();
    test_embedding(&api_key)?;
    println!("Test embedding took {}ms.", embedding_start.elapsed().as_millis());

    let chat_start = std::time::Instant::now();
    test_chat(&api_key)?;
    println!("Test chat took {}ms.", chat_start.elapsed().as_millis());

    println!("Test took {}ms in total.", total_start.elapsed().as_millis());
    Ok(())
}
