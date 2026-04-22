# cardano-tx-viz

🔍 A terminal-based Cardano transaction debugger for Aiken/Plutus V3 developers.

[![Crates.io](https://img.shields.io/crates/v/cardano-tx-viz.svg)](https://crates.io/crates/cardano-tx-viz)
[![Documentation](https://docs.rs/cardano-tx-viz/badge.svg)](https://docs.rs/cardano-tx-viz)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/yourusername/cardano-tx-viz/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](https://www.rust-lang.org)
[![Cardano](https://img.shields.io/badge/cardano-conway-green.svg)](https://cardano.org)

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

### Via Cargo (Recommended)

```bash
cargo install cardano-tx-viz
```

This will download, compile, and install the latest version from [crates.io](https://crates.io/crates/cardano-tx-viz).


### Pre-built Binaries

Pre-built binaries for Linux, macOS, and Windows are available on the [releases page](https://github.com/yourusername/cardano-tx-viz/releases).

## Quick Start

```bash
# Install
cargo install cardano-tx-viz

# Run
cardano-tx-viz

# Fetch a transaction directly
cardano-tx-viz --hash f2754b2d3a9e9e6f4b3e3d9f8c5e5a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8

# Use a different network
cardano-tx-viz --network preprod --hash <tx-hash>
```

## Configuration (Optional)

### Blockfrost API Key

For better rate limits and reliability, configure a Blockfrost API key:

**Option 1: Config file** (`~/.config/cardano-tx-viz/config.toml`):
```toml
[blockfrost]
api_key = "your_blockfrost_api_key_here"
default_network = "mainnet"
```

**Option 2: Environment variable**:
```bash
export BLOCKFROST_API_KEY="your_api_key_here"
```

If no API key is provided, the app falls back to Koios public endpoints automatically.

## Usage

### Command Line Options

```
USAGE:
    cardano-tx-viz [OPTIONS]

OPTIONS:
    -h, --help               Print help information
    -V, --version            Print version information
    -n, --network <NETWORK>  Cardano network [default: mainnet] 
                             [possible values: mainnet, preprod, preview]
    -t, --hash <HASH>        Transaction hash to inspect on startup
        --debug              Enable debug logging
```

### Interactive Keybindings

#### Navigation
| Key | Action |
|-----|--------|
| `/` or `i` | Focus search field |
| `Enter` | Fetch transaction |
| `↑` / `↓` | Navigate tree nodes |
| `→` / `Space` | Expand tree node |
| `←` | Collapse tree node |
| `PageUp` / `PageDown` | Scroll detail panel |

#### Clipboard Operations
| Key | Action |
|-----|--------|
| `c` | Copy selected node content |
| `p` | Copy policy ID of selected asset |
| `r` | Copy raw CBOR/datum hex |
| `Ctrl+V` | Paste (in search mode) |

#### Application
| Key | Action |
|-----|--------|
| `q` or `Esc` | Quit application |

## Screenshots

```
┌─────────────────────────────────────────────────────┐
│  🔍 cardano-tx-viz  │  hash: f2754b2d...            │
├────────────────────────┬────────────────────────────┤
│  TX TREE               │  DETAIL                    │
│  ▼ Inputs (2)          │  📦 Datum                  │
│    ▶ #0 addr1q...      │  Raw CBOR: d87980...      │
│    ▶ #1 addr1q...      │                            │
│  ▼ Outputs (3)         │  Decoded:                  │
│    ▶ #0 addr1q...      │  Constr 0 [               │
│      ▶ Datum           │    Int(1000000)            │
│        Constr 0 [...]  │    Bytes("deadbeef")       │
│  ▼ Redeemers (1)       │  ]                         │
│    ▶ Spend #0          │                            │
├────────────────────────┴────────────────────────────┤
│  [/] search  [↑/↓] navigate  [c] copy  [q] quit    │
└─────────────────────────────────────────────────────┘
```

## Features in Detail

### Transaction Inspection

- **Inputs** - View all transaction inputs with addresses and values
- **Outputs** - Inspect outputs including multi-asset values and datums
- **Datum Decoding** - Expand inline datums to see decoded Plutus data
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
- Try with `--debug` flag for more detailed error messages

### Debug Mode

Enable debug logging to see detailed information:

```bash
cardano-tx-viz --debug
```

Logs are written to stderr and can be redirected:

```bash
cardano-tx-viz --debug 2> debug.log
```

## For Aiken/Plutus Developers

This tool is specifically designed for smart contract developers:

- **Quick Datum Inspection** - See exactly what data your validator receives
- **Redeemer Verification** - Check redeemer structure and execution units
- **Script Reference Detection** - Identify reference scripts in outputs
- **CBOR Export** - Copy raw CBOR for debugging or testing

### Example Workflow

```bash
# 1. Submit a transaction from your Aiken contract
aiken tx submit ...

# 2. Grab the transaction hash and inspect it
cardano-tx-viz --hash <tx-hash> --network preview

# 3. Navigate to your validator's output
# 4. Expand the datum to verify the data structure
# 5. Press 'r' to copy the raw CBOR for testing
```


## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request


## Dependencies

- **[ratatui](https://crates.io/crates/ratatui)** - Terminal UI framework
- **[pallas](https://crates.io/crates/pallas)** - Cardano primitives and CBOR handling
- **[tokio](https://crates.io/crates/tokio)** - Async runtime
- **[reqwest](https://crates.io/crates/reqwest)** - HTTP client
- **[clap](https://crates.io/crates/clap)** - CLI argument parsing
- **[arboard](https://crates.io/crates/arboard)** - Clipboard support

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Pallas](https://github.com/txpipe/pallas) - Cardano Rust libraries
- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Aiken](https://aiken-lang.org) - Smart contract platform for Cardano
- [Blockfrost](https://blockfrost.io) - Cardano API service
- [Koios](https://koios.rest) - Public Cardano API

---

Made with ❤️ for the Cardano developer community