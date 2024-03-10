use serde::Serialize;
use crate::Message;

#[derive(Debug, Serialize)]
pub struct MessageRequest<'a> {
    pub model: &'a str,
    pub messages: Vec<Message<'a>>,
    pub max_tokens: u32,
    pub system: Option<&'a str>,
    pub metadata: Option<Metadata<'a>>,
    pub stop_sequences: Option<Vec<&'a str>>,
    pub stream: bool,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct Metadata<'a> {
    pub user_id: Option<&'a str>,
}

/// Example usage:
///
/// ```rust
/// use mechanise::requests::MessageRequestBuilder;
///
/// let request = MessageRequestBuilder::new()
///     .model("my-model")
///     .messages(vec![Message::new()])
///     .max_tokens(100)
///     .system("You are a helpful assistant.")
///     //  .metadata(Metadata { user_id: Some("user123") }) // See docs https://docs.anthropic.com/claude/reference/messages_post
///     .stop_sequences(vec!["###"])
///     .stream(true)
///     .temperature(0.7)
///     .top_p(0.9)
///     .top_k(40)
///     .build();
/// ```
pub struct MessageRequestBuilder<'a> {
    model: Option<&'a str>,
    messages: Option<Vec<Message<'a>>>,
    max_tokens: Option<u32>,
    system: Option<&'a str>,
    metadata: Option<Metadata<'a>>,
    stop_sequences: Option<Vec<&'a str>>,
    stream: Option<bool>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    top_k: Option<u32>,
}

impl<'a> MessageRequestBuilder<'a> {
    pub fn new() -> Self {
        MessageRequestBuilder {
            model: None,
            messages: None,
            max_tokens: None,
            system: None,
            metadata: None,
            stop_sequences: None,
            stream: None,
            temperature: None,
            top_p: None,
            top_k: None,
        }
    }

    pub fn build(self) -> MessageRequest<'a> {
        MessageRequest {
            model: self.model.unwrap_or(""),
            messages: self.messages.unwrap_or_default(),
            max_tokens: self.max_tokens.unwrap_or(0),
            system: self.system,
            metadata: self.metadata,
            stop_sequences: self.stop_sequences,
            stream: self.stream.unwrap_or(false),
            temperature: self.temperature,
            top_p: self.top_p,
            top_k: self.top_k,
        }
    }

    pub fn model(mut self, model: &'a str) -> Self {
        self.model = Some(model);
        self
    }

    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn messages(mut self, messages: Vec<Message<'a>>) -> Self {
        self.messages = Some(messages);
        self
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    pub fn system(mut self, system: &'a str) -> Self {
        self.system = Some(system);
        self
    }

    pub fn metadata(mut self, metadata: Metadata<'a>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn stop_sequences(mut self, stop_sequences: Vec<&'a str>) -> Self {
        self.stop_sequences = Some(stop_sequences);
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }
}