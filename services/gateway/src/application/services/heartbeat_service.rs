use tungstenite::Message as WsMessage;
use crate::domain::models::Message;
use crate::domain::ports::WebSocketPort;
use tokio::time::{self, Duration};
use log::info;

#[derive(Clone)]
pub struct HeartbeatService;

impl HeartbeatService {
    pub async fn handle_heartbeat<T: WebSocketPort>(
        &self,
        message: Message,
        websocket: &mut T,
        missed_heartbeats: &mut usize,
    ) -> Result<(), String> {
        if let Message::Heartbeat = message {
            *missed_heartbeats = 0;
            websocket
                .send(WsMessage::Text("Heartbeat".into()))
                .await?;
            Ok(())
        } else {
            Err("Invalid message type for heartbeat".to_string())
        }
    }

    pub async fn monitor_heartbeats<T: WebSocketPort>(
        &self,
        websocket: &mut T,
        missed_heartbeats: &mut usize,
        max_missed_heartbeats: usize,
    ) {
        let interval = Duration::from_secs(5);
        loop {
            time::sleep(interval).await;
            info!("heartbeat missed");
            *missed_heartbeats += 1;

            if *missed_heartbeats >= max_missed_heartbeats {
                let _ = websocket.send(WsMessage::Close(None)).await;
                info!("connection closed");
                break;
            }
        }
    }
}
