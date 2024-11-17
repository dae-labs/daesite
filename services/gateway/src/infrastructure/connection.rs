use base64::prelude::*;
use futures::SinkExt;
use rand::rngs::OsRng;
use rand::RngCore;
use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::protocol::CloseFrame;
use super::compression::{compress_data, decompress_data};
use crate::domain::error::GatewayError;
use crate::domain::messages::Message;
use crate::domain::models::id::ID;
use futures_util::stream::StreamExt;
use log::{debug, error, trace};
use tokio::net::TcpStream;
use tokio::time::{self, Duration, Interval};
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use tokio_tungstenite::WebSocketStream;
use x25519_dalek::{PublicKey, EphemeralSecret};

pub struct Connection {
    id: ID,
    socket: WebSocketStream<TcpStream>,
    heartbeat_interval: Interval,
    missed_heartbeats: usize,
    closed: bool,
}

impl Connection {
    pub fn new(id: i64, socket: WebSocketStream<TcpStream>) -> Self {
        Self {
            id,
            socket,
            heartbeat_interval: time::interval(Duration::from_secs(20)),
            missed_heartbeats: 0,
            closed: false,
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
            Message::Auth { token } => debug!("Received auth token: {}", token),
            Message::QrCodeRequest => trace!("Received QR Code request from client {}", self.id),
            Message::Heartbeat => self.reset_heartbeat(),
            Message::Echo { message } => debug!("Echoing message to client {}: {}", self.id, message),
            Message::PublicKey { encoded_public_key } => self.handle_public_key(encoded_public_key).await?,
            _ => debug!("Unknown message from client {}", self.id),
        }

        Ok(())
    }

    async fn handle_public_key(&mut self, encoded_public_key: String) -> Result<(), GatewayError> {
        let public_key = Self::decode_public_key(&encoded_public_key, self.id)?;
        let shared_secret = EphemeralSecret::random_from_rng(&mut OsRng).diffie_hellman(&public_key);

        let mut nonce = vec![0u8; 32];
        OsRng.fill_bytes(&mut nonce);

        let encrypted_nonce: Vec<u8> = nonce.iter()
            .zip(shared_secret.as_bytes().iter().cycle())
            .map(|(n, s)| n ^ s)
            .collect();

        self.send_message(Message::Nonce { encrypted_nonce: BASE64_STANDARD.encode(encrypted_nonce) }).await;
        Ok(())
    }

    fn decode_public_key(encoded_public_key: &str, client_id: i64) -> Result<PublicKey, GatewayError> {
        let decoded_key = BASE64_STANDARD.decode(encoded_public_key)
            .map_err(|e| GatewayError::PublicKeyDeserializationError(format!("Failed to decode public key for client {}: {:?}", client_id, e)))?;

        if decoded_key.len() == 32 {
            let key_array: [u8; 32] = decoded_key[..].try_into().unwrap();
            Ok(PublicKey::from(key_array))
        } else {
            Err(GatewayError::PublicKeyDeserializationError(format!(
                "Invalid public key length for client {}: expected 32 bytes, got {}",
                client_id, decoded_key.len()
            )))
        }
    }

    async fn send_message(&mut self, message: Message) {
        if let Ok(serialized) = rmp_serde::to_vec(&message) {
            if let Ok(compressed) = compress_data(&serialized) {
                if let Err(e) = self.socket.send(WsMessage::Binary(compressed)).await {
                    error!("Failed to send message to client {}: {:?}", self.id, e);
                }
            } else {
                error!("Failed to compress message for client {}", self.id);
            }
        } else {
            error!("Failed to serialize message for client {}", self.id);
        }
    }

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

        debug!("Client {} missed heartbeat {}/3", self.id, self.missed_heartbeats);
        Ok(())
    }

    pub async fn run(&mut self) {
        self.heartbeat_interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

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
        if matches!(error, GatewayError::ClientClosedConnection | GatewayError::MissedHeartbeats) {
            self.closed = true;
        } else {
            error!("Error handling message for client {}: {:?}", self.id, error);
        }
    }

    async fn close(&mut self, reason: &str) -> Result<(), GatewayError> {
        debug!("Closing connection {}: {}", self.id, reason);
        self.closed = true;
        self.close_socket().await;
        Ok(())
    }

    async fn close_socket(&mut self) {
        if !self.closed {
            if let Err(e) = self.socket.close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: "Closing connection gracefully".into(),
            })).await {
                error!("Error closing WebSocket for client {}: {:?}", self.id, e);
            } else {
                debug!("Closed connection {} gracefully.", self.id);
            }
        }
    }
}
