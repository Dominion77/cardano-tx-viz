use anyhow::Result;
use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use cardano_tx_viz::app::App;
use cardano_tx_viz::config::Config;
use cardano_tx_viz::fetcher::Network;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Transaction hash to inspect
    #[arg(short = 't', long)]
    hash: Option<String>,

    /// Cardano network (mainnet, preprod, preview)
    #[arg(short, long, default_value = "mainnet")]
    network: String,

    /// Enable debug logging
    #[arg(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Load configuration
    let config = Config::load()?;
    let network = args.network.parse::<Network>()?;
    let fetcher_config = config.get_fetcher_config(network.clone());

    // Create app
    let mut app = App::new(network, fetcher_config);

    // If hash provided, fetch immediately
    if let Some(hash) = args.hash {
        app.start_fetch(hash);
    }

    // Start TUI
    cardano_tx_viz::ui::run(app).await?;

    Ok(())
}
