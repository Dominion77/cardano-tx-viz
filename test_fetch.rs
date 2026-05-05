use cardano_tx_viz::fetcher::{FetcherConfig, Network};

#[tokio::main]
async fn main() {
    // Test with a known good transaction
    let hash = "df8c580d50c1b8f97bd0831edb622be2737bc2a7e46971a8b369ab12e51cb214";

    println!("Testing Koios fetcher with hash: {}", hash);

    let config = FetcherConfig::Koios {
        network: Network::Mainnet,
    };
    let fetcher = config.create_fetcher();

    match fetcher.fetch(hash).await {
        Ok(raw_tx) => {
            println!("✅ SUCCESS!");
            println!("  Hash: {}", raw_tx.hash);
            println!("  CBOR length: {} bytes", raw_tx.cbor.len());

            // Try to parse it
            let mut parser = cardano_tx_viz::decoder::TxParser::new();
            match parser.parse_transaction(&raw_tx.cbor) {
                Ok(tx) => {
                    println!("✅ PARSED!");
                    println!("  Inputs: {}", tx.inputs.len());
                    println!("  Outputs: {}", tx.outputs.len());
                }
                Err(e) => {
                    println!("❌ PARSE ERROR: {:#}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ FETCH ERROR:");
            println!("{:#}", e);
        }
    }
}
