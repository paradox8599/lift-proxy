use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct ChatBody {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: Option<bool>,
    pub max_tokens: Option<i32>,
    pub temperature: Option<f32>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Delta {
    pub role: Option<String>,
    pub content: String,
    pub finish_reason: Option<String>,
    pub match_stop: Option<i32>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Choice {
    pub index: Option<i32>,
    pub delta: Delta,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct StreamChunk {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<i32>,
    pub model: Option<String>,
    pub choices: Vec<Choice>,
}


