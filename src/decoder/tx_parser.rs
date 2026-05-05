use anyhow::{Context, Result};
use pallas_codec::minicbor;
use pallas_crypto::hash::Hasher;
use pallas_primitives::conway::{
    AuxiliaryData, DatumOption, Metadata, Metadatum, PostAlonzoTransactionOutput,
    PseudoTransactionOutput, TransactionBody, Tx, Value as PallasValue, WitnessSet,
};
use std::collections::HashMap;

use crate::decoder::cbor::{bytes_to_hex, try_decode_as_plutus_data};
use crate::decoder::types::{
    AssetView, DatumView, InputView, OutputView, PlutusNode, RedeemerView, TxView,
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
        let tx: Tx = minicbor::decode(tx_cbor).context("Failed to decode transaction CBOR")?;

        let hash = self.compute_tx_hash(tx_cbor)?;
        let inputs = self.parse_inputs(&tx.transaction_body)?;
        let outputs = self.parse_outputs(&tx.transaction_body)?;
        let redeemers = self.parse_redeemers(&tx.transaction_witness_set)?;
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
        let tx: Tx = minicbor::decode(tx_cbor)?;
        let tx_body_bytes = minicbor::to_vec(&tx.transaction_body)?;

        let hash = Hasher::<256>::hash(&tx_body_bytes);
        Ok(hex::encode(hash))
    }

    fn parse_inputs(&self, tx_body: &TransactionBody) -> Result<Vec<InputView>> {
        let mut inputs = Vec::new();

        for input in &tx_body.inputs {
            let input_view = InputView {
                tx_hash: hex::encode(input.transaction_id),
                index: input.index as u32,
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

    fn parse_output(
        &mut self,
        output: &PseudoTransactionOutput<PostAlonzoTransactionOutput>,
    ) -> Result<OutputView> {
        match output {
            PseudoTransactionOutput::PostAlonzo(post_alonzo) => {
                let address = hex::encode(post_alonzo.address.to_vec());
                let value = self.parse_value(&post_alonzo.value)?;
                let datum = if let Some(ref datum_opt) = post_alonzo.datum_option {
                    self.parse_datum_option(datum_opt)?
                } else {
                    None
                };
                let script_ref = if let Some(ref script_wrap) = post_alonzo.script_ref {
                    self.parse_script_ref_inner(&script_wrap.0)?
                } else {
                    None
                };

                Ok(OutputView {
                    address,
                    value,
                    datum,
                    script_ref,
                })
            }
            PseudoTransactionOutput::Legacy(legacy) => {
                let address = hex::encode(legacy.address.as_ref() as &[u8]);
                Ok(OutputView {
                    address,
                    value: vec![], // Legacy outputs handled simply
                    datum: None,
                    script_ref: None,
                })
            }
        }
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
            PallasValue::Multiasset(ada, multi_asset) => {
                // Add ADA amount
                assets.push(AssetView {
                    policy_id: "ada".to_string(),
                    asset_name: "lovelace".to_string(),
                    amount: *ada,
                });

                // Add other assets
                for (policy_id_bytes, assets_map) in multi_asset.iter() {
                    let policy_id = hex::encode(policy_id_bytes.as_ref());

                    for (asset_name_bytes, amount) in assets_map.iter() {
                        let asset_name: String = if asset_name_bytes.is_empty() {
                            String::new()
                        } else {
                            hex::encode(asset_name_bytes.to_vec())
                        };

                        assets.push(AssetView {
                            policy_id: policy_id.clone(),
                            asset_name,
                            amount: u64::from(*amount),
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

    fn parse_script_ref_inner(
        &self,
        script: &pallas_primitives::conway::ScriptRef,
    ) -> Result<Option<String>> {
        use pallas_primitives::conway::PseudoScript;
        let script_type = match script {
            PseudoScript::NativeScript(_) => "NativeScript",
            PseudoScript::PlutusV1Script(_) => "PlutusV1",
            PseudoScript::PlutusV2Script(_) => "PlutusV2",
            PseudoScript::PlutusV3Script(_) => "PlutusV3",
        };
        Ok(Some(script_type.to_string()))
    }

    fn parse_redeemers(&self, witness_set: &WitnessSet) -> Result<Vec<RedeemerView>> {
        let mut redeemers = Vec::new();

        if let Some(ref redeemer_set) = witness_set.redeemer {
            for (key, value) in redeemer_set.iter() {
                let tag = match key.tag {
                    pallas_primitives::conway::RedeemerTag::Spend => "Spend".to_string(),
                    pallas_primitives::conway::RedeemerTag::Mint => "Mint".to_string(),
                    pallas_primitives::conway::RedeemerTag::Cert => "Cert".to_string(),
                    pallas_primitives::conway::RedeemerTag::Reward => "Reward".to_string(),
                    pallas_primitives::conway::RedeemerTag::Vote => "Vote".to_string(),
                    pallas_primitives::conway::RedeemerTag::Propose => "Propose".to_string(),
                };

                let data_bytes = minicbor::to_vec(&value.data)?;
                let data = try_decode_as_plutus_data(&data_bytes);

                let ex_units = (value.ex_units.mem, value.ex_units.steps);

                redeemers.push(RedeemerView {
                    tag,
                    index: key.index,
                    data,
                    ex_units,
                });
            }
        }

        Ok(redeemers)
    }

    fn parse_metadata(&self, tx: &Tx) -> Result<Option<serde_json::Value>> {
        // Metadata is in auxiliary_data, not transaction_body
        match &tx.auxiliary_data {
            pallas_codec::utils::Nullable::Some(aux_data) => {
                let metadata = self.extract_metadata(aux_data);
                if let Some(meta) = metadata {
                    let json = self.metadata_kvp_to_json(meta)?;
                    Ok(Some(json))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn extract_metadata<'a>(&self, aux_data: &'a AuxiliaryData) -> Option<&'a Metadata> {
        match aux_data {
            AuxiliaryData::Shelley(metadata) => Some(metadata),
            AuxiliaryData::ShelleyMa(metadata) => Some(&metadata.transaction_metadata),
            AuxiliaryData::PostAlonzo(post_alonzo) => post_alonzo.metadata.as_ref(),
        }
    }

    fn metadata_kvp_to_json(&self, metadata: &Metadata) -> Result<serde_json::Value> {
        let mut json_map = serde_json::Map::new();
        for (label, metadatum) in metadata.iter() {
            json_map.insert(label.to_string(), self.metadatum_to_json(metadatum)?);
        }
        Ok(serde_json::Value::Object(json_map))
    }

    fn metadatum_to_json(&self, metadatum: &Metadatum) -> Result<serde_json::Value> {
        match metadatum {
            Metadatum::Map(map) => {
                let mut json_map = serde_json::Map::new();
                for (key, value) in map.iter() {
                    if let Metadatum::Text(key_str) = key {
                        json_map.insert(key_str.clone(), self.metadatum_to_json(value)?);
                    } else if let Metadatum::Int(key_int) = key {
                        json_map.insert(
                            format!("{}", i128::from(*key_int)),
                            self.metadatum_to_json(value)?,
                        );
                    }
                }
                Ok(serde_json::Value::Object(json_map))
            }
            Metadatum::Array(items) => {
                let json_items: Result<Vec<_>> = items
                    .iter()
                    .map(|item| self.metadatum_to_json(item))
                    .collect();
                Ok(serde_json::Value::Array(json_items?))
            }
            Metadatum::Int(n) => {
                let val: i128 = (*n).into();
                Ok(serde_json::Value::Number((val as i64).into()))
            }
            Metadatum::Bytes(bytes) => Ok(serde_json::Value::String(hex::encode(bytes.to_vec()))),
            Metadatum::Text(s) => Ok(serde_json::Value::String(s.clone())),
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
