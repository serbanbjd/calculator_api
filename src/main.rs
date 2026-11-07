use std::net::SocketAddr;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Railway and the Dockerfile inject PORT; 8080 matches the container default.
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, calculator_api::app()).await
}
