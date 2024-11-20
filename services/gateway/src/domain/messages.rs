use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Message {
    Auth { token: String },
    QrCodeRequest,
    Heartbeat,
    Echo { message: String },
    PublicKey { encoded_public_key: String },
    QrCode { encoded_qrcode: String },
    Nonce { encrypted_nonce: String },
    NonceProof { nonce: String },
}
