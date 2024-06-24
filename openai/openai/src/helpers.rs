use llm_interface::openai::EmbeddingResponse;
use llm_interface::openai::Usage;
use llm_interface::openai::LLMRequest;
use serde::Deserialize;
use serde::Serialize;

pub const REGISTER_API_KEY_CONTEXT: u8 = 0;
pub const EMBEDDING_CONTEXT: u8 = 1;
pub const OPENAI_CHAT_CONTEXT: u8 = 2;
pub const GROQ_CHAT_CONTEXT: u8 = 3;
pub const CHAT_IMAGE_CONTEXT: u8 = 4;

// TODO: Zena: We should probably derive this through a trait at some point?
pub fn request_to_context(request: &LLMRequest) -> u8 {
    match request {
        LLMRequest::RegisterGroqApiKey(_) | LLMRequest::RegisterOpenaiApiKey(_) => REGISTER_API_KEY_CONTEXT,
        LLMRequest::Embedding(_) => EMBEDDING_CONTEXT,
        LLMRequest::OpenaiChat(_) => OPENAI_CHAT_CONTEXT,
        LLMRequest::GroqChat(_) => GROQ_CHAT_CONTEXT,
        LLMRequest::ChatImage(_) => CHAT_IMAGE_CONTEXT,
    }
}

pub fn serialize_without_none<T: Serialize>(input: &T) -> serde_json::Result<Vec<u8>> {
    let mut value = serde_json::to_value(input)?;
    if let serde_json::Value::Object(ref mut map) = value {
        map.retain(|_, v| !v.is_null());
    }
    serde_json::to_vec(&value)
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OpenAiEmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: Usage,
}

impl OpenAiEmbeddingResponse {
    pub fn to_embedding_response(&self) -> EmbeddingResponse {
        let embedding_values: Vec<Vec<f32>> = self.data.iter()
            .map(|embedding_data| embedding_data.embedding.iter().map(|&value| value as f32).collect())
            .collect();
        EmbeddingResponse {
            embeddings: embedding_values,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EmbeddingData {
    pub object: String,
    pub index: u32,
    pub embedding: Vec<f64>,
}

