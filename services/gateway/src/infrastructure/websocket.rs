use super::compression::decompress_data;
use crate::domain::error::GatewayError;
use crate::domain::requests::Message;
use futures_util::stream::StreamExt;
use log::{debug, error, trace};
use tokio::net::TcpStream;
use tokio::time::{self, Duration, Interval};
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use tokio_tungstenite::WebSocketStream;
use x25519_dalek::PublicKey;

pub struct Connection {
    id: i64,
    socket: WebSocketStream<TcpStream>,
    heartbeat_interval: Interval,
    missed_heartbeats: usize,
    closed: bool,
    public_key: Option<PublicKey>,
}

impl Connection {
    pub fn new(id: i64, socket: WebSocketStream<TcpStream>) -> Self {
        Self {
            id,
            socket,
            heartbeat_interval: time::interval(Duration::from_secs(20)),
            missed_heartbeats: 0,
            closed: false,
            public_key: None,
        }
    }

    async fn handle_incoming_message(&mut self, ws_msg: WsMessage) -> Result<(), GatewayError> {
        if ws_msg.is_close() {
            self.close("Connection closed by client").await?;
            return Err(GatewayError::ClientClosedConnection);
        }

        if let WsMessage::Binary(bytes) = ws_msg {
            let decompressed_data = decompress_data(&bytes)?;
            let message: Message = rmp_serde::from_slice(&decompressed_data)?;
            self.handle_message(message).await?;
        }

        Ok(())
    }

    async fn handle_message(&mut self, msg: Message) -> Result<(), GatewayError> {
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
                debug!("PublicKey message from client {}", self.id);

                if encoded_public_key.len() == 32 {
                    let key_array: [u8; 32] = encoded_public_key[..].try_into().unwrap();
                    self.public_key = Some(PublicKey::from(key_array));
                    debug!("Stored x25519 public key for client {}.", self.id);
                } else {
                    let error_message = format!(
                        "Invalid public key length for client {}: expected 32 bytes, got {}",
                        self.id, encoded_public_key.len()
                    );
                    error!("{}", error_message);
                    return Err(GatewayError::PublicKeyDeserializationError(error_message));
                }
            },
            Message::Nonce { encrypted_nonce } => {
                debug!("Nonce message from client {}", self.id);
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

    async fn handle_heartbeat(&mut self) -> Result<(), GatewayError> {
        self.missed_heartbeats += 1;
        if self.missed_heartbeats >= 3 {
            self.close("Missed too many heartbeats").await?;
            return Err(GatewayError::MissedHeartbeats);
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

    async fn handle_error(&mut self, error: GatewayError) {
        if matches!(
            error,
            GatewayError::ClientClosedConnection | GatewayError::MissedHeartbeats
        ) {
            self.closed = true;
        } else {
            error!("Error handling message for client {}: {:?}", self.id, error);
        }
    }

    async fn close(&mut self, reason: &str) -> Result<(), GatewayError> {
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
