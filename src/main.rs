use chat_general::{Settings, ChatServer, init_logging};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    init_logging();

    let settings = Settings::new()?;
    
    tracing::info!(
        server.host = %settings.server.host,
        server.port = %settings.server.port,
        "Starting Chat-General server"
    );

    let server = ChatServer::builder()
        .settings(settings)
        .build()?;

    server.run().await?;

    Ok(())
}