#!/bin/bash

echo "🔥 Installing Firework CLI (fwk)..."

cd "$(dirname "$0")"

cargo build --release -p fwk

if [ $? -eq 0 ]; then
    echo "✓ Build successful!"
    echo ""
    echo "To use fwk globally, you can:"
    echo "  1. Copy to your PATH:"
    echo "     sudo cp target/release/fwk /usr/local/bin/"
    echo ""
    echo "  2. Or add an alias to your shell config:"
    echo "     alias fwk='$(pwd)/target/release/fwk'"
    echo ""
    echo "  3. Or use cargo install:"
    echo "     cargo install --path fwk"
    echo ""
    echo "Quick start:"
    echo "  ./target/release/fwk new my-project"
    echo "  cd my-project"
    echo "  ../target/release/fwk run dev"
else
    echo "✗ Build failed!"
    exit 1
fi
