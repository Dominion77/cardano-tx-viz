use cardano_tx_viz::{
    app::App,
    decoder::TxParser,
    fetcher::{FetcherConfig, Network},
};

#[tokio::test]
async fn test_app_creation() {
    let app = App::new(
        Network::Mainnet,
        FetcherConfig::Koios {
            network: Network::Mainnet,
        },
    );

    // Verify app is created with correct initial state
    assert!(matches!(
        app.fetch_state,
        cardano_tx_viz::app::FetchState::Idle
    ));
    assert_eq!(app.input_hash, "");
    assert_eq!(app.network, Network::Mainnet);

    println!("App created successfully");
}

#[test]
fn test_tx_parser_creation() {
    let parser = TxParser::new();

    // Parser should be created successfully
    // This is a simple smoke test
    drop(parser);

    println!("TxParser created successfully");
}

#[test]
fn test_network_enum() {
    let mainnet = Network::Mainnet;
    let preprod = Network::Preprod;
    let preview = Network::Preview;

    assert_ne!(mainnet, preprod);
    assert_ne!(preprod, preview);

    println!("Network enum works correctly");
}

#[test]
fn test_fetcher_config_creation() {
    let koios_config = FetcherConfig::Koios {
        network: Network::Mainnet,
    };
    let blockfrost_config = FetcherConfig::Blockfrost {
        api_key: "test_key".to_string(),
        network: Network::Mainnet,
    };

    // Configs should be created successfully
    match koios_config {
        FetcherConfig::Koios { network } => assert_eq!(network, Network::Mainnet),
        _ => panic!("Wrong config type"),
    }

    match blockfrost_config {
        FetcherConfig::Blockfrost { api_key, network } => {
            assert_eq!(api_key, "test_key");
            assert_eq!(network, Network::Mainnet);
        }
        _ => panic!("Wrong config type"),
    }

    println!("FetcherConfig creation works correctly");
}

#[test]
fn test_plutus_node_display() {
    use cardano_tx_viz::decoder::PlutusNode;

    let int_node = PlutusNode::Int(42);
    let text_node = PlutusNode::Text("hello".to_string());
    let bytes_node = PlutusNode::Bytes("deadbeef".to_string());

    // Test that nodes can be displayed using pretty print
    assert!(!int_node.to_string_pretty().is_empty());
    assert!(!text_node.to_string_pretty().is_empty());
    assert!(!bytes_node.to_string_pretty().is_empty());

    println!("PlutusNode display works correctly");
}

#[test]
fn test_clipboard_functionality() {
    use cardano_tx_viz::clipboard;

    // Test clipboard availability (may fail on headless systems)
    let available = clipboard::is_clipboard_available();

    if available {
        let result = clipboard::copy_to_clipboard("test");
        match result {
            Ok(_) => println!("Clipboard copy successful"),
            Err(e) => println!("Clipboard not available: {}", e),
        }
    } else {
        println!("Clipboard not available in this environment");
    }
}

#[test]
fn test_asset_view_creation() {
    use cardano_tx_viz::decoder::AssetView;

    let ada_asset = AssetView {
        policy_id: "ada".to_string(),
        asset_name: "lovelace".to_string(),
        amount: 1000000,
    };

    let native_asset = AssetView {
        policy_id: "policy123".to_string(),
        asset_name: "token456".to_string(),
        amount: 100,
    };

    assert_eq!(ada_asset.policy_id, "ada");
    assert_eq!(native_asset.amount, 100);

    println!("AssetView creation works correctly");
}
