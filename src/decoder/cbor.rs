use anyhow::{Context, Result};
use pallas_codec::minicbor;
use pallas_primitives::conway::{PlutusData, RawBytes};
use crate::decoder::types::PlutusNode;

pub fn decode_plutus_data(cbor_bytes: &[u8]) -> Result<PlutusNode> {
    let plutus_data: PlutusData = minicbor::decode(cbor_bytes)
        .context("Failed to decode CBOR as PlutusData")?;
    
    Ok(convert_plutus_data(plutus_data))
}

pub fn try_decode_as_plutus_data(cbor_bytes: &[u8]) -> PlutusNode {
    match decode_plutus_data(cbor_bytes) {
        Ok(node) => node,
        Err(_) => {
            // Fallback: return as raw bytes
            PlutusNode::Bytes(hex::encode(cbor_bytes))
        }
    }
}

fn convert_plutus_data(data: PlutusData) -> PlutusNode {
    match data {
        PlutusData::Constr(constr) => {
            let fields = constr
                .fields
                .into_iter()
                .map(convert_plutus_data)
                .collect();
            PlutusNode::Constr(constr.tag.into(), fields)
        }
        PlutusData::Map(map) => {
            let entries = map
                .entries
                .into_iter()
                .map(|(k, v)| (convert_plutus_data(k), convert_plutus_data(v)))
                .collect();
            PlutusNode::Map(entries)
        }
        PlutusData::BigInt(big_int) => {
            match big_int {
                pallas_primitives::conway::big_int::BigInt::Int(i) => PlutusNode::Int(i as i128),
                pallas_primitives::conway::big_int::BigInt::BigUInt(bytes) |
                pallas_primitives::conway::big_int::BigInt::BigNInt(bytes) => {
                    // Handle big integers by converting to i128 if possible, or fallback to bytes
                    if bytes.len() <= 16 {
                        let mut value: i128 = 0;
                        for &byte in bytes.as_ref().iter() {
                            value = (value << 8) | (byte as i128);
                        }
                        if matches!(big_int, pallas_primitives::conway::big_int::BigInt::BigNInt(_)) {
                            value = -value;
                        }
                        PlutusNode::Int(value)
                    } else {
                        PlutusNode::Bytes(hex::encode(bytes.as_ref()))
                    }
                }
            }
        }
        PlutusData::BoundedBytes(bytes) => {
            PlutusNode::Bytes(hex::encode(bytes.as_ref()))
        }
        PlutusData::Array(array) => {
            let items = array
                .iter()
                .cloned()
                .map(convert_plutus_data)
                .collect();
            PlutusNode::List(items)
        }
    }
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

pub fn extract_raw_cbor(data: &PlutusData) -> Result<String> {
    let mut bytes = Vec::new();
    minicbor::encode(data, &mut bytes)
        .context("Failed to encode PlutusData to CBOR")?;
    Ok(hex::encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pallas_primitives::conway::{Constr, PlutusData};

    #[test]
    fn test_decode_simple_int() {
        let data = PlutusData::BigInt(pallas_primitives::conway::big_int::BigInt::Int(42));
        let node = convert_plutus_data(data);
        assert_eq!(node, PlutusNode::Int(42));
    }

    #[test]
    fn test_decode_constr() {
        let constr = Constr {
            tag: 121,
            fields: vec![
                PlutusData::BigInt(pallas_primitives::conway::big_int::BigInt::Int(1)),
                PlutusData::BigInt(pallas_primitives::conway::big_int::BigInt::Int(2)),
            ],
        };
        let data = PlutusData::Constr(constr);
        let node = convert_plutus_data(data);
        
        match node {
            PlutusNode::Constr(tag, fields) => {
                assert_eq!(tag, 121);
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0], PlutusNode::Int(1));
                assert_eq!(fields[1], PlutusNode::Int(2));
            }
            _ => panic!("Expected Constr"),
        }
    }

    #[test]
    fn test_decode_map() {
        use pallas_primitives::conway::Map;
        
        let map = Map {
            entries: vec![
                (
                    PlutusData::BigInt(pallas_primitives::conway::big_int::BigInt::Int(1)),
                    PlutusData::BoundedBytes(RawBytes::from(vec![1, 2, 3])),
                )
            ],
        };
        let data = PlutusData::Map(map);
        let node = convert_plutus_data(data);
        
        match node {
            PlutusNode::Map(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0, PlutusNode::Int(1));
                assert_eq!(entries[0].1, PlutusNode::Bytes("010203".to_string()));
            }
            _ => panic!("Expected Map"),
        }
    }

    #[test]
    fn test_decode_array() {
        let array = vec![
            PlutusData::BigInt(pallas_primitives::conway::big_int::BigInt::Int(1)),
            PlutusData::BigInt(pallas_primitives::conway::big_int::BigInt::Int(2)),
        ];
        let data = PlutusData::Array(array);
        let node = convert_plutus_data(data);
        
        match node {
            PlutusNode::List(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], PlutusNode::Int(1));
                assert_eq!(items[1], PlutusNode::Int(2));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_decode_bounded_bytes() {
        let data = PlutusData::BoundedBytes(RawBytes::from(vec![0xde, 0xad, 0xbe, 0xef]));
        let node = convert_plutus_data(data);
        assert_eq!(node, PlutusNode::Bytes("deadbeef".to_string()));
    }

    #[test]
    fn test_try_decode_fallback() {
        let invalid_cbor = vec![0xFF, 0xFF, 0xFF]; // Invalid CBOR
        let node = try_decode_as_plutus_data(&invalid_cbor);
        assert!(matches!(node, PlutusNode::Bytes(_)));
    }
}