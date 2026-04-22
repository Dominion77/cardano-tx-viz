use anyhow::Result;
use async_trait::async_trait;

use super::{RawTx, TxFetcher};

#[derive(Debug, Clone)]
pub struct MockFetcher {
    pub tx_responses: std::collections::HashMap<String, Result<RawTx>>,
    pub datum_responses: std::collections::HashMap<String, Result<Vec<u8>>>,
}

impl MockFetcher {
    pub fn new() -> Self {
        Self {
            tx_responses: std::collections::HashMap::new(),
            datum_responses: std::collections::HashMap::new(),
        }
    }

    pub fn with_tx(mut self, hash: &str, response: Result<RawTx>) -> Self {
        self.tx_responses.insert(hash.to_string(), response);
        self
    }

    pub fn with_datum(mut self, hash: &str, response: Result<Vec<u8>>) -> Self {
        self.datum_responses.insert(hash.to_string(), response);
        self
    }
}

impl Default for MockFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TxFetcher for MockFetcher {
    async fn fetch(&self, hash: &str) -> Result<RawTx> {
        match self.tx_responses.get(hash) {
            Some(response) => match response {
                Ok(tx) => Ok(tx.clone()),
                Err(e) => Err(anyhow::anyhow!("{}", e)),
            },
            None => Err(anyhow::anyhow!("Mock transaction not found")),
        }
    }

    async fn fetch_datum(&self, datum_hash: &str) -> Result<Vec<u8>> {
        match self.datum_responses.get(datum_hash) {
            Some(response) => match response {
                Ok(data) => Ok(data.clone()),
                Err(e) => Err(anyhow::anyhow!("{}", e)),
            },
            None => Err(anyhow::anyhow!("Mock datum not found")),
        }
    }
}