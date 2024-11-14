pub mod domain;
pub mod infrastructure;

use id::Generator;
use infrastructure::connection::Connection;
use std::net::SocketAddr;
use tokio_tungstenite::accept_async;

pub async fn run_server(addr: SocketAddr) {
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let mut id_generator = Generator::default();
    println!("WebSocket server running on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = match accept_async(stream).await {
            Ok(ws) => ws,
            Err(e) => {
                eprintln!("Failed to complete WebSocket handshake: {}", e);
                continue; // skip this connection and continue to accept new ones
            }
        };

        let mut client = Connection::new(id_generator.generate(), ws_stream);
        tokio::spawn(async move {
            client.run().await;
        });
    }
}
