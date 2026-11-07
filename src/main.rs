use std::net::SocketAddr;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // NFR-7: level from RUST_LOG, `info` by default.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    // Railway and the Dockerfile inject PORT; 8080 matches the container default.
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "listening");
    axum::serve(listener, calculator_api::app()).await
}
