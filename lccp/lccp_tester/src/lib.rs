use kinode_process_lib::{await_message, get_blob, call_init, println, Address, ProcessId, Request};
use interface::lccp::{ChatRequest, EmbeddingRequest, LLMRequest, LLMResponse};
use anyhow::Context;

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

const PROMPT: &str = "Where is my mind?";
const RESPONSE: &str = "\nI am not sure. I have been thinking about it for a long time, but I can't seem to find the answer. It's like my thoughts are scattered and I can't focus on anything.\n\nWhat is the purpose of this";

call_init!(init);

fn init(_our: Address) {
    println!("Tester: begin");
    match run_test() {
        Ok(_) => println!("Tester: Great Success!"),
        Err(e) => println!("Tester: Failure! {:?}", e),
    }
}

fn are_equal(a: f32, b: f32) -> bool {
    (a - b).abs() < std::f32::EPSILON
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
            ProcessId::new(Some("lccp"), "llm", "kinode"),
        ))
        .body(request.to_bytes())
        .send_and_await_response(response_timeout)??;
    let response = LLMResponse::parse(msg.body())?;
    validate(response)
}

fn test_embedding() -> anyhow::Result<()> {
    let embedding_params = EmbeddingRequest {
        content: PROMPT.to_string(),
        image_data: None,
    };
    send_request_and_validate_response(LLMRequest::Embedding(embedding_params), 3, |response| {
        if let LLMResponse::Embedding(embedding_response) = response {
            assert!(embedding_response.embedding.len() == 2560);
            assert!(are_equal(embedding_response.embedding[0], -0.8872467));
            Ok(())
        } else {
            Err(anyhow::anyhow!("Expected Embedding response"))
        }
    })
}

fn test_chat_non_streaming() -> anyhow::Result<()> {
    let chat_params = ChatRequest {
        prompt: PROMPT.to_string(),
        n_predict: Some(50),
        temperature: Some(0.0),
        ..Default::default()
    };
    send_request_and_validate_response(LLMRequest::Chat(chat_params), 3, |response| {
        if let LLMResponse::Chat(chat_response) = response {
            assert_eq!(chat_response.content, RESPONSE);
            println!("Non streaming chat works");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Expected Chat response"))
        }
    })
}

fn _test_chat_streaming() -> anyhow::Result<()> {
    let chat_params = ChatRequest {
        prompt: PROMPT.to_string(),
        n_predict: Some(50),
        temperature: Some(0.0),
        stream: Some(true),
        ..Default::default()
    };
    let request = LLMRequest::Chat(chat_params);
    let _ = Request::new()
        .target(Address::new(
            "our",
            ProcessId::new(Some("lccp"), "llm", "kinode"),
        ))
        .body(request.to_bytes())
        .send();
    loop {
        let message = await_message()?;
        if !message.is_request() {
            let bytes = get_blob().context("Couldn't get blob")?;
            let response = LLMResponse::parse(bytes.bytes.as_slice())?;
            if let LLMResponse::ChatStreaming(chat_response) = response {
                println!("Chat response: {:?}", chat_response);
                if chat_response.stop.unwrap_or(false) {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn run_test() -> anyhow::Result<()> {
    let total_start = std::time::Instant::now();

    let start = std::time::Instant::now();
    test_embedding()?;
    println!("Tester: Embedding took {}ms", start.elapsed().as_millis());

    let start = std::time::Instant::now();
    test_chat_non_streaming()?;
    println!("Tester: Non streaming chat took {}ms", start.elapsed().as_millis());

    // TODO: Zen: Implement once SSE handling is done
    // let start = std::time::Instant::now();
    // test_chat_streaming()?;
    // println!("Tester: Streaming chat took {}ms", start.elapsed().as_millis());

    println!(
        "Tester: All tests passed, and it took {}ms",
        total_start.elapsed().as_millis()
    );
    Ok(())
}
