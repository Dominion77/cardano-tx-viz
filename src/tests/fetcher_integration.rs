use cardano_tx_viz::fetcher::{
    mock::MockFetcher, 
    FetcherConfig, 
    Network, 
    RawTx, 
    TxFetcher
};

#[tokio::test]
async fn test_mock_fetcher() {
    let mock_tx = RawTx {
        hash: "test123".to_string(),
        cbor: vec![1, 2, 3, 4],
    };
    
    let mock = MockFetcher::new()
        .with_tx("test123", Ok(mock_tx.clone()))
        .with_datum("datum123", Ok(vec![5, 6, 7, 8]));

    let result = mock.fetch("test123").await.unwrap();
    assert_eq!(result.hash, "test123");
    assert_eq!(result.cbor, vec![1, 2, 3, 4]);

    let datum = mock.fetch_datum("datum123").await.unwrap();
    assert_eq!(datum, vec![5, 6, 7, 8]);
}

#[tokio::test]
async fn test_network_from_str() {
    assert!(matches!(
        "mainnet".parse::<Network>().unwrap(),
        Network::Mainnet
    ));
    assert!(matches!(
        "preprod".parse::<Network>().unwrap(),
        Network::Preprod
    ));
    assert!(matches!(
        "preview".parse::<Network>().unwrap(),
        Network::Preview
    ));
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