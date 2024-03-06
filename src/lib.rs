
#![allow(dead_code)]
use std::env;
use reqwest::{Client};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use std::fmt;
use tokio::io::{AsyncBufReadExt, BufReader};


#[derive(Debug, Deserialize)]
struct Usage {
   input_tokens: u32,
   output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
   id: String,
   #[serde(rename = "type")]
   type_field: String,
   role: String,
   content: Vec<ContentBlock>,
   model: String,
   stop_reason: String,
   stop_sequence: Option<String>,
   usage: Usage,
}


#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum StreamEvent {
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
        delta: ContentBlockDelta,
    },
    ContentBlockStop {
        index: usize,
    },
    MessageDelta {
        delta: MessageResponseDelta,
    },
    MessageStop,
}

#[derive(Debug, Deserialize)]
struct ContentBlockDelta {
    #[serde(rename = "type")]
    type_field: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct MessageResponseDelta {
    stop_reason: String,
    stop_sequence: Option<String>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
   #[serde(rename = "type")]
   type_field: String,
   text: String,
}

#[derive(Debug, Serialize)]
struct Message<'a> {
   role: &'a str,
   content: &'a str,
}

#[derive(Debug, Serialize)]
struct CreateMessageRequest<'a> {
   model: &'a str,
   max_tokens: u32,
   messages: Vec<Message<'a>>,
   steam: bool
}

#[derive(Error, Debug)]
enum AnthropicError {
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
}


struct AnthropicClient {
   client: Client,
   api_key: String,
}

impl AnthropicClient {
   fn new() -> Self {
       let api_key = env::var("ANTHROPIC_API_KEY")
           .expect("ANTHROPIC_API_KEY must be set");
       let client = Client::new();
       Self { client, api_key }
   }

   async fn create_message<'a>(&self, model: &'a str, max_tokens: u32, messages: Vec<Message<'a>>) -> Result<MessageResponse, AnthropicError> {
       let request = CreateMessageRequest {
           model,
           max_tokens,
           messages,
           steam: false
       };

       let response = self.client
           .post("https://api.anthropic.com/v1/messages")
           .bearer_auth(&self.api_key)
           .header("anthropic-version", "2023-06-01")
           .header("content-type", "application/json")
           .json(&request)
           .send()
           .await?;

       let status = response.status().as_u16();
       if status != 200 {
           let body = response.text().await?;
           return Err(AnthropicError::AnthropicError(body));
       }

       let response_body = response.json().await?;
       Ok(response_body)
   }

   
}


//snip1

use futures_util::StreamExt;
use futures_util::Stream;

impl AnthropicClient {
    // ... (previous implementations)

    async fn create_message_stream<'a>(
        &self,
        model: &'a str,
        max_tokens: u32,
        messages: Vec<Message<'a>>,
    ) -> Result<(), AnthropicError> {
        let request = CreateMessageRequest {
            model,
            max_tokens,
            messages,
            steam: true,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .bearer_auth(&self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if response.status() != reqwest::StatusCode::OK {
            let body = response.text().await?;
            return Err(AnthropicError::AnthropicError(body));
        }

        self.handle_stream(response.bytes_stream()).await
    }

    async fn handle_stream<S>(&self, mut stream: S) -> Result<(), AnthropicError>
    where
        S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
    {
        let mut message_response = None;
        let mut content_blocks = Vec::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let line = std::str::from_utf8(&chunk)?;

            if line.starts_with("event: ") {
                let event_type = &line["event: ".len()..];
                let event = serde_json::from_str::<StreamEvent>(&format!(r#"{{"type":"{}"}}"#, event_type))
                    .map_err(AnthropicError::EventDeserializationError)?;

                match event {
                    StreamEvent::MessageStart { message } => {
                        message_response = Some(message);
                        content_blocks.clear();
                    }
                    StreamEvent::ContentBlockStart { index, content_block } => {
                        content_blocks.insert(index, content_block);
                    }
                    StreamEvent::Ping => {
                        // Ignore ping events
                    }
                    StreamEvent::ContentBlockDelta { index, delta } => {
                        if let Some(block) = content_blocks.get_mut(index) {
                            block.text.push_str(&delta.text);
                        } else {
                            return Err(AnthropicError::AnthropicError(
                                "Received content block delta for unknown index".to_string(),
                            ));
                        }
                    }
                    StreamEvent::ContentBlockStop { index: _ } => {
                        // Ignore content block stop events
                    }
                    StreamEvent::MessageDelta { delta } => {
                        if let Some(ref mut message) = message_response {
                            message.stop_reason = delta.stop_reason;
                            message.stop_sequence = delta.stop_sequence;
                            message.usage = delta.usage;
                        } else {
                            return Err(AnthropicError::AnthropicError(
                                "Received message delta before message start".to_string(),
                            ));
                        }
                    }
                    StreamEvent::MessageStop => {
                        if let Some(ref message) = message_response {
                            println!("Final message response: {:?}", message);
                            println!("Content blocks: {:?}", content_blocks);
                        } else {
                            return Err(AnthropicError::AnthropicError(
                                "Received message stop before message start".to_string(),
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test{
use super::*;
const TEST_PROMPT: &'static str = "write me a rust function that generates a SECURE password of length `n`. Ideally, use the openssl crate, iterator patterns and be idiomatic. respond only with the code.";

#[tokio::test]
async fn runit() {
    let client = AnthropicClient::new();
    let messages = vec![Message { role: "user", content: TEST_PROMPT }];
    let response = client.create_message("claude-3-opus-20240229", 1024, messages).await;

    match response {
        Ok(res) => {
            println!("Response: {:?}", res);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}
}