use tungstenite::Message as WsMessage;
use crate::domain::models::Message;
use crate::domain::ports::WebSocketPort;

#[derive(Clone)]
pub struct EchoService;

impl EchoService {
    pub async fn handle_echo<T: WebSocketPort>(
        &self,
        message: Message,
        websocket: &mut T,
    ) -> Result<(), String> {
        if let Message::Echo { content } = message {
            websocket
                .send(WsMessage::Text(content.clone()))
                .await?;
            Ok(())
        } else {
            Err("Invalid message type for echo".to_string())
        }
    }
}
