use futures_util::Stream;
use futures_util::StreamExt;
use log::error;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use std::env;

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
        let request = CreateMessageRequest {
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
    ) -> Result<(), AnthropicError> {
        let request = CreateMessageRequest {
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

        self.handle_stream(response.bytes_stream()).await
    }
    async fn handle_stream<S>(&self, mut stream: S) -> Result<(), AnthropicError>
    where
        S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
    {
        let mut event_type = String::new();
        let mut data_json = String::new();
        let mut processing_data = false;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let line = std::str::from_utf8(&chunk)?;
            _ = append_chunk_to_file(line, "raw_chunk.jsonl");
            let line = CONTROL_CHAR_REGEX.replace_all(line, "");

            if line.starts_with("event: ") {
                if !event_type.is_empty() && !data_json.is_empty() {
                    // Process the previous event
                    let full_event_json =
                        format!(r#"{{"type":"{}","data":{}}}"#, event_type, data_json);

                    if let Ok(event) = serde_json::from_str::<StreamEvent>(&full_event_json) {
                        handle_event(event);
                    }
                    event_type.clear();
                    data_json.clear();
                }
                event_type = line["event: ".len()..].trim().to_string();
                processing_data = false;
            } else if line.starts_with("data: ") {
                data_json = line["data: ".len()..].trim().to_string();
                processing_data = true;
            }

            if processing_data {
                // Attempt to deserialize and process the event here
                let full_event_json =
                    format!(r#"{{"type":"{}","data":{}}}"#, event_type, data_json);
                if let Ok(event) = serde_json::from_str::<StreamEvent>(&full_event_json) {
                    handle_event(event);
                } else {
                    eprintln!("{}", full_event_json);
                    panic!()
                }
                event_type.clear();
                data_json.clear();
                processing_data = false;
            }
        }

        Ok(())
    }
}

fn handle_event(event: StreamEvent) {
    match event {
        StreamEvent::MessageStart { message } => {
            if let Some(content) = message.content.last() {
                print!("{}", content.text);
                _ = append_chunk_to_file(&content.text, "resp.txt");
            }
        }
        StreamEvent::ContentBlockStart {
            index,
            content_block,
        } => {
            print!("{}", content_block.text);
            _ = append_chunk_to_file(&content_block.text, "resp.txt");
        }
        StreamEvent::Ping => {}
        StreamEvent::ContentBlockDelta { index, delta } => {
            _ = append_chunk_to_file(&delta.text, "resp.txt");

            print!("{}", delta.text);
        }
        StreamEvent::ContentBlockStop { index ,} => {}
        StreamEvent::MessageDelta { delta,.. } => {
            dbg!(&delta.usage);
            _ = append_chunk_to_file(&format!("{:#?}", delta.usage), "resp.txt");
        }
        StreamEvent::MessageStop => {
            dbg!("stop");
        }
    }
}

fn append_chunk_to_file(sanitised_line: &str, file_path: &str) -> Result<(), AnthropicError> {
    use std::fs::OpenOptions;
    use std::io::Write;
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)
        .unwrap();

    writeln!(file, "{}", sanitised_line).unwrap();

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
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

    #[cfg(test)]
    use super::*; // Import necessary structs/enums and functions from the parent module
    use tokio::fs::File;
    use tokio::io::AsyncBufReadExt; // For reading lines asynchronously
    use tokio::io::BufReader;

    #[tokio::test]
    async fn can_parse() -> Result<(), Box<dyn std::error::Error>> {
        let file_path = "raw_chunk.jsonl"; // Adjust the file path as necessary
        let file = File::open(file_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut current_data = String::new();

        while let Some(line) = lines.next_line().await? {
            if line.starts_with("event: ") {
                continue;
            } else if line.starts_with("data: ") {
                current_data = line["data: ".len()..].to_string();
                parse_event(&current_data)?;
            }
        }

        Ok(())
    }

    fn parse_event(data: &str) -> Result<(), Box<dyn std::error::Error>> {
        // This is where you construct the JSON string and attempt to deserialize it
        // Construct a full event JSON string
        let v = serde_json::to_value(&data).unwrap();
        println!("\n\n{}", v.to_string());

        dbg!("V PARSED");
        let event_json = format!("{}", data);

        // let event: StreamEvent = serde_json::from_str(&event_json)?;
        let event: StreamEvent = match serde_json::from_str(&event_json){
            Ok(v) => v,
            Err(e) => {
                eprintln!("\n\n{e}\n\n\n{data}");
                panic!()
            }
        };

        // Here, you would normally process the event...
        // For this test, we're just checking if parsing succeeds
        println!("Successfully parsed an event: {:?}", event);

        Ok(())
    }
}
