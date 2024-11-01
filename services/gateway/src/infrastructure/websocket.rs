use futures_util::{stream::StreamExt, SinkExt};
use log::{debug, error, trace};
use qrcode::{EcLevel, QrCode, Version};
use tokio::net::TcpStream;
use tokio::time::{self, Duration, Interval};
use tokio_tungstenite::{tungstenite::protocol::Message as WsMessage, WebSocketStream};
use crate::domain::requests::Message;
use image::Luma;

use super::error::WebSocketError;

pub struct Client {
    id: i64,
    socket: WebSocketStream<TcpStream>,
    heartbeat_interval: Interval,
    missed_heartbeats: usize,
    closed: bool,
}

impl Client {
    pub fn new(id: i64, socket: WebSocketStream<TcpStream>) -> Self {
        Self {
            id,
            socket,
            heartbeat_interval: time::interval(Duration::from_secs(20)),
            missed_heartbeats: 0,
            closed: false,
        }
    }

    async fn handle_incoming_message(&mut self, ws_msg: WsMessage) -> Result<(), WebSocketError> {
        if ws_msg.is_close() {
            self.closed = true;
            debug!("Connection {} closed by client.", self.id);
            return Err(WebSocketError::ClientClosedConnection);
        }

        if let WsMessage::Binary(bytes) = ws_msg {
            let message: Message = rmp_serde::from_slice(&bytes)?;
            self.handle_message(message).await?;
        }

        Ok(())
    }

    async fn handle_message(&mut self, msg: Message) -> Result<(), WebSocketError> {
        debug!("Received message from: {}", self.id);
        match msg {
            Message::Auth { token } => {
                debug!("Received auth token: {}", token);
                self.socket.send(WsMessage::Text(self.id.to_string())).await?;
            }
            Message::QrCodeRequest => {
                trace!("Received QR Code request from client {}", self.id);
                let session_token = "unique_token";

                let code = QrCode::with_version(session_token.as_bytes(), Version::Normal(10), EcLevel::M).unwrap();
                let image = code.render::<Luma<u8>>().build();

                self.socket.send(WsMessage::Binary(image.to_vec())).await?;
            }
            Message::Heartbeat => {
                trace!("Received heartbeat from client {}", self.id);
                self.missed_heartbeats = 0;
                self.heartbeat_interval.reset();
            }
            Message::Echo { message } => {
                debug!("Echoing message to client {}: {}", self.id, message);
                self.socket.send(WsMessage::Text(message)).await?;
            }
        }
        Ok(())
    }

    async fn handle_heartbeat(&mut self) -> Result<(), WebSocketError> {
        self.missed_heartbeats += 1;

        if self.missed_heartbeats >= 3 {
            debug!("Closing connection {} due to missed heartbeats.", self.id);
            self.closed = true; // Set closed state early to prevent further processing
            return Err(WebSocketError::MissedHeartbeats);
        }

        debug!("Client {} missed heartbeat {}/3", self.id, self.missed_heartbeats);
        Ok(())
    }

    pub async fn run(&mut self) {
        self.heartbeat_interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                Some(Ok(msg)) = self.socket.next() => {
                    if let Err(e) = self.handle_incoming_message(msg).await {
                        self.handle_error(e).await; // Extracted error handling
                        break;
                    }
                },
                _ = self.heartbeat_interval.tick() => {
                    if let Err(e) = self.handle_heartbeat().await {
                        self.handle_error(e).await; // Extracted error handling
                        break;
                    }
                },
            }
        }

        self.close_socket().await; // Extracted close logic
    }

    async fn handle_error(&mut self, error: WebSocketError) {
        match error {
            WebSocketError::ClientClosedConnection | WebSocketError::MissedHeartbeats => {
                self.closed = true;
            },
            _ => {
                error!("Error handling message for client {}: {:?}", self.id, error);
            },
        }
    }

    async fn close_socket(&mut self) {
        if !self.closed {
            if let Err(e) = self.socket.close(None).await {
                error!("Error closing WebSocket for client {}: {:?}", self.id, e);
            } else {
                debug!("Closed connection {} gracefully.", self.id);
            }
        }
    }
}
