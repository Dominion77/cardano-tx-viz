use cardano_tx_viz::fetcher::{
    FetcherConfig, 
    Network, 
    RawTx
};

#[test]
fn test_raw_tx_creation() {
    let raw_tx = RawTx {
        hash: "test123".to_string(),
        cbor: vec![1, 2, 3, 4],
    };
    
    assert_eq!(raw_tx.hash, "test123");
    assert_eq!(raw_tx.cbor, vec![1, 2, 3, 4]);
}

#[test]
fn test_network_variants() {
    let mainnet = Network::Mainnet;
    let preprod = Network::Preprod;
    let preview = Network::Preview;
    
    // Just verify they can be created
    assert_ne!(format!("{:?}", mainnet), "");
    assert_ne!(format!("{:?}", preprod), "");
    assert_ne!(format!("{:?}", preview), "");
}

#[test]
fn test_fetcher_config_creation() {
    let config = FetcherConfig::Blockfrost {
        api_key: "test_key".to_string(),
        network: Network::Mainnet,
    };
    
    let fetcher = config.create_fetcher();
    // Just verify it doesn't panic
    drop(fetcher);
}