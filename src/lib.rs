use std::env;
use reqwest::Client;
use serde::{Serialize, Deserialize};

use thiserror::Error;
use std::fmt;

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
}

#[derive(Error, Debug)]
enum AnthropicError {
   #[error(transparent)]
   ReqwestError(#[from] reqwest::Error),
   #[error("Unexpected response status: {0}")]
   UnexpectedStatus(u16),
   #[error("Error response from Anthropic: {0}")]
   AnthropicError(String),
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