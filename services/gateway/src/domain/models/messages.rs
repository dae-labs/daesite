use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    Auth { token: String },
    Echo { content: String },
    Heartbeat,
}
