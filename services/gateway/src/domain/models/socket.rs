use super::messages::Message;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;

#[async_trait::async_trait]
pub trait SocketPort: Send + Sync {
    async fn send(&mut self, message: WsMessage) -> Result<(), String>;
    async fn receive(&mut self) -> Option<Result<Message, String>>;
    async fn close(&mut self) -> Result<(), String>;
}
