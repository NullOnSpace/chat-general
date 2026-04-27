use chat_general::{init_logging, ChatServer, Settings};

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

    let server = ChatServer::builder().settings(settings).build().await?;

    server.run().await?;

    Ok(())
}
