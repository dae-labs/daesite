use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use crate::domain::models::Message;

#[async_trait::async_trait]
pub trait WebSocketPort {
    async fn send(&mut self, message: WsMessage) -> Result<(), String>;
    async fn receive(&mut self) -> Option<Result<Message, String>>;
}
