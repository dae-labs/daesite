pub mod application;
pub mod domain;
pub mod infrastructure;

use id::Generator;
use infrastructure::websocket::Client;
use tokio_tungstenite::accept_async;
use std::net::SocketAddr;

pub async fn run_server(addr: SocketAddr) {
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let mut id_generator = Generator::default();
    println!("WebSocket server running on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream).await.unwrap();
        let mut client = Client::new(id_generator.generate(), ws_stream);
        tokio::spawn(async move {
            client.run().await;
        });
    }
}
