use reqwest::Client;
use serde::Serialize;
use std::env;

pub mod requests;
#[cfg(feature = "streaming")]
pub mod streaming;
pub mod anthropic_types;

pub use crate::anthropic_types::*;
pub use requests::*;


pub struct AnthropicClient {
    client: Client,
    api_key: String,
}

impl Default for AnthropicClient {
    fn default() -> Self {
        Self::new()
    }
}

impl AnthropicClient {
    pub fn new() -> Self {
        let api_key = env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");
        let client = Client::new();
        Self { client, api_key }
    }

    pub async fn create_message<'a, T>(
        &self,
        request: &T,
    ) -> Result<MessageResponse, AnthropicError> where T: Serialize {
      
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key) // Use the correct header name
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(request)
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
mod test {
    use super::*;
  
    pub const TEST_PROMPT: &str = "Write me a rust function that generates a SECURE password of length `n`. Ideally, use the openssl crate, iterator patterns and be idiomatic. respond ONLY with the code, I do NOT require an explination.";
    
    #[ignore = "let's not waste API credits"]
    #[tokio::test]
    async fn run_single_resp() {
        let client = AnthropicClient::new();
        let messages = vec![Message {
            role: "user",
            content: TEST_PROMPT,
        }];

        let request = SimpleMessageRequest {
            model: "claude-3-opus-20240229",
            messages,
            max_tokens: 128,
            stream: true,
        };
        let response = client
            .create_message(&request)
            .await;

        match response {
            Ok(res) => {
                println!("Response: {:?}", res);
            }
            Err(err) => {
                panic!("Error: {}", err);
            }
        }
    }
}
