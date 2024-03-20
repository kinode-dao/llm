pub mod lccp {

    use serde::Deserialize;
    use serde::Serialize;
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub enum LLMRequest {
        Embedding(EmbeddingRequest),
        Chat(ChatRequest),
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub enum LLMResponse {
        Embedding(EmbeddingResponse),
        Chat(ChatResponse),
        ChatStreaming(ChatStreamingResponse),
    }

    impl LLMRequest {
        pub fn to_bytes(&self) -> Vec<u8> {
            match self {
                LLMRequest::Chat(_) | LLMRequest::Embedding(_) => serde_json::to_vec(self).unwrap(),
            }
        }

        pub fn parse(bytes: &[u8]) -> Result<LLMRequest, serde_json::Error> {
            serde_json::from_slice::<LLMRequest>(bytes)
        }
    }

    impl LLMResponse {
        pub fn to_bytes(&self) -> Vec<u8> {
            match self {
                LLMResponse::Chat(_)
                | LLMResponse::Embedding(_)
                | LLMResponse::ChatStreaming(_) => serde_json::to_vec(self).unwrap(),
            }
        }

        pub fn parse(bytes: &[u8]) -> Result<LLMResponse, serde_json::Error> {
            serde_json::from_slice::<LLMResponse>(bytes)
        }
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct EmbeddingRequest {
        pub content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub image_data: Option<Vec<ImageData>>, // Optional, since it's not always multimodal
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct ImageData {
        pub data: String, // Base64 encoded image data
        pub id: u32,      // Image identifier
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct EmbeddingResponse {
        pub embedding: Vec<f32>,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct ChatRequest {
        pub prompt: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub temperature: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub dynatemp_range: Option<(f64, f64)>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub dynatemp_exponent: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub top_k: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub top_p: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub min_p: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub n_predict: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub n_keep: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub stream: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub stop: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tfs_z: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub typical_p: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub repeat_penalty: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub repeat_last_n: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub penalize_nl: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub presence_penalty: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub frequency_penalty: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub penalty_prompt: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub mirostat: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub mirostat_tau: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub mirostat_eta: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub grammar: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub seed: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub ignore_eos: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub logit_bias: Option<Vec<(i32, f64)>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub n_probs: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub min_keep: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub image_data: Option<Vec<ImageData>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub slot_id: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub cache_prompt: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub system_prompt: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub samplers: Option<Vec<String>>,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct ChatResponse {
        pub content: String,
        pub generation_settings: GenerationSettings,
        pub model: String,
        pub prompt: String,
        pub slot_id: i32,
        pub stop: bool,
        pub stopped_eos: bool,
        pub stopped_limit: bool,
        pub stopped_word: bool,
        pub stopping_word: String,
        pub timings: Timings,
        pub tokens_cached: i32,
        pub tokens_evaluated: i32,
        pub tokens_predicted: i32,
        pub truncated: bool,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct ChatStreamingResponse {
        pub content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub generation_settings: Option<GenerationSettings>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub model: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prompt: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub slot_id: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub stop: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub stopped_eos: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub stopped_limit: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub stopped_word: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub stopping_word: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub timings: Option<Timings>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tokens_cached: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tokens_evaluated: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tokens_predicted: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub truncated: Option<bool>,
    }
    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct GenerationSettings {
        pub dynatemp_exponent: f64,
        pub dynatemp_range: f64,
        pub frequency_penalty: f64,
        pub grammar: String,
        pub ignore_eos: bool,
        pub logit_bias: Vec<serde_json::Value>, // Adjust according to actual type
        pub min_keep: i32,
        pub min_p: f64,
        pub mirostat: i32,
        pub mirostat_eta: f64,
        pub mirostat_tau: f64,
        pub model: String,
        pub n_ctx: i32,
        pub n_keep: i32,
        pub n_predict: i32,
        pub n_probs: i32,
        pub penalize_nl: bool,
        pub penalty_prompt_tokens: Vec<serde_json::Value>, // Adjust according to actual type
        pub presence_penalty: f64,
        pub repeat_last_n: i32,
        pub repeat_penalty: f64,
        pub samplers: Vec<String>,
        pub seed: u32,
        pub stop: Vec<serde_json::Value>, // Adjust according to actual type
        pub stream: bool,
        pub temperature: f64,
        pub tfs_z: f64,
        pub top_k: i32,
        pub top_p: f64,
        pub typical_p: f64,
        pub use_penalty_prompt_tokens: bool,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct Timings {
        pub predicted_ms: f64,
        pub predicted_n: i32,
        pub predicted_per_second: f64,
        pub predicted_per_token_ms: f64,
        pub prompt_ms: f64,
        pub prompt_n: i32,
        pub prompt_per_second: f64,
        pub prompt_per_token_ms: f64,
    }
}

#[allow(dead_code)]
pub mod openai {
    use serde::Deserialize;
    use serde::Serialize;
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub enum LLMRequest {
        Embedding(EmbeddingRequest),
        Chat(ChatRequest),
    }

    impl LLMRequest {
        pub fn new_chat_request(api_key: String, params: ChatParams) -> Self {
            let chat_request = ChatRequest { api_key, params };
            LLMRequest::Chat(chat_request)
        }

        pub fn to_bytes(&self) -> Vec<u8> {
            match self {
                LLMRequest::Chat(_) | LLMRequest::Embedding(_) => serde_json::to_vec(self).unwrap(),
            }
        }

        pub fn parse(bytes: &[u8]) -> Result<LLMRequest, serde_json::Error> {
            serde_json::from_slice::<LLMRequest>(bytes)
        }
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct EmbeddingRequest {
        pub api_key: String,
        pub params: EmbeddingParams,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct EmbeddingParams {
        pub input: String,
        pub model: String,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct ChatRequest {
        pub api_key: String,
        pub params: ChatParams,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct ChatParams {
        pub model: String,
        pub messages: Vec<Message>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub frequency_penalty: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub logit_bias: Option<HashMap<String, i32>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub logprobs: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub top_logprobs: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max_tokens: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub n: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub presence_penalty: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub response_format: Option<ResponseFormat>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub seed: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub stop: Option<Stop>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub stream: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub temperature: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub top_p: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tools: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tool_choice: Option<ToolChoice>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user: Option<String>,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct Message {
        pub role: String,
        pub content: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    #[serde(untagged)]
    pub enum ResponseFormat {
        JsonObject { type_field: String },
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    #[serde(untagged)]
    pub enum Stop {
        String(String),
        Array(Vec<String>),
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    #[serde(untagged)]
    pub enum ToolChoice {
        None,
        Auto,
        SpecificFunction {
            type_field: String,
            function: Function,
        },
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct Function {
        pub name: String,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct ChatResponse {
        pub id: String,
        pub object: String,
        pub created: i64,
        pub model: String,
        pub system_fingerprint: String,
        pub choices: Vec<Choice>,
        pub usage: Usage,
    }

    impl ChatResponse {
        pub fn to_chat_response(&self) -> String {
            self.choices[0].message.content.clone()
        }

        pub fn to_message_response(&self) -> Message {
            self.choices[0].message.clone()
        }
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct Choice {
        pub index: i32,
        pub message: Message,
        pub logprobs: Option<serde_json::Value>,
        pub finish_reason: String,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct Usage {
        pub prompt_tokens: i32,
        pub completion_tokens: Option<i32>,
        pub total_tokens: i32,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub enum LLMResponse {
        Embedding(EmbeddingResponse),
        Chat(ChatResponse),
    }

    impl LLMResponse {
        pub fn to_bytes(&self) -> Vec<u8> {
            match self {
                LLMResponse::Chat(_) | LLMResponse::Embedding(_) => {
                    serde_json::to_vec(self).unwrap()
                }
            }
        }

        pub fn parse(bytes: &[u8]) -> Result<LLMResponse, serde_json::Error> {
            serde_json::from_slice::<LLMResponse>(bytes)
        }
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct EmbeddingResponse {
        pub embedding: Vec<f32>,
    }

    impl EmbeddingResponse {
        pub fn from_openai_response(openai_response: OpenAiEmbeddingResponse) -> Self {
            let embedding_values: Vec<f32> = openai_response.data[0]
                .embedding
                .iter()
                .map(|&value| value as f32)
                .collect();
            EmbeddingResponse {
                embedding: embedding_values,
            }
        }
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct OpenAiEmbeddingResponse {
        pub object: String,
        pub data: Vec<EmbeddingData>,
        pub model: String,
        pub usage: Usage,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct EmbeddingData {
        pub object: String,
        pub index: u32,
        pub embedding: Vec<f64>,
    }
}

pub mod groq {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub enum ChatCompletionRequest {
        ChatCompletion(ChatCompletionParams),
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct ChatCompletionParams {
        pub model: String,
        pub messages: Vec<Message>,
        pub seed: Option<u32>,
        pub temperature: Option<f64>,
        pub max_tokens: Option<i32>,
        pub top_p: Option<f64>,
        pub stream: Option<bool>,
        pub stop: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct Message {
        pub role: String,
        pub content: String,
        pub name: Option<String>,
    }

    impl ChatCompletionRequest {
        pub fn new_chat_completion_request(
            model: String,
            messages: Vec<Message>,
            seed: Option<u32>,
        ) -> Self {
            let chat_completion_params = ChatCompletionParams {
                model,
                messages,
                seed,
                temperature: None,
                max_tokens: None,
                top_p: None,
                stream: None,
                stop: None,
            };
            ChatCompletionRequest::ChatCompletion(chat_completion_params)
        }

        pub fn to_bytes(&self) -> Vec<u8> {
            serde_json::to_vec(self).unwrap()
        }

        pub fn parse(bytes: &[u8]) -> Result<ChatCompletionRequest, serde_json::Error> {
            serde_json::from_slice::<ChatCompletionRequest>(bytes)
        }
    }
}
