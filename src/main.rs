use chat_general::{Settings, ChatServer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chat_general=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let settings = Settings::new()?;
    
    tracing::info!(
        "Starting Chat-General server on {}:{}",
        settings.server.host,
        settings.server.port
    );

    let server = ChatServer::builder()
        .settings(settings)
        .build()?;

    server.run().await?;

    Ok(())
}
