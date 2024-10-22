use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    Auth { token: String },
    Echo { content: String },
    Heartbeat,
}

#[derive(Debug)]
pub struct User {
    pub token: String,
}

impl User {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}
