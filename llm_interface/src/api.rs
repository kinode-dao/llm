pub mod openai {
    use kinode_process_lib::{our_capabilities, spawn, Address, OnExit, ProcessId, Request};
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;

    use crate::openai::{ChatParams, ChatRequest, LLMRequest, LLMResponse, Message, Provider, ChatImageRequest, ChatImageParams};

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
            self.send_request_and_parse_response(request)
        }

        pub fn chat_with_image(
            &self,
            chat_image_params: ChatImageParams,
        ) -> anyhow::Result<Message> {
            let chat_image_request = ChatImageRequest {
                params: chat_image_params,
                api_key: self.openai_key.clone(),
                provider: Provider::OpenAi,
            };
            let request = LLMRequest::ChatImage(chat_image_request);
            self.send_request_and_parse_response(request)
        }

        // Private method to send request and parse response
        fn send_request_and_parse_response(&self, request: LLMRequest) -> anyhow::Result<Message> {
            let res = Request::new()
                .target(self.openai_worker.clone())
                .body(request.to_bytes())
                .send_and_await_response(10);

            match res {
                Ok(res) => {
                    match res {
                        Ok(res) => {
                            let res = LLMResponse::parse(res.body());
                            match res {
                                Ok(res) => {
                                    match res {
                                        LLMResponse::Chat(chat) => Ok(chat.to_message_response()),
                                        _ => {
                                            println!("Error querying OpenAI: wrong result type");
                                            Err(anyhow::Error::msg("Error querying OpenAI: wrong result type"))
                                        }
                                    }
                                },
                                Err(e) => {
                                    println!("Failed to parse response: {:?}", e);
                                    Err(anyhow::Error::msg(format!("Failed to parse response: {:?}", e)))
                                },
                            }
                        },
                        Err(e) => {
                            println!("Failed to send request or await response: {:?}", e);
                            Err(anyhow::Error::msg(format!("Failed to send request or await response: {:?}", e)))
                        },
                    }
                },
                Err(e) => {
                    println!("Failed to send request or await response: {:?}", e);
                    Err(anyhow::Error::msg(format!("Failed to send request or await response: {:?}", e)))
                }
            }
        }
    }
}

pub mod lccp {
    // TODO:
}
