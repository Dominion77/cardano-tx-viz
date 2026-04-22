### `README.md`

```markdown
# cardano-tx-viz

 A terminal-based Cardano transaction debugger for Aiken/Plutus V3 developers.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)
![Cardano](https://img.shields.io/badge/cardano-conway-green.svg)

## Features

- **Fast Terminal UI** - Inspect transactions without leaving your terminal
- **Offline-Capable** - Parse and view previously fetched transactions
- **Full Plutus V3 Support** - Decode all Conway-era transaction components
- **Rich Datum Visualization** - Pretty-print Plutus data with syntax highlighting
- **Multi-Asset Support** - View ADA and native assets with proper formatting
- **Dual Backend Support** - Blockfrost (with API key) or Koios (public fallback)
- **Clipboard Integration** - Copy raw CBOR, policy IDs, or decoded data
- **Keyboard-Driven** - Vim-inspired keybindings for fast navigation

## Installation

### From Source

```bash
git clone https://github.com/yourusername/cardano-tx-viz.git
cd cardano-tx-viz
cargo build --release
```

The binary will be available at `target/release/cardano-tx-viz`

### Using Cargo

```bash
cargo install --git https://github.com/yourusername/cardano-tx-viz
```

## Configuration

### Blockfrost API Key (Optional)

Create a config file at `~/.config/cardano-tx-viz/config.toml`:

```toml
[blockfrost]
api_key = "your_blockfrost_api_key_here"

default_network = "mainnet"
```

Or set the environment variable:

```bash
export BLOCKFROST_API_KEY="your_api_key_here"
```

If no API key is provided, the app falls back to Koios public endpoints.

## Usage

### Basic Usage

```bash
# Start the TUI
cardano-tx-viz

# Fetch a specific transaction on startup
cardano-tx-viz --hash f2754b2d3a9e9e6f4b3e3d9f8c5e5a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8

# Specify network (mainnet, preprod, preview)
cardano-tx-viz --network preprod --hash <tx-hash>

# Enable debug logging
cardano-tx-viz --debug
```

### Command Line Options

```
USAGE:
    cardano-tx-viz [OPTIONS]

OPTIONS:
    -h, --help               Print help information
    -v, --version            Print version information
    -n, --network <NETWORK>  Cardano network [default: mainnet] [possible values: mainnet, preprod, preview]
    -t, --hash <HASH>        Transaction hash to inspect
        --debug              Enable debug logging
```

## Keybindings

### Navigation

| Key | Action |
|-----|--------|
| `/` or `i` | Focus search field |
| `Enter` | Fetch transaction |
| `↑` / `↓` | Navigate tree nodes |
| `→` / `Space` | Expand tree node |
| `←` | Collapse tree node |
| `PageUp` / `PageDown` | Scroll detail panel |

### Clipboard Operations

| Key | Action |
|-----|--------|
| `c` | Copy selected node content |
| `p` | Copy policy ID of selected asset |
| `r` | Copy raw CBOR/datum hex |
| `Ctrl+V` | Paste (in search mode) |

### Application

| Key | Action |
|-----|--------|
| `q` or `Esc` | Quit application |

## Features in Detail

### Transaction Tree View

- **Inputs** - View all transaction inputs with addresses and values
- **Outputs** - Inspect outputs including multi-asset values and datums
- **Datum Inspection** - Expand inline datums to see decoded Plutus data
- **Redeemers** - View redeemer tags, indices, and execution units
- **Metadata** - Browse transaction metadata in formatted JSON

### Plutus Data Decoding

The app automatically decodes Plutus data into readable format:

```
Constr 0 [
  Int(1000000)
  Bytes("deadbeef")
  Map {
    Text("key"): Int(42)
  }
]
```

### Multi-Asset Display

Assets are displayed with human-readable formatting:

- ADA: `₳ 1.500000`
- Native assets: `1000 MyToken (policy_id...)`

### Network Support

- **Mainnet** - Production Cardano network
- **Preprod** - Pre-production testnet
- **Preview** - Preview testnet

## Architecture

```
cardano-tx-viz
├── decoder/          # CBOR parsing and Plutus data decoding
├── fetcher/          # Blockfrost and Koios API clients
├── ui/              # Terminal UI components
├── app.rs           # Application state and event loop
├── clipboard.rs     # Cross-platform clipboard support
└── config.rs        # Configuration management
```

## Development

### Prerequisites

- Rust 1.70+
- Cargo

### Building

```bash
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test integration_test

# Run with output
cargo test -- --nocapture
```

### Project Structure

```
src/
├── main.rs           # Entry point, CLI parsing
├── lib.rs            # Library exports
├── app.rs            # App state and event handling
├── config.rs         # Configuration loading
├── clipboard.rs      # Clipboard utilities
├── decoder/
│   ├── mod.rs        # Module exports
│   ├── types.rs      # Domain types (TxView, PlutusNode, etc.)
│   ├── cbor.rs       # CBOR ↔ PlutusNode conversion
│   └── tx_parser.rs  # Transaction parsing logic
├── fetcher/
│   ├── mod.rs        # TxFetcher trait and config
│   ├── blockfrost.rs # Blockfrost API client
│   ├── koios.rs      # Koios API client
│   └── mock.rs       # Mock fetcher for testing
└── ui/
    ├── mod.rs        # Main UI render loop
    ├── tx_tree.rs    # Transaction tree widget
    └── detail.rs     # Detail panel widget
```

## Dependencies

- **ratatui** - Terminal UI framework
- **pallas** - Cardano primitives and CBOR handling
- **tokio** - Async runtime
- **reqwest** - HTTP client
- **clap** - CLI argument parsing
- **arboard** - Clipboard support

## Troubleshooting

### Clipboard not working on Linux

Install `xclip` or `xsel`:

```bash
# Ubuntu/Debian
sudo apt install xclip

# Arch
sudo pacman -S xclip

# Fedora
sudo dnf install xclip
```

### Transaction not found

- Verify the transaction hash is correct (64 hex characters)
- Check network selection matches the transaction's network
- Ensure you're using a valid Blockfrost API key or have internet for Koios

### Debug Mode

Enable debug logging to see detailed information:

```bash
cardano-tx-viz --debug
```

Logs are written to stderr and can be redirected:

```bash
cardano-tx-viz --debug 2> debug.log
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Pallas](https://github.com/txpipe/pallas) - Cardano Rust libraries
- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Aiken](https://aiken-lang.org) - Smart contract platform for Cardano

## Support

For bugs and feature requests, please [open an issue](https://github.com/yourusername/cardano-tx-viz/issues).

---

Made with ❤️ for the Cardano developer community
```

### `LICENSE` file

```text
MIT License

Copyright (c) 2024 cardano-tx-viz contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### `.gitignore`

```gitignore
# Rust
/target/
**/*.rs.bk
*.pdb

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Config (contains API keys)
config.toml
.env

# Logs
*.log
debug.log

# Build artifacts
release/
debug/
```

### `Cargo.toml` (Final version)

```toml
[package]
name = "cardano-tx-viz"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Terminal-based Cardano transaction debugger for Aiken/Plutus V3 developers"
license = "MIT"
repository = "https://github.com/yourusername/cardano-tx-viz"
readme = "README.md"
keywords = ["cardano", "plutus", "aiken", "blockchain", "tui"]
categories = ["command-line-utilities", "development-tools"]

[lib]
name = "cardano_tx_viz"
path = "src/lib.rs"

[[bin]]
name = "cardano-tx-viz"
path = "src/main.rs"

[[example]]
name = "manual_test"
path = "tests/manual_test.rs"

[dependencies]
# Terminal UI
ratatui = "0.27"
crossterm = "0.27"

# Cardano libraries
pallas = "0.30"
pallas-codec = "0.30"
pallas-primitives = "0.30"
pallas-crypto = "0.30"

# Async runtime
tokio = { version = "1.35", features = ["full"] }

# HTTP client
reqwest = { version = "0.11", features = ["json"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# CLI
clap = { version = "4.4", features = ["derive"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Async trait
async-trait = "0.1"

# Utilities
hex = "0.4"
arboard = "3.3"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
toml = "0.8"
dirs = "5.0"

[dev-dependencies]
tokio-test = "0.4"
mockito = "1.2"

[profile.release]
lto = true
codegen-units = 1
strip = true
```