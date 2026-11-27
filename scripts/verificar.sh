#!/usr/bin/env bash
# Mesmas verificações do CI: formatação, lint e testes de todos os crates.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
echo "Tudo certo."
