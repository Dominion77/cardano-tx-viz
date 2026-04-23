pub mod blockfrost;
pub mod koios;
pub mod mock;

use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct RawTx {
    pub hash: String,
    pub cbor: Vec<u8>,
}

#[async_trait]
pub trait TxFetcher: Send + Sync {
    async fn fetch(&self, hash: &str) -> Result<RawTx>;
    async fn fetch_datum(&self, datum_hash: &str) -> Result<Vec<u8>>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Network {
    Mainnet,
    Preprod,
    Preview,
}

impl Network {
    pub fn blockfrost_base_url(&self) -> &'static str {
        match self {
            Network::Mainnet => "https://cardano-mainnet.blockfrost.io/api/v0",
            Network::Preprod => "https://cardano-preprod.blockfrost.io/api/v0",
            Network::Preview => "https://cardano-preview.blockfrost.io/api/v0",
        }
    }

    pub fn koios_base_url(&self) -> &'static str {
        match self {
            Network::Mainnet => "https://api.koios.rest/api/v1",
            Network::Preprod => "https://preprod.koios.rest/api/v1",
            Network::Preview => "https://preview.koios.rest/api/v1",
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            Network::Mainnet => "mainnet",
            Network::Preprod => "preprod",
            Network::Preview => "preview",
        }
    }
}

impl std::str::FromStr for Network {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(Network::Mainnet),
            "preprod" => Ok(Network::Preprod),
            "preview" => Ok(Network::Preview),
            _ => Err(anyhow::anyhow!("Invalid network: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FetcherConfig {
    Blockfrost { api_key: String, network: Network },
    Koios { network: Network },
}

impl FetcherConfig {
    pub fn create_fetcher(self) -> Box<dyn TxFetcher> {
        match self {
            FetcherConfig::Blockfrost { api_key, network } => {
                Box::new(blockfrost::BlockfrostFetcher::new(api_key, network))
            }
            FetcherConfig::Koios { network } => {
                Box::new(koios::KoiosFetcher::new(network))
            }
        }
    }
}