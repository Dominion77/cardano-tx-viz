use cardano_tx_viz::decoder::{AssetView, PlutusNode, TxParser};

#[tokio::test]
async fn test_parse_simple_transaction() {
    // This test would use a real mainnet transaction CBOR
    // For now, we'll test the parser structure
    let _parser = TxParser::new();

    // A minimal transaction CBOR would go here
    // This is a placeholder for actual test data
}

#[test]
fn test_plutus_node_conversion() {
    use cardano_tx_viz::decoder::cbor::try_decode_as_plutus_data;

    // Simple integer datum
    let cbor = vec![0x01]; // CBOR for integer 1
    let node = try_decode_as_plutus_data(&cbor);
    assert_eq!(node, PlutusNode::Int(1));
}

#[test]
fn test_asset_view_creation() {
    let asset = AssetView {
        policy_id: "ada".to_string(),
        asset_name: "lovelace".to_string(),
        amount: 1000000,
    };

    assert_eq!(asset.policy_id, "ada");
    assert_eq!(asset.amount, 1000000);
}
