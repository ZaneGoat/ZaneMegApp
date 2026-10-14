#!/usr/bin/env bash
# Quick launcher for ZaneMegApp (Rust Edition)
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

if [ -f "$DIR/target/release/zanemegapp" ]; then
    exec "$DIR/target/release/zanemegapp" "$@"
elif [ -f "$DIR/target/debug/zanemegapp" ]; then
    exec "$DIR/target/debug/zanemegapp" "$@"
else
    echo "Binary not found, building with cargo..."
    "$HOME/.cargo/bin/cargo" run --release -- "$@"
fi
