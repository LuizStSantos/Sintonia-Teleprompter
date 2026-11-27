#!/usr/bin/env bash
# Gera o .exe do Windows a partir do Linux (compilação cruzada) e empacota em .zip.
# Requisitos (Debian/Ubuntu): sudo apt install gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64 cmake nasm
#                             rustup target add x86_64-pc-windows-gnu
set -euo pipefail
cd "$(dirname "$0")/.."
VERSAO="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
cargo build --release --locked --target x86_64-pc-windows-gnu
PASTA="dist/Sintonia Teleprompter"
rm -rf dist && mkdir -p "$PASTA"
cp target/x86_64-pc-windows-gnu/release/sintonia-teleprompter.exe "$PASTA/Sintonia Teleprompter.exe"
cp packaging/windows/LEIA-ME.txt "$PASTA/"
(cd dist && python3 -c "
import os, zipfile, sys
with zipfile.ZipFile(sys.argv[1], 'w', zipfile.ZIP_DEFLATED) as z:
    for f in os.listdir('Sintonia Teleprompter'):
        z.write(os.path.join('Sintonia Teleprompter', f))
" "sintonia-teleprompter-$VERSAO-windows-x64.zip")
rm -rf "$PASTA"
echo "Pronto: dist/sintonia-teleprompter-$VERSAO-windows-x64.zip"
