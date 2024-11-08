use thiserror::Error;

#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("Client closed connection")]
    ClientClosedConnection,

    #[error("Missed heartbeats")]
    MissedHeartbeats,

    #[error("Error sending message")]
    SendError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Error during message processing")]
    ProcessingError(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to deserialize message")]
    DeserializationError(#[from] rmp_serde::decode::Error),

    #[error("Failed to deserialize public key")]
    PublicKeyDeserializationError(String),

    #[error("Zlib error")]
    ZlibError(#[from] std::io::Error),
}
