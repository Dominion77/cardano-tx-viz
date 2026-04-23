use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use super::{Network, RawTx, TxFetcher};

#[derive(Debug, Clone)]
pub struct KoiosFetcher {
    client: Client,
    network: Network,
}

impl KoiosFetcher {
    pub fn new(network: Network) -> Self {
        Self {
            client: Client::new(),
            network,
        }
    }

    async fn post<T: for<'de> Deserialize<'de>>(&self, endpoint: &str, body: serde_json::Value) -> Result<T> {
        let url = format!("{}{}", self.network.koios_base_url(), endpoint);
        let response = self
            .client
            .post(&url)
            .json(&body)
            .header("Accept", "application/json")
            .send()
            .await
            .context(format!("Failed to connect to Koios API at {}", url))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            if status.as_u16() == 404 {
                return Err(anyhow!("Transaction not found on {} network", self.network.to_string()));
            }
            
            return Err(anyhow!(
                "Koios API returned error {} for {}: {}",
                status.as_u16(),
                url,
                error_text
            ));
        }

        let koios_response: Vec<T> = response
            .json()
            .await
            .context("Failed to parse Koios API response (invalid JSON)")?;
        
        koios_response
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Transaction not found (empty response from Koios)"))
    }
}

#[async_trait]
impl TxFetcher for KoiosFetcher {
    async fn fetch(&self, hash: &str) -> Result<RawTx> {
        #[derive(Deserialize)]
        struct TxCborResponse {
            tx_hash: String,
            cbor: String,
        }

        let body = json!({
            "_tx_hashes": [hash]
        });

        let response: TxCborResponse = self.post("/tx_cbor", body)
            .await
            .context(format!("Failed to fetch transaction {} from Koios", hash))?;
        
        let cbor = hex::decode(&response.cbor)
            .context("Failed to decode CBOR hex from Koios response")?;

        Ok(RawTx {
            hash: response.tx_hash,
            cbor,
        })
    }

    async fn fetch_datum(&self, datum_hash: &str) -> Result<Vec<u8>> {
        #[derive(Deserialize)]
        struct DatumInfoResponse {
            bytes: String,
        }

        let body = json!({
            "_datum_hashes": [datum_hash]
        });

        let response: DatumInfoResponse = self.post("/datum_info", body).await?;
        
        hex::decode(&response.bytes)
            .context("Failed to decode datum bytes from Koios")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_base_urls() {
        assert_eq!(
            Network::Mainnet.koios_base_url(),
            "https://api.koios.rest/api/v1"
        );
        assert_eq!(
            Network::Preprod.koios_base_url(),
            "https://preprod.koios.rest/api/v1"
        );
        assert_eq!(
            Network::Preview.koios_base_url(),
            "https://preview.koios.rest/api/v1"
        );
    }
}