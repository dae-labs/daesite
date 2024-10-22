use tungstenite::Message as WsMessage;
use crate::domain::models::Message;

use crate::domain::ports::WebSocketPort;

#[derive(Clone)]
pub struct AuthService {
    valid_tokens: Vec<String>,
}

impl AuthService {
    pub fn new() -> Self {
        Self {
            valid_tokens: vec!["valid_token".to_string()],
        }
    }

    pub fn validate_token(&self, token: &str) -> bool {
        self.valid_tokens.contains(&token.to_string())
    }

    pub async fn handle_auth<T: WebSocketPort>(
        &self,
        message: Message,
        websocket: &mut T,
    ) -> Result<(), String> {
        if let Message::Auth { token } = message {
            if self.validate_token(&token) {
                websocket
                    .send(WsMessage::Text("Authenticated".into()))
                    .await?;
                Ok(())
            } else {
                websocket
                    .send(WsMessage::Text("Unauthorized".into()))
                    .await?;
                Err("Unauthorized".to_string())
            }
        } else {
            Err("Invalid message type for auth".to_string())
        }
    }
}
