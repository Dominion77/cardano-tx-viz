#!/bin/bash

echo "=== Testing cardano-tx-viz CLI ==="
echo ""

# Test 1: Help works
echo -n "1. Help menu works: "
cargo run -- --help > /dev/null 2>&1 && echo "✅" || echo "❌"

# Test 2: Version works
echo -n "2. Version works: "
cargo run -- --version > /dev/null 2>&1 && echo "✅" || echo "❌"

# Test 3: --network flag parses
echo -n "3. --network flag: "
cargo run -- --network mainnet --help > /dev/null 2>&1 && echo "✅" || echo "❌"

# Test 4: -t flag parses
echo -n "4. -t flag: "
cargo run -- -t abc --help > /dev/null 2>&1 && echo "✅" || echo "❌"

# Test 5: --hash flag parses
echo -n "5. --hash flag: "
cargo run -- --hash abc --help > /dev/null 2>&1 && echo "✅" || echo "❌"

# Test 6: --debug flag parses
echo -n "6. --debug flag: "
cargo run -- --debug --help > /dev/null 2>&1 && echo "✅" || echo "❌"

# Test 7: Invalid network rejected
echo -n "7. Invalid network rejected: "
cargo run -- --network invalid 2>&1 | grep -q "Invalid" && echo "✅" || echo "❌"

# Test 8: Compiles without warnings
echo -n "8. No warnings: "
cargo build 2>&1 | grep -q "warning" && echo "❌" || echo "✅"

echo ""
echo "=== Done ==="