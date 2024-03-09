use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[serde(rename = "message")]
pub struct MessageResponse {
    pub id: String,
    #[serde(rename = "type")]
    _type: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub _type: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct ContentBlockDeltaDetails {
    #[serde(rename = "type")]
    pub _type: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct Delta {
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    #[serde(flatten)]
    pub usage: Usage,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    MessageStart {
        message: MessageResponse,
    },
    ContentBlockStart {
        index: usize,
        content_block: ContentBlock,
    },
    Ping,
    ContentBlockDelta {
        index: usize,
        delta: ContentBlockDeltaDetails,
    },
    ContentBlockStop {
        index: usize,
    },
    MessageDelta {
        #[serde(flatten)]
        delta: Delta,
      
    },
    MessageStop,
}


#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<'a> {
    pub role: &'a str,
    pub content: &'a str,
}

#[derive(Debug, Serialize)]
pub struct CreateMessageRequest<'a> {
    pub model: &'a str,
    pub max_tokens: u32,
    pub messages: Vec<Message<'a>>,
    pub stream: bool,
}

#[derive(Error, Debug)]
pub enum AnthropicError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error("Unexpected response status: {0}")]
    UnexpectedStatus(u16),

    #[error("Error response from Anthropic: {0}")]
    AnthropicError(String),

    #[error("Error deserializing stream event: {0}")]
    EventDeserializationError(#[from] serde_json::Error),

    #[error("Error converting bytes to string: {0}")]
    BytesToStringError(#[from] std::str::Utf8Error),

    #[error("Error processing values received from Anthropic Responses")]
    ParseResponseError,
}
