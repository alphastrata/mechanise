use futures_util::Stream;
use futures_util::StreamExt;
use log::error;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use std::env;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

pub mod anthropic_types;
use crate::anthropic_types::*;

static CONTROL_CHAR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\x00-\x1F]").unwrap());

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

    pub async fn create_message<'a>(
        &self,
        model: &'a str,
        max_tokens: u32,
        messages: Vec<Message<'a>>,
    ) -> Result<MessageResponse, AnthropicError> {
        let request = MessageRequest {
            model,
            max_tokens,
            messages,
            stream: false,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key) // Use the correct header name
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

    pub async fn create_message_stream<'a>(
        &self,
        model: &'a str,
        max_tokens: u32,
        messages: Vec<Message<'a>>,
    ) -> Result<UnboundedReceiver<String>, AnthropicError> {
        let request = MessageRequest {
            model,
            messages,
            max_tokens,
            stream: true,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key) // Use the correct header name
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if response.status() != reqwest::StatusCode::OK {
            error!("StatusCode: {}", &response.status());
            let body = response.text().await?;
            return Err(AnthropicError::AnthropicError(body));
        }

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        self.handle_stream(response.bytes_stream(), tx).await?;

        Ok(rx)
    }
    async fn handle_stream<S>(
        &self,
        mut stream: S,
        tx: UnboundedSender<String>,
    ) -> Result<(), AnthropicError>
    where
        S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
    {
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let line = CONTROL_CHAR_REGEX.replace_all(std::str::from_utf8(&chunk)?, "");

            
            if let Some(line) = line.strip_suffix("data: ") {
                let ev = StreamEvent::parse(&line)?;
                if let Some(resp_text) = handle_event(ev) {
                    _ = tx.send(resp_text);
                }
            }   
        }

        Ok(())
    }
}

fn handle_event(event: StreamEvent) -> Option<String> {
    match event {
        StreamEvent::MessageStart { message } => {
            if let Some(content) = message.content.last() {
                return Some(content.text.to_owned());
            }
        }
        StreamEvent::ContentBlockStart {
            index,
            content_block,
        } => {
            return Some(content_block.text);
        }
        StreamEvent::Ping => {}
        StreamEvent::ContentBlockDelta { index, delta } => {
            return Some(delta.text);
        }
        StreamEvent::ContentBlockStop { index } => {}
        StreamEvent::MessageDelta { delta, .. } => {}
        StreamEvent::MessageStop => {
            dbg!("stop");
        }
    }
    None
}
impl StreamEvent {
    fn parse(data: &str) -> Result<Self, AnthropicError> {
        Ok(serde_json::from_str::<Self>(data)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::fs::File;
    use tokio::io::AsyncBufReadExt;
    use tokio::io::BufReader;

    // const TEST_PROMPT: &str = "Write me a rust function that generates a SECURE password of length `n`. Ideally, use the openssl crate, iterator patterns and be idiomatic. respond ONLY with the code, I do NOT require an explination.";
    const TEST_PROMPT: &str = "Hello claude!";

    #[ignore = "let's not waste API credits"]
    #[tokio::test]
    async fn runit() {
        pretty_env_logger::try_init().ok();
        let client = AnthropicClient::new();
        let messages = vec![Message {
            role: "user",
            content: TEST_PROMPT,
        }];
        let response = client
            .create_message("claude-3-opus-20240229", 128, messages)
            .await;

        match response {
            Ok(res) => {
                println!("Response: {:?}", res);
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                panic!();
            }
        }
    }

    #[ignore = "let's not waste API credits"]
    #[tokio::test]
    async fn test_create_message_stream() {
        pretty_env_logger::try_init().ok();
        let client = AnthropicClient::new();
        let messages = vec![Message {
            role: "user",
            content: TEST_PROMPT,
        }];

        let result = client
            .create_message_stream("claude-3-opus-20240229", 2, messages) // I'm a cheapskate :p
            .await;

        match result {
            Ok(resp) => {
                dbg!(resp);
            }
            Err(err) => {
                panic!("Error during streaming: {}", err);
            }
        }
    }

    #[tokio::test]
    async fn can_parse() -> Result<(), Box<dyn std::error::Error>> {
        let file_path = "raw_chunk.jsonl"; // Adjust the file path as necessary
        let file = File::open(file_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            if line.starts_with("event: ") {
                continue;
            } else if line.starts_with("data: ") {
                let current_data = line["data: ".len()..].to_string();
                assert!(StreamEvent::parse(&current_data).is_ok());
            }
        }

        Ok(())
    }
}
