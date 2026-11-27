#!/bin/bash
# Monta "Sintonia Teleprompter.app" a partir do executável (uso: criar-app.sh <executavel>).
set -euo pipefail
BIN="$1"
RAIZ="$(cd "$(dirname "$0")/../.." && pwd)"
VERSAO="$(grep -m1 '^version' "$RAIZ/Cargo.toml" | cut -d'"' -f2)"
APP="Sintonia Teleprompter.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/sintonia-teleprompter"
chmod +x "$APP/Contents/MacOS/sintonia-teleprompter"

# Ícone .icns a partir do PNG do Sintonia
ICONSET="$(mktemp -d)/sintonia.iconset"
mkdir -p "$ICONSET"
for t in 16 32 64 128 256 512; do
  sips -z $t $t "$RAIZ/assets/sintonia-icone.png" --out "$ICONSET/icon_${t}x${t}.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/sintonia.icns"

sed "s/__VERSAO__/$VERSAO/g" "$RAIZ/packaging/macos/Info.plist" > "$APP/Contents/Info.plist"
echo "Criado: $APP ($VERSAO)"
