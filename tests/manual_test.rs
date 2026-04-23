use cardano_tx_viz::{
    app::App,
    fetcher::{FetcherConfig, Network},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Cardano TX Viz Manual Test ===\n");
    
    // Test 1: Fetch known transaction
    println!("Test 1: Testing Koios API connection...");
    println!("NOTE: Use a real mainnet transaction hash to test fetching");
    println!("Example: cardano-tx-viz --hash <real-tx-hash>");
    
    // This is a placeholder hash - replace with a real one to test
    let tx_hash = "f2754b2d3a9e9e6f4b3e3d9f8c5e5a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8";
    println!("Testing with hash: {}", tx_hash);
    
    let fetcher_config = FetcherConfig::Koios { 
        network: Network::Mainnet 
    };
    let fetcher = fetcher_config.create_fetcher();
    
    match fetcher.fetch(tx_hash).await {
        Ok(raw_tx) => {
            println!("Transaction fetched successfully");
            println!("  CBOR size: {} bytes", raw_tx.cbor.len());
            
            // Parse transaction
            let mut parser = cardano_tx_viz::decoder::TxParser::new();
            match parser.parse_transaction(&raw_tx.cbor) {
                Ok(tx) => {
                    println!("Transaction parsed successfully");
                    println!("  Hash: {}", tx.hash);
                    println!("  Inputs: {}", tx.inputs.len());
                    println!("  Outputs: {}", tx.outputs.len());
                    println!("  Redeemers: {}", tx.redeemers.len());
                    
                    // Show first output
                    if let Some(output) = tx.outputs.first() {
                        println!("\n  First output:");
                        println!("    Address: {}", output.address);
                        for asset in &output.value {
                            println!("    Asset: {} {} ({})", 
                                asset.amount, asset.asset_name, asset.policy_id);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to parse transaction: {:#}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to fetch transaction:");
            println!("  Error: {:#}", e);
            println!("\nPossible causes:");
            println!("  - Invalid transaction hash (not a real transaction)");
            println!("  - Network connectivity issues");
            println!("  - Koios API temporarily unavailable");
            println!("  - Transaction not on mainnet (try --network preprod/preview)");
        }
    }
    
    // Test 2: App state transitions
    println!("\nTest 2: Testing app state transitions...");
    let mut app = App::default();
    println!("  Initial state: {:?}", app.fetch_state);
    
    app.start_fetch("test".to_string());
    println!("  After start_fetch: {:?}", app.fetch_state);
    
    // Test 3: Input mode switching
    println!("\nTest 3: Testing input modes...");
    println!("  Default mode: {:?}", app.input_mode);
    
    // Simulate key press '/'
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    app.handle_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()));
    println!("  After '/': {:?}", app.input_mode);
    
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    println!("  After Esc: {:?}", app.input_mode);
    
    println!("\n=== All manual tests complete ===");
    Ok(())
}