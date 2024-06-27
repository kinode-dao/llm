
pub mod openai {
    use serde::Deserialize;
    use serde::Serialize;
    use std::collections::HashMap;
    use derive_builder::Builder;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub enum LLMRequest {
        RegisterOpenaiApiKey(RegisterApiKeyRequest),
        RegisterGroqApiKey(RegisterApiKeyRequest),
        RegisterClaudeApiKey(RegisterApiKeyRequest),
        Embedding(EmbeddingRequest),
        OpenaiChat(ChatRequest),
        GroqChat(ChatRequest),
        ChatImage(ChatImageRequest),
        ClaudeChat(ClaudeChatRequest),
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Builder)]
    pub struct RegisterApiKeyRequest {
        pub api_key: String,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Builder)]
    pub struct EmbeddingRequest {
        pub input: Vec<String>,
        pub model: String,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Builder)]
    pub struct ClaudeChatRequest {
        pub model: String,
        pub messages: Vec<Message>,
        #[builder(default)]
        pub max_tokens: Option<i32>,
        // Add other optional fields as needed
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Builder)]
    pub struct ChatRequest {
        pub model: String,
        pub messages: Vec<Message>,
        #[builder(default)]
        pub frequency_penalty: Option<f64>,
        #[builder(default)]
        pub logit_bias: Option<HashMap<String, i32>>,
        #[builder(default)]
        pub logprobs: Option<bool>,
        #[builder(default)]
        pub top_logprobs: Option<i32>,
        #[builder(default)]
        pub max_tokens: Option<i32>,
        #[builder(default)]
        pub n: Option<i32>,
        #[builder(default)]
        pub presence_penalty: Option<f64>,
        #[builder(default)]
        pub response_format: Option<ResponseFormat>,
        #[builder(default)]
        pub seed: Option<i32>,
        #[builder(default)]
        pub stop: Option<Stop>,
        #[builder(default)]
        pub stream: Option<bool>,
        #[builder(default)]
        pub temperature: Option<f64>,
        #[builder(default)]
        pub top_p: Option<f64>,
        #[builder(default)]
        pub tools: Option<Vec<String>>,
        #[builder(default)]
        pub tool_choice: Option<ToolChoice>,
        #[builder(default)]
        pub user: Option<String>,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Builder)]
    pub struct ChatImageRequest {
        pub model: String,
        pub messages: Vec<ChatImageMessage>,
        #[builder(default)]
        pub frequency_penalty: Option<f64>,
        #[builder(default)]
        pub logit_bias: Option<HashMap<String, i32>>,
        #[builder(default)]
        pub logprobs: Option<bool>,
        #[builder(default)]
        pub top_logprobs: Option<i32>,
        #[builder(default)]
        pub max_tokens: Option<i32>,
        #[builder(default)]
        pub n: Option<i32>,
        #[builder(default)]
        pub presence_penalty: Option<f64>,
        #[builder(default)]
        pub response_format: Option<ResponseFormat>,
        #[builder(default)]
        pub seed: Option<i32>,
        #[builder(default)]
        pub stop: Option<Stop>,
        #[builder(default)]
        pub stream: Option<bool>,
        #[builder(default)]
        pub temperature: Option<f64>,
        #[builder(default)]
        pub top_p: Option<f64>,
        #[builder(default)]
        pub tools: Option<Vec<String>>,
        #[builder(default)]
        pub tool_choice: Option<ToolChoice>,
        #[builder(default)]
        pub user: Option<String>,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Builder)]
    pub struct Message {
        pub role: String,
        pub content: String,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Builder)]
    pub struct ChatImageMessage {
        pub role: String,
        pub content: Vec<ChatImageContent>,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Builder)]
    pub struct ChatImageContent {
        #[serde(rename = "type")]
        pub content_type: String,
        #[builder(default)]
        pub text: Option<String>,
        #[serde(rename = "image_url")]
        #[builder(default)]
        pub image_url: Option<ImageUrl>,
    }

    
    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Builder)]
    pub struct ImageUrl {
        pub url: String,
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

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Builder)]
    pub struct Function {
        pub name: String,
    }

    // TODO: Zena: Well great, we probably need an intermediate struct for the response.
    // Why does openai make the jsons so dumb? People just want the response. (self.choices[0].message.content.clone())
    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct ChatResponse {
        pub id: Option<String>,
        pub object: Option<String>,
        pub created: Option<i64>,
        pub model: Option<String>,
        pub system_fingerprint: Option<String>,
        pub choices: Vec<Choice>,
        pub usage: Option<Usage>,
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
        Ok,
        Embedding(EmbeddingResponse),
        Chat(ChatResponse),
        ClaudeChat(ClaudeChatResponse),
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct ClaudeChatResponse {
        pub content: Vec<Content>,
        pub id: String,
        pub model: String,
        pub role: String,
        pub stop_reason: String,
        pub stop_sequence: Option<String>,
        #[serde(rename = "type")]
        pub response_type: String,
        pub usage: Usage,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct Content {
        #[serde(rename = "type")]
        pub content_type: String,
        pub text: String,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct EmbeddingResponse {
        pub embeddings: Vec<Vec<f32>>, 
    }

}

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