use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use futures_util::{StreamExt, SinkExt};
use crate::domain::models::Message;
use crate::domain::ports::WebSocketPort;
use tokio::net::TcpStream;

pub struct WebSocketAdapter {
    websocket: tokio_tungstenite::WebSocketStream<TcpStream>,
}

impl WebSocketAdapter {
    pub fn new(websocket: tokio_tungstenite::WebSocketStream<TcpStream>) -> Self {
        Self { websocket }
    }
}

#[async_trait::async_trait]
impl WebSocketPort for WebSocketAdapter {
    async fn send(&mut self, message: WsMessage) -> Result<(), String> {
        self.websocket.send(message).await.map_err(|e| e.to_string())
    }

    async fn receive(&mut self) -> Option<Result<Message, String>> {
        if let Some(Ok(WsMessage::Binary(data))) = self.websocket.next().await {
            let message: Result<Message, _> = rmp_serde::from_slice(&data);
            Some(message.map_err(|e| e.to_string()))
        } else {
            None
        }
    }
}
