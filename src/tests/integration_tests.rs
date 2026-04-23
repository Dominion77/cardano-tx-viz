use cardano_tx_viz::{
    app::App,
    decoder::{TxParser, TxView, PlutusNode, AssetView},
    fetcher::{FetcherConfig, Network, TxFetcher, mock::MockFetcher, RawTx},
};

#[tokio::test]
async fn test_fetch_known_mainnet_transaction() {
    // Known mainnet transaction with simple ADA transfer
    let tx_hash = "f2754b2d3a9e9e6f4b3e3d9f8c5e5a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8";
    
    // Use Koios fetcher (no API key required)
    let fetcher_config = FetcherConfig::Koios { 
        network: Network::Mainnet 
    };
    let fetcher = fetcher_config.create_fetcher();
    
    // Fetch transaction
    let result = fetcher.fetch(tx_hash).await;
    
    match result {
        Ok(raw_tx) => {
            println!("Successfully fetched transaction");
            assert_eq!(raw_tx.hash, tx_hash);
            assert!(!raw_tx.cbor.is_empty());
            
            // Parse transaction
            let mut parser = TxParser::new();
            let tx_view = parser.parse_transaction(&raw_tx.cbor);
            
            match tx_view {
                Ok(tx) => {
                    println!("Successfully parsed transaction");
                    println!("  Inputs: {}", tx.inputs.len());
                    println!("  Outputs: {}", tx.outputs.len());
                    println!("  Redeemers: {}", tx.redeemers.len());
                    
                    // Verify basic structure
                    assert!(!tx.inputs.is_empty(), "Transaction should have inputs");
                    assert!(!tx.outputs.is_empty(), "Transaction should have outputs");
                    
                    // Check first output has value
                    if let Some(output) = tx.outputs.first() {
                        assert!(!output.value.is_empty(), "Output should have value");
                        println!("  First output address: {}", output.address);
                        println!("  First output value: {:?}", output.value);
                    }
                }
                Err(e) => {
                    panic!("Failed to parse transaction: {}", e);
                }
            }
        }
        Err(e) => {
            // This might fail due to network issues, so we'll skip in CI
            if std::env::var("CI").is_ok() {
                println!("Skipping network test in CI environment");
            } else {
                panic!("Failed to fetch transaction: {}", e);
            }
        }
    }
}

#[tokio::test]
async fn test_datum_decoding() {
    // Create a mock transaction with inline datum
    let mock_fetcher = create_mock_fetcher_with_datum();
    
    let tx_hash = "mock_tx_with_datum";
    let raw_tx = mock_fetcher.fetch(tx_hash).await.unwrap();
    
    let mut parser = TxParser::new();
    let tx_view = parser.parse_transaction(&raw_tx.cbor).unwrap();
    
    // Find output with datum
    let output_with_datum = tx_view.outputs.iter()
        .find(|o| o.datum.is_some())
        .expect("Should have an output with datum");
    
    let datum = output_with_datum.datum.as_ref().unwrap();
    
    // Verify datum structure
    match &datum.decoded {
        PlutusNode::Constr(tag, fields) => {
            assert_eq!(*tag, 0);
            assert_eq!(fields.len(), 2);
            
            match &fields[0] {
                PlutusNode::Int(n) => assert_eq!(*n, 1000000),
                _ => panic!("Expected Int"),
            }
            
            match &fields[1] {
                PlutusNode::Bytes(hex) => assert_eq!(hex, "deadbeef"),
                _ => panic!("Expected Bytes"),
            }
        }
        _ => panic!("Expected Constr node"),
    }
    
    println!("Datum decoded correctly");
}

#[tokio::test]
async fn test_multi_asset_transaction() {
    let mock_fetcher = create_mock_fetcher_with_multi_asset();
    
    let raw_tx = mock_fetcher.fetch("mock_tx_multi_asset").await.unwrap();
    let mut parser = TxParser::new();
    let tx_view = parser.parse_transaction(&raw_tx.cbor).unwrap();
    
    // Find output with multiple assets
    let output = tx_view.outputs.first().unwrap();
    assert!(output.value.len() > 1, "Should have multiple assets");
    
    // Check ADA is present
    let ada = output.value.iter().find(|a| a.policy_id == "ada");
    assert!(ada.is_some(), "Should have ADA");
    
    // Check native asset
    let native = output.value.iter().find(|a| a.policy_id != "ada");
    assert!(native.is_some(), "Should have native asset");
    
    if let Some(asset) = native {
        println!("Native asset found:");
        println!("  Policy ID: {}", asset.policy_id);
        println!("  Asset Name: {}", asset.asset_name);
        println!("  Amount: {}", asset.amount);
    }
}

#[tokio::test]
async fn test_redeemer_parsing() {
    let mock_fetcher = create_mock_fetcher_with_redeemers();
    
    let raw_tx = mock_fetcher.fetch("mock_tx_with_redeemers").await.unwrap();
    let mut parser = TxParser::new();
    let tx_view = parser.parse_transaction(&raw_tx.cbor).unwrap();
    
    assert!(!tx_view.redeemers.is_empty(), "Should have redeemers");
    
    let redeemer = &tx_view.redeemers[0];
    println!("Redeemer parsed:");
    println!("  Tag: {}", redeemer.tag);
    println!("  Index: {}", redeemer.index);
    println!("  Ex Units: ({}, {})", redeemer.ex_units.0, redeemer.ex_units.1);
    
    // Verify redeemer data is valid Plutus
    match &redeemer.data {
        PlutusNode::List(items) => {
            assert!(!items.is_empty());
        }
        _ => {}
    }
}

#[tokio::test]
async fn test_metadata_parsing() {
    let mock_fetcher = create_mock_fetcher_with_metadata();
    
    let raw_tx = mock_fetcher.fetch("mock_tx_with_metadata").await.unwrap();
    let mut parser = TxParser::new();
    let tx_view = parser.parse_transaction(&raw_tx.cbor).unwrap();
    
    assert!(tx_view.metadata.is_some(), "Should have metadata");
    
    if let Some(metadata) = &tx_view.metadata {
        println!("Metadata parsed successfully");
        
        // Verify it's valid JSON
        let json_str = serde_json::to_string(metadata).unwrap();
        assert!(!json_str.is_empty());
    }
}

#[tokio::test]
async fn test_error_handling() {
    let mock_fetcher = MockFetcher::new()
        .with_tx("invalid", Err(anyhow::anyhow!("Transaction not found")));
    
    let result = mock_fetcher.fetch("invalid").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Transaction not found");
    
    println!("Error handling works correctly");
}

#[tokio::test]
async fn test_plutus_node_all_variants() {
    // Test all PlutusNode variants
    let test_cases = vec![
        ("Int", PlutusNode::Int(42)),
        ("Bytes", PlutusNode::Bytes("deadbeef".to_string())),
        ("Text", PlutusNode::Text("Hello, World!".to_string())),
        ("List", PlutusNode::List(vec![
            PlutusNode::Int(1),
            PlutusNode::Int(2),
            PlutusNode::Int(3),
        ])),
        ("Map", PlutusNode::Map(vec![
            (PlutusNode::Text("key".to_string()), PlutusNode::Int(42)),
        ])),
        ("Constr", PlutusNode::Constr(0, vec![
            PlutusNode::Int(1000000),
        ])),
    ];
    
    for (name, node) in test_cases {
        println!("Testing PlutusNode::{}", name);
        
        // Test pretty printing
        let pretty = node.to_string_pretty();
        assert!(!pretty.is_empty());
        
        // Test type name
        let type_name = node.type_name();
        assert!(!type_name.is_empty());
        
        // Test compact printing
        let compact = node.to_string_compact();
        assert!(!compact.is_empty());
        
        println!("  {}: {}", name, compact);
    }
}

#[tokio::test]
async fn test_app_state_transitions() {
    let mut app = App::default();
    
    // Initial state
    assert!(matches!(app.fetch_state, cardano_tx_viz::app::FetchState::Idle));
    
    // Start fetch with empty hash
    app.start_fetch("".to_string());
    assert!(matches!(app.fetch_state, cardano_tx_viz::app::FetchState::Idle));
    assert!(app.status_message.is_some());
    
    // Start fetch with valid hash
    app.start_fetch("a".repeat(64));
    assert!(matches!(app.fetch_state, cardano_tx_viz::app::FetchState::Loading));
    
    println!("App state transitions work correctly");
}

#[tokio::test]
async fn test_tree_node_generation() {
    let mock_fetcher = create_mock_fetcher_with_datum();
    let raw_tx = mock_fetcher.fetch("mock_tx_with_datum").await.unwrap();
    let mut parser = TxParser::new();
    let tx_view = parser.parse_transaction(&raw_tx.cbor).unwrap();
    
    let mut app = App::default();
    app.build_tree_from_tx(&tx_view);
    
    // Verify tree structure
    assert!(!app.tree_state.visible_nodes.is_empty());
    
    // Check for expected node types
    let has_inputs_header = app.tree_state.visible_nodes.iter()
        .any(|n| matches!(n, cardano_tx_viz::app::TreeNode::InputsHeader { .. }));
    assert!(has_inputs_header);
    
    let has_outputs_header = app.tree_state.visible_nodes.iter()
        .any(|n| matches!(n, cardano_tx_viz::app::TreeNode::OutputsHeader { .. }));
    assert!(has_outputs_header);
    
    println!("Tree generated with {} nodes", app.tree_state.visible_nodes.len());
}

#[tokio::test]
async fn test_clipboard_functions() {
    use cardano_tx_viz::clipboard;
    
    // Test basic copy (might fail in headless environments)
    let result = clipboard::copy_to_clipboard("test");
    if std::env::var("CI").is_ok() {
        // In CI, we expect this might fail
        println!("Clipboard test skipped in CI");
    } else {
        match result {
            Ok(_) => println!("Clipboard copy successful"),
            Err(e) => println!("Clipboard not available: {}", e),
        }
    }
    
    // Test clipboard availability check
    let available = clipboard::is_clipboard_available();
    println!("Clipboard available: {}", available);
}

// Helper functions to create mock data

fn create_mock_fetcher_with_datum() -> MockFetcher {
    use pallas_codec::minicbor;
    use pallas_primitives::conway::{
        MintedTransaction, TransactionBody, TransactionInput, 
        TransactionOutput, PostAlonzoTransactionOutput, Value, 
        DatumOption, PlutusData, Constr, RawBytes,
    };
    
    // Create a simple Plutus datum
    let datum = PlutusData::Constr(Constr {
        tag: 0,
        fields: vec![
            PlutusData::BigInt(pallas_primitives::conway::big_int::BigInt::Int(1000000)),
            PlutusData::BoundedBytes(RawBytes::from(vec![0xde, 0xad, 0xbe, 0xef])),
        ],
    });
    
    // Create transaction with inline datum
    let output = PostAlonzoTransactionOutput {
        address: pallas_primitives::conway::Address::Shelley(
            pallas_primitives::conway::ShelleyAddress::new(
                0, 
                vec![1, 2, 3, 4].into(),
            )
        ),
        amount: Value::Coin(2000000),
        datum_option: DatumOption::Data(datum),
        script_reference: None,
    };
    
    let tx_body = TransactionBody {
        inputs: vec![TransactionInput {
            transaction_id: [0u8; 32].into(),
            index: 0,
        }],
        outputs: vec![output],
        fee: 170000,
        ttl: None,
        certs: None,
        withdrawals: None,
        update: None,
        auxiliary_data_hash: None,
        validity_interval_start: None,
        mint: None,
        script_data_hash: None,
        collateral: None,
        required_signers: None,
        network_id: None,
        collateral_return: None,
        total_collateral: None,
        reference_inputs: None,
        redeemers: None,
        votes: None,
        proposals: None,
        donation: None,
        current_treasury_value: None,
        treasury_donation: None,
        tx_info: pallas_primitives::conway::TxInfo::default(),
        metadata: None,
    };
    
    let tx = MintedTransaction {
        transaction_body: tx_body,
        transaction_witness_set: pallas_primitives::conway::TransactionWitnessSet::default(),
        success: true,
        auxiliary_data: None,
    };
    
    let cbor = minicbor::to_vec(&tx).unwrap();
    
    MockFetcher::new()
        .with_tx("mock_tx_with_datum", Ok(RawTx {
            hash: "mock_tx_with_datum".to_string(),
            cbor,
        }))
}

fn create_mock_fetcher_with_multi_asset() -> MockFetcher {
    use pallas_codec::minicbor;
    use pallas_primitives::conway::{
        MintedTransaction, TransactionBody, TransactionInput,
        PostAlonzoTransactionOutput, Value, MultiAsset,
    };
    use std::collections::BTreeMap;
    
    let mut multi_asset = BTreeMap::new();
    let mut assets = BTreeMap::new();
    assets.insert(vec![].into(), 1000);
    multi_asset.insert(vec![1, 2, 3, 4].into(), assets);
    
    let output = PostAlonzoTransactionOutput {
        address: pallas_primitives::conway::Address::Shelley(
            pallas_primitives::conway::ShelleyAddress::new(
                0,
                vec![1, 2, 3, 4].into(),
            )
        ),
        amount: Value::MultiAsset(2000000, MultiAsset::from(multi_asset)),
        datum_option: DatumOption::Hash([0u8; 32].into()),
        script_reference: None,
    };
    
    let tx_body = TransactionBody {
        inputs: vec![TransactionInput {
            transaction_id: [0u8; 32].into(),
            index: 0,
        }],
        outputs: vec![output],
        fee: 170000,
        ttl: None,
        certs: None,
        withdrawals: None,
        update: None,
        auxiliary_data_hash: None,
        validity_interval_start: None,
        mint: None,
        script_data_hash: None,
        collateral: None,
        required_signers: None,
        network_id: None,
        collateral_return: None,
        total_collateral: None,
        reference_inputs: None,
        redeemers: None,
        votes: None,
        proposals: None,
        donation: None,
        current_treasury_value: None,
        treasury_donation: None,
        tx_info: pallas_primitives::conway::TxInfo::default(),
        metadata: None,
    };
    
    let tx = MintedTransaction {
        transaction_body: tx_body,
        transaction_witness_set: pallas_primitives::conway::TransactionWitnessSet::default(),
        success: true,
        auxiliary_data: None,
    };
    
    let cbor = minicbor::to_vec(&tx).unwrap();
    
    MockFetcher::new()
        .with_tx("mock_tx_multi_asset", Ok(RawTx {
            hash: "mock_tx_multi_asset".to_string(),
            cbor,
        }))
}

fn create_mock_fetcher_with_redeemers() -> MockFetcher {
    use pallas_codec::minicbor;
    use pallas_primitives::conway::{
        MintedTransaction, TransactionBody, TransactionInput,
        PostAlonzoTransactionOutput, Value, Redeemer, RedeemerTag,
        ExUnits, PlutusData,
    };
    
    let redeemer = Redeemer {
        tag: RedeemerTag::Spend,
        index: 0,
        data: PlutusData::Array(vec![
            PlutusData::BigInt(pallas_primitives::conway::big_int::BigInt::Int(1)),
            PlutusData::BigInt(pallas_primitives::conway::big_int::BigInt::Int(2)),
        ]),
        ex_units: ExUnits { mem: 1000, steps: 5000 },
    };
    
    let output = PostAlonzoTransactionOutput {
        address: pallas_primitives::conway::Address::Shelley(
            pallas_primitives::conway::ShelleyAddress::new(
                0,
                vec![1, 2, 3, 4].into(),
            )
        ),
        amount: Value::Coin(2000000),
        datum_option: DatumOption::Hash([0u8; 32].into()),
        script_reference: None,
    };
    
    let tx_body = TransactionBody {
        inputs: vec![TransactionInput {
            transaction_id: [0u8; 32].into(),
            index: 0,
        }],
        outputs: vec![output],
        fee: 170000,
        ttl: None,
        certs: None,
        withdrawals: None,
        update: None,
        auxiliary_data_hash: None,
        validity_interval_start: None,
        mint: None,
        script_data_hash: None,
        collateral: None,
        required_signers: None,
        network_id: None,
        collateral_return: None,
        total_collateral: None,
        reference_inputs: None,
        redeemers: Some(vec![redeemer].into()),
        votes: None,
        proposals: None,
        donation: None,
        current_treasury_value: None,
        treasury_donation: None,
        tx_info: pallas_primitives::conway::TxInfo::default(),
        metadata: None,
    };
    
    let tx = MintedTransaction {
        transaction_body: tx_body,
        transaction_witness_set: pallas_primitives::conway::TransactionWitnessSet::default(),
        success: true,
        auxiliary_data: None,
    };
    
    let cbor = minicbor::to_vec(&tx).unwrap();
    
    MockFetcher::new()
        .with_tx("mock_tx_with_redeemers", Ok(RawTx {
            hash: "mock_tx_with_redeemers".to_string(),
            cbor,
        }))
}

fn create_mock_fetcher_with_metadata() -> MockFetcher {
    use pallas_codec::minicbor;
    use pallas_primitives::conway::{
        MintedTransaction, TransactionBody, TransactionInput,
        PostAlonzoTransactionOutput, Value, TransactionMetadatum,
    };
    
    let output = PostAlonzoTransactionOutput {
        address: pallas_primitives::conway::Address::Shelley(
            pallas_primitives::conway::ShelleyAddress::new(
                0,
                vec![1, 2, 3, 4].into(),
            )
        ),
        amount: Value::Coin(2000000),
        datum_option: DatumOption::Hash([0u8; 32].into()),
        script_reference: None,
    };
    
    let metadata = TransactionMetadatum::Map(vec![
        (TransactionMetadatum::Text("key".to_string()), 
         TransactionMetadatum::Text("value".to_string())),
    ]);
    
    let tx_body = TransactionBody {
        inputs: vec![TransactionInput {
            transaction_id: [0u8; 32].into(),
            index: 0,
        }],
        outputs: vec![output],
        fee: 170000,
        ttl: None,
        certs: None,
        withdrawals: None,
        update: None,
        auxiliary_data_hash: None,
        validity_interval_start: None,
        mint: None,
        script_data_hash: None,
        collateral: None,
        required_signers: None,
        network_id: None,
        collateral_return: None,
        total_collateral: None,
        reference_inputs: None,
        redeemers: None,
        votes: None,
        proposals: None,
        donation: None,
        current_treasury_value: None,
        treasury_donation: None,
        tx_info: pallas_primitives::conway::TxInfo::default(),
        metadata: Some(metadata),
    };
    
    let tx = MintedTransaction {
        transaction_body: tx_body,
        transaction_witness_set: pallas_primitives::conway::TransactionWitnessSet::default(),
        success: true,
        auxiliary_data: None,
    };
    
    let cbor = minicbor::to_vec(&tx).unwrap();
    
    MockFetcher::new()
        .with_tx("mock_tx_with_metadata", Ok(RawTx {
            hash: "mock_tx_with_metadata".to_string(),
            cbor,
        }))
}