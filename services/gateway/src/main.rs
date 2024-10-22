use gateway::{application, domain::{models::Message, ports::WebSocketPort}};
use log::info;
use tokio::net::TcpListener;
use application::services::{auth_service::AuthService, echo_service::EchoService, heartbeat_service::HeartbeatService};
use gateway::infrastructure::websocket_adapter::WebSocketAdapter;
use tokio_tungstenite::accept_async;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    env_logger::init();

    let auth_service = AuthService::new();
    let echo_service = EchoService;
    let heartbeat_service = HeartbeatService;

    let addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    println!("WebSocket server is running on ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let auth_service = auth_service.clone();
        let echo_service = echo_service.clone();
        let heartbeat_service = heartbeat_service.clone();

        tokio::spawn(async move {
            let websocket = accept_async(stream)
                .await
                .expect("Failed to accept WebSocket connection");

            info!("connection open");

            let mut adapter = WebSocketAdapter::new(websocket);
            let mut missed_heartbeats = 0;

            if let Some(Ok(message)) = adapter.receive().await {
                info!("{:?}", message);

                if auth_service.handle_auth(message, &mut adapter).await.is_ok() {
                    while let Some(Ok(message)) = adapter.receive().await {
                        match message {
                            Message::Echo { .. } => {
                                echo_service.handle_echo(message, &mut adapter).await.unwrap();
                            }
                            Message::Heartbeat => {
                                heartbeat_service
                                    .handle_heartbeat(message, &mut adapter, &mut missed_heartbeats)
                                    .await
                                    .unwrap();
                            }
                            _ => {}
                        }
                    }
                }
            }

            heartbeat_service
                .monitor_heartbeats(&mut adapter, &mut missed_heartbeats, 3)
                .await;
        });
    }
}
