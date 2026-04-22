use anyhow::{Context, Result};
use pallas_codec::minicbor;
use pallas_primitives::conway::{
    TransactionBody, TransactionInput, TransactionOutput, Redeemer, 
    Value as PallasValue, PlutusData, NativeScript, PlutusV3Script,
    DatumOption, PostAlonzoTransactionOutput, MintedTransaction,
    TransactionMetadatum
};
use pallas_crypto::hash::Hasher;
use std::collections::HashMap;

use crate::decoder::cbor::{try_decode_as_plutus_data, bytes_to_hex};
use crate::decoder::types::{
    TxView, InputView, OutputView, DatumView, RedeemerView, 
    AssetView, PlutusNode
};

pub struct TxParser {
    // Cache for resolved datums (hash -> decoded PlutusNode)
    datum_cache: HashMap<String, PlutusNode>,
}

impl TxParser {
    pub fn new() -> Self {
        Self {
            datum_cache: HashMap::new(),
        }
    }

    pub fn parse_transaction(&mut self, tx_cbor: &[u8]) -> Result<TxView> {
        let tx: MintedTransaction = minicbor::decode(tx_cbor)
            .context("Failed to decode transaction CBOR")?;

        let hash = self.compute_tx_hash(tx_cbor)?;
        let inputs = self.parse_inputs(&tx.transaction_body)?;
        let outputs = self.parse_outputs(&tx.transaction_body)?;
        let redeemers = self.parse_redeemers(&tx.transaction_body)?;
        let metadata = self.parse_metadata(&tx)?;

        Ok(TxView {
            hash,
            inputs,
            outputs,
            redeemers,
            metadata,
        })
    }

    fn compute_tx_hash(&self, tx_cbor: &[u8]) -> Result<String> {
        let tx: MintedTransaction = minicbor::decode(tx_cbor)?;
        let tx_body_bytes = minicbor::to_vec(&tx.transaction_body)?;
        
        let hash = Hasher::<256>::hash(&tx_body_bytes);
        Ok(hex::encode(hash))
    }

    fn parse_inputs(&self, tx_body: &TransactionBody) -> Result<Vec<InputView>> {
        let mut inputs = Vec::new();
        
        for input in &tx_body.inputs {
            let input_view = InputView {
                tx_hash: hex::encode(&input.transaction_id),
                index: input.index.into(),
                address: String::new(), // Will be populated from UTxO data
                value: Vec::new(),      // Will be populated from UTxO data
                datum: None,            // Will be resolved from reference inputs
            };
            inputs.push(input_view);
        }
        
        Ok(inputs)
    }

    fn parse_outputs(&mut self, tx_body: &TransactionBody) -> Result<Vec<OutputView>> {
        let mut outputs = Vec::new();
        
        for output in &tx_body.outputs {
            let output_view = self.parse_output(output)?;
            outputs.push(output_view);
        }
        
        Ok(outputs)
    }

    fn parse_output(&mut self, output: &PostAlonzoTransactionOutput) -> Result<OutputView> {
        let address = self.parse_address(&output.address)?;
        let value = self.parse_value(&output.amount)?;
        let datum = self.parse_datum_option(&output.datum_option)?;
        let script_ref = self.parse_script_ref(&output.script_reference)?;
        
        Ok(OutputView {
            address,
            value,
            datum,
            script_ref,
        })
    }

    fn parse_address(&self, address: &pallas_primitives::conway::Address) -> Result<String> {
        // For now, just encode as hex - we can add bech32 encoding later
        Ok(hex::encode(minicbor::to_vec(address)?))
    }

    fn parse_value(&self, value: &PallasValue) -> Result<Vec<AssetView>> {
        let mut assets = Vec::new();
        
        match value {
            PallasValue::Coin(ada) => {
                assets.push(AssetView {
                    policy_id: "ada".to_string(),
                    asset_name: "lovelace".to_string(),
                    amount: *ada,
                });
            }
            PallasValue::MultiAsset(ada, multi_asset) => {
                // Add ADA amount
                assets.push(AssetView {
                    policy_id: "ada".to_string(),
                    asset_name: "lovelace".to_string(),
                    amount: *ada,
                });
                
                // Add other assets
                for (policy_id_bytes, assets_map) in multi_asset.iter() {
                    let policy_id = hex::encode(policy_id_bytes);
                    
                    for (asset_name_bytes, amount) in assets_map.iter() {
                        let asset_name = if asset_name_bytes.is_empty() {
                            String::new()
                        } else {
                            hex::encode(asset_name_bytes)
                        };
                        
                        assets.push(AssetView {
                            policy_id: policy_id.clone(),
                            asset_name,
                            amount: *amount,
                        });
                    }
                }
            }
        }
        
        Ok(assets)
    }

    fn parse_datum_option(&mut self, datum_option: &DatumOption) -> Result<Option<DatumView>> {
        match datum_option {
            DatumOption::Hash(hash) => {
                // Datum hash - will be resolved later via fetcher
                let hash_hex = hex::encode(hash);
                
                // Check cache first
                if let Some(decoded) = self.datum_cache.get(&hash_hex) {
                    return Ok(Some(DatumView {
                        raw_cbor: String::new(), // Will be populated when fetched
                        decoded: decoded.clone(),
                    }));
                }
                
                // Placeholder - actual resolution happens in app layer
                Ok(Some(DatumView {
                    raw_cbor: hash_hex.clone(),
                    decoded: PlutusNode::Text(format!("[Datum Hash: {}]", hash_hex)),
                }))
            }
            DatumOption::Data(data) => {
                // Inline datum - decode immediately
                let data_bytes = minicbor::to_vec(data)?;
                let raw_cbor = bytes_to_hex(&data_bytes);
                let decoded = try_decode_as_plutus_data(&data_bytes);
                
                Ok(Some(DatumView { raw_cbor, decoded }))
            }
        }
    }

    fn parse_script_ref(&self, script_ref: &Option<pallas_primitives::conway::ScriptRef>) -> Result<Option<String>> {
        match script_ref {
            Some(script) => {
                let script_type = match script {
                    pallas_primitives::conway::ScriptRef::NativeScript(_) => "NativeScript",
                    pallas_primitives::conway::ScriptRef::PlutusV1Script(_) => "PlutusV1",
                    pallas_primitives::conway::ScriptRef::PlutusV2Script(_) => "PlutusV2",
                    pallas_primitives::conway::ScriptRef::PlutusV3Script(_) => "PlutusV3",
                };
                Ok(Some(script_type.to_string()))
            }
            None => Ok(None),
        }
    }

    fn parse_redeemers(&self, tx_body: &TransactionBody) -> Result<Vec<RedeemerView>> {
        let mut redeemers = Vec::new();
        
        if let Some(redeemer_set) = &tx_body.redeemers {
            for (index, redeemer) in redeemer_set.iter().enumerate() {
                let redeemer_view = self.parse_redeemer(redeemer, index as u32)?;
                redeemers.push(redeemer_view);
            }
        }
        
        Ok(redeemers)
    }

    fn parse_redeemer(&self, redeemer: &Redeemer, index: u32) -> Result<RedeemerView> {
        let tag = match redeemer.tag {
            pallas_primitives::conway::RedeemerTag::Spend => "Spend".to_string(),
            pallas_primitives::conway::RedeemerTag::Mint => "Mint".to_string(),
            pallas_primitives::conway::RedeemerTag::Cert => "Cert".to_string(),
            pallas_primitives::conway::RedeemerTag::Reward => "Reward".to_string(),
            pallas_primitives::conway::RedeemerTag::Vote => "Vote".to_string(),
            pallas_primitives::conway::RedeemerTag::Propose => "Propose".to_string(),
        };
        
        let data_bytes = minicbor::to_vec(&redeemer.data)?;
        let data = try_decode_as_plutus_data(&data_bytes);
        
        let ex_units = (
            redeemer.ex_units.mem,
            redeemer.ex_units.steps,
        );
        
        Ok(RedeemerView {
            tag,
            index,
            data,
            ex_units,
        })
    }

    fn parse_metadata(&self, tx: &MintedTransaction) -> Result<Option<serde_json::Value>> {
        if let Some(metadata) = &tx.transaction_body.metadata {
            // Convert metadata to JSON for display
            let json = self.metadata_to_json(metadata)?;
            Ok(Some(json))
        } else {
            Ok(None)
        }
    }

    fn metadata_to_json(&self, metadata: &TransactionMetadatum) -> Result<serde_json::Value> {
        match metadata {
            TransactionMetadatum::Map(map) => {
                let mut json_map = serde_json::Map::new();
                for entry in map {
                    if let (TransactionMetadatum::Text(key), value) = entry {
                        json_map.insert(key.clone(), self.metadata_to_json(value)?);
                    } else if let (TransactionMetadatum::Int(key), value) = entry {
                        json_map.insert(key.to_string(), self.metadata_to_json(value)?);
                    }
                }
                Ok(serde_json::Value::Object(json_map))
            }
            TransactionMetadatum::List(items) => {
                let json_items: Result<Vec<_>> = items.iter()
                    .map(|item| self.metadata_to_json(item))
                    .collect();
                Ok(serde_json::Value::Array(json_items?))
            }
            TransactionMetadatum::Int(n) => {
                Ok(serde_json::Value::Number((*n as i64).into()))
            }
            TransactionMetadatum::Bytes(bytes) => {
                Ok(serde_json::Value::String(hex::encode(bytes)))
            }
            TransactionMetadatum::Text(s) => {
                Ok(serde_json::Value::String(s.clone()))
            }
        }
    }

    // Method to resolve datum hashes later
    pub fn resolve_datum_hash(&mut self, hash: &str, data: Vec<u8>) {
        let decoded = try_decode_as_plutus_data(&data);
        self.datum_cache.insert(hash.to_string(), decoded);
    }
}

impl Default for TxParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_parser_creation() {
        let parser = TxParser::new();
        assert!(parser.datum_cache.is_empty());
    }

    #[test]
    fn test_parse_value_ada_only() {
        let parser = TxParser::new();
        let value = PallasValue::Coin(1000000);
        let assets = parser.parse_value(&value).unwrap();
        
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].policy_id, "ada");
        assert_eq!(assets[0].amount, 1000000);
    }
}