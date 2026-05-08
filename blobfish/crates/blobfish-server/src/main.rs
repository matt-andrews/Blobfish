use std::path::PathBuf;
use std::sync::Arc;
use clap::{Parser, Subcommand};
use tracing::{info};
use blobfish_core::models::config::Config;
use blobfish_core::object_service::ObjectService;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(long, default_value = "dev.toml")]
        config: PathBuf,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_init();

    let args = Cli::parse();
    match args.command{
        Commands::Serve{config} => {
            let config_str = &std::fs::read_to_string(config)?;
            let config: Config = toml::from_str(config_str)?;
            run(&config).await?;
        }
    }

    Ok(())
}

async fn run(config: &Config) -> anyhow::Result<()> {
    let meta_config: Config = config.clone();
    let db = tokio::task::spawn_blocking(|| {
        blobfish_meta::init(meta_config)
    }).await?;

    let object_service: ObjectService = ObjectService::new(Arc::new(db?), config.clone());

    info!("Serving blobfish server at {}...", config.node.bind_addr);

    let listener = tokio::net::TcpListener::bind(&config.node.bind_addr).await?;

    axum::serve(
        listener,
        blobfish_api::routes::router(object_service)
    ).await?;

    Ok(())
}

fn tracing_init(){
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let format = std::env::var("RUST_LOG_FORMAT")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let ansi = std::env::var("NO_COLOR").is_err();
    let fmt_layer: Box<dyn tracing_subscriber::Layer<_> + Send + Sync> = match format.as_str() {
        "json" => Box::new(fmt::layer().json()),
        "pretty" => Box::new(fmt::layer().pretty().with_ansi(ansi)),
        _ => Box::new(fmt::layer().compact().with_ansi(ansi)),
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}