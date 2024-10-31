use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebSocketError {
    #[error("Failed to deserialize message")]
    DeserializationError(#[from] rmp_serde::decode::Error),

    #[error("Client closed the connection")]
    ClientClosedConnection,

    #[error("Missed heartbeats threshold reached")]
    MissedHeartbeats,

    #[error("WebSocket send error: {0}")]
    SendError(#[from] tokio_tungstenite::tungstenite::Error),
}
