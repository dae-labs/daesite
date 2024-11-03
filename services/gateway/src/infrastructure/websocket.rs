use super::compression::decompress_data;
use crate::domain::error::WebSocketError;
use crate::domain::requests::Message;
use futures_util::stream::StreamExt;
use log::{debug, error, trace};
use tokio::net::TcpStream;
use tokio::time::{self, Duration, Interval};
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use tokio_tungstenite::WebSocketStream;

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
            self.close("Connection closed by client").await?;
            return Err(WebSocketError::ClientClosedConnection);
        }

        if let WsMessage::Binary(bytes) = ws_msg {
            let decompressed_data = decompress_data(&bytes)?;
            let message: Message = rmp_serde::from_slice(&decompressed_data)?;
            self.handle_message(message).await?;
        }

        Ok(())
    }

    async fn handle_message(&mut self, msg: Message) -> Result<(), WebSocketError> {
        debug!("Received message from client {}: {:?}", self.id, msg);
        match msg {
            Message::Auth { token } => {
                debug!("Received auth token: {}", token);
            }
            Message::QrCodeRequest => {
                trace!("Received QR Code request from client {}", self.id);
            }
            Message::Heartbeat => {
                trace!("Received heartbeat from client {}", self.id);
                self.reset_heartbeat();
            }
            Message::Echo { message } => {
                debug!("Echoing message to client {}: {}", self.id, message);
            }
            Message::PublicKey { encoded_public_key } => {

            },
            Message::Nonce { encrypted_nonce } => {

            },
        }
        Ok(())
    }

    //fn generate_qr_code(&self, session_token: &str) -> Result<Vec<u8>, WebSocketError> {
    //    let code = QrCode::with_version(session_token.as_bytes(), Version::Normal(10), EcLevel::M)?;
    //    let image = code.render::<Luma<u8>>().build();
    //    Ok(image.to_vec())
    //}

    fn reset_heartbeat(&mut self) {
        self.missed_heartbeats = 0;
        self.heartbeat_interval.reset();
    }

    async fn handle_heartbeat(&mut self) -> Result<(), WebSocketError> {
        self.missed_heartbeats += 1;
        if self.missed_heartbeats >= 3 {
            self.close("Missed too many heartbeats").await?;
            return Err(WebSocketError::MissedHeartbeats);
        }

        debug!(
            "Client {} missed heartbeat {}/3",
            self.id, self.missed_heartbeats
        );
        Ok(())
    }

    pub async fn run(&mut self) {
        self.heartbeat_interval
            .set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                Some(Ok(msg)) = self.socket.next() => {
                    if let Err(e) = self.handle_incoming_message(msg).await {
                        self.handle_error(e).await;
                        break;
                    }
                },
                _ = self.heartbeat_interval.tick() => {
                    if let Err(e) = self.handle_heartbeat().await {
                        self.handle_error(e).await;
                        break;
                    }
                },
            }
        }

        self.close_socket().await;
    }

    async fn handle_error(&mut self, error: WebSocketError) {
        if matches!(
            error,
            WebSocketError::ClientClosedConnection | WebSocketError::MissedHeartbeats
        ) {
            self.closed = true;
        } else {
            error!("Error handling message for client {}: {:?}", self.id, error);
        }
    }

    async fn close(&mut self, reason: &str) -> Result<(), WebSocketError> {
        debug!("Closing connection {}: {}", self.id, reason);
        self.closed = true;
        Ok(self.close_socket().await)
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
