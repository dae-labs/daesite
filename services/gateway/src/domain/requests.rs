use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Message {
    Auth { token: String },
    QrCodeRequest,
    Heartbeat,
    Echo { message: String },
}
