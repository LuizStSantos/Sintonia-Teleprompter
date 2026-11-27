#!/usr/bin/env bash
# Regera os ícones (PNG e .ico) a partir de crates/teleprompter/assets/favicon.svg.
# Requisitos: Node com Playwright (Chromium) e Python com Pillow.
set -euo pipefail
cd "$(dirname "$0")/.."
A=crates/teleprompter/assets
node scripts/renderizar-svg.js "$A/favicon.svg" "$A/icones/icone-1024.png"
python3 - "$A/icones" <<'PY'
import sys
from PIL import Image
pasta = sys.argv[1]
im = Image.open(f"{pasta}/icone-1024.png").convert("RGBA")
for t in (16, 24, 32, 48, 64, 128, 256, 512):
    im.resize((t, t), Image.LANCZOS).save(f"{pasta}/icone-{t}.png")
im.save(f"{pasta}/sintonia.ico", sizes=[(s, s) for s in (16, 24, 32, 48, 64, 128, 256)])
PY
echo "Ícones atualizados em $A/icones"
