#!/bin/bash
# Instala o Sintonia Teleprompter para o usuário atual (sem precisar de root).
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p ~/.local/bin ~/.local/share/applications ~/.local/share/icons/hicolor/128x128/apps
install -m755 sintonia-teleprompter ~/.local/bin/
install -m644 sintonia-icone.png ~/.local/share/icons/hicolor/128x128/apps/sintonia-teleprompter.png
install -m644 sintonia-teleprompter.desktop ~/.local/share/applications/
echo "Instalado. Procure por \"Sintonia Teleprompter\" no menu de aplicativos."
echo "Se não abrir, instale: sudo apt install libxkbcommon-x11-0 libgl1"
