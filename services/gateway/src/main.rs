use std::net::SocketAddr;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() {
    env_logger::init();

    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    gateway::run_server(addr).await;
}
