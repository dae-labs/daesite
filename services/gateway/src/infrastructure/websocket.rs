use futures_util::{stream::StreamExt, SinkExt};
use log::{debug, error, trace};
use tokio::net::TcpStream;
use tokio::time::{self, Duration};
use tokio_tungstenite::{tungstenite::protocol::Message as WsMessage, WebSocketStream};
use crate::domain::requests::Message;

pub struct Client {
    id: i64,
    socket: WebSocketStream<TcpStream>,
    missed_heartbeats: usize,
    closed: bool,
}

impl Client {
    pub fn new(id: i64, socket: WebSocketStream<TcpStream>) -> Self {
        Self {
            id,
            socket,
            missed_heartbeats: 0,
            closed: false,
        }
    }

    async fn handle_message(&mut self, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
        match msg {
            Message::Auth { token } => {
                debug!("Received auth token: {}", token);
                // Add authentication logic here
            }
            Message::QrCodeRequest => {
                trace!("Received QR Code request from client {}", self.id);
                // Handle QR code generation here
            }
            Message::Heartbeat => {
                trace!("Received heartbeat from client {}", self.id);
                self.missed_heartbeats = 0;
            }
            Message::Echo { message } => {
                trace!("Echoing message to client {}: {}", self.id, message);
                self.socket.send(WsMessage::Text(message)).await?;
            }
        }
        Ok(())
    }

    pub async fn run(&mut self) {
        let mut heartbeat_interval = time::interval(Duration::from_secs(10));
        heartbeat_interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                Some(Ok(msg)) = self.socket.next() => {
                    if msg.is_close() {
                        self.closed = true;
                        trace!("Connection {} closed by client.", self.id);
                        break;
                    }
                    if let WsMessage::Text(text) = msg {
                        match rmp_serde::from_slice(text.as_bytes()) {
                            Ok(message) => {
                                if let Err(e) = self.handle_message(message).await {
                                    error!("Error handling message for client {}: {:?}", self.id, e);
                                }
                            }
                            Err(e) => error!("Failed to deserialize message for client {}: {:?}", self.id, e),
                        }
                    }
                },
                _ = heartbeat_interval.tick() => {
                    if self.missed_heartbeats >= 3 {
                        debug!("Closing connection {} due to missed heartbeats.", self.id);
                        break;
                    }
                    self.missed_heartbeats += 1;
                    debug!("Client {} missed heartbeat {}/3", self.id, self.missed_heartbeats);
                },
            }
        }

        if !self.closed {
            if let Err(e) = self.socket.close(None).await {
                error!("Error closing WebSocket for client {}: {:?}", self.id, e);
            } else {
                trace!("Closed connection {} gracefully.", self.id);
            }
        }
    }
}
