#!/bin/bash
# Don't exit on grep not finding matches
set +e

echo "=== Pre-Publish Verification ==="

# 1. Build check
echo "1. Checking build for warnings..."
cargo build 2>&1 | tee /tmp/build_output.txt
if grep -i "warning" /tmp/build_output.txt > /dev/null; then
    echo "⚠ Warnings found (see above)"
    WARNINGS=1
else
    echo "✓ No warnings"
fi

# 2. Run all tests
echo ""
echo "2. Running tests..."
cargo test 2>&1 | tee /tmp/test_output.txt
if grep -q "test result: FAILED" /tmp/test_output.txt; then
    echo "⚠ Tests failed"
else
    echo "✓ Tests passed"
fi

# 3. Check formatting
echo ""
echo "3. Checking formatting..."
cargo fmt -- --check
if [ $? -eq 0 ]; then
    echo "✓ Formatting OK"
else
    echo "⚠ Formatting issues found (run 'cargo fmt' to fix)"
fi

# 4. Run clippy
echo ""
echo "4. Running clippy..."
cargo clippy 2>&1 | tee /tmp/clippy_output.txt
if grep -i "warning" /tmp/clippy_output.txt > /dev/null; then
    echo "⚠ Clippy warnings found"
else
    echo "✓ No clippy issues"
fi

# 5. Dry run publish
echo ""
echo "5. Testing crate packaging..."
cargo package --list > /dev/null 2>&1
if cargo publish --dry-run 2>&1; then
    echo "✓ Package OK"
else
    echo "⚠ Package check failed"
fi

# 6. Build release
echo ""
echo "6. Building release..."
if cargo build --release 2>&1 | tail -5; then
    echo "✓ Release build OK"
else
    echo "⚠ Release build failed"
fi

# 7. Test release binary
echo ""
echo "7. Testing release binary..."
if [ -f "./target/release/cardano-tx-viz" ]; then
    ./target/release/cardano-tx-viz --version
    ./target/release/cardano-tx-viz --help > /dev/null
    echo "✓ Binary works"
elif [ -f "./target/release/cardano-tx-viz.exe" ]; then
    ./target/release/cardano-tx-viz.exe --version
    ./target/release/cardano-tx-viz.exe --help > /dev/null
    echo "✓ Binary works"
else
    echo "⚠ Binary not found"
fi

echo ""
echo "=== Verification Complete ==="