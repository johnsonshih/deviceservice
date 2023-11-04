use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    env_logger::init();
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    deviceservice::restapihttpserver::httpserver(addr).await;
}
