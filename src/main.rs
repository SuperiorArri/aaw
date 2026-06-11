use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aaw=info,tower_http=info".into()),
        )
        .init();

    info!("** Arri's Audio Workstation **");

    return Ok(())
}
