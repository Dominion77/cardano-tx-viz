use anyhow::{Context, Result};
use serde::Deserialize;


use crate::fetcher::{FetcherConfig, Network};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub blockfrost: Option<BlockfrostConfig>,
    pub default_network: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BlockfrostConfig {
    pub api_key: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = dirs::config_dir()
            .map(|p| p.join("cardano-tx-viz").join("config.toml"))
            .filter(|p| p.exists());

        if let Some(path) = config_path {
            let content = std::fs::read_to_string(&path)
                .context("Failed to read config file")?;
            toml::from_str(&content).context("Failed to parse config file")
        } else {
            // Return default config if no file exists
            Ok(Config {
                blockfrost: None,
                default_network: Some("mainnet".to_string()),
            })
        }
    }

    pub fn get_fetcher_config(&self, network: Network) -> FetcherConfig {
        // Try Blockfrost first if API key is available
        if let Some(bf_config) = &self.blockfrost {
            FetcherConfig::Blockfrost {
                api_key: bf_config.api_key.clone(),
                network,
            }
        } else {
            // Fallback to Koios
            FetcherConfig::Koios { network }
        }
    }
}