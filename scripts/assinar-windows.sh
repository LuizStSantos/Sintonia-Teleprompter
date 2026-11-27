#!/usr/bin/env bash
# Assina um .exe do Windows a partir do Linux, com o certificado .pfx da emissora.
# Requisito: sudo apt install osslsigncode
# Uso: scripts/assinar-windows.sh caminho/do/programa.exe certificado.pfx
#      (a senha do certificado é pedida no terminal; não fica em lugar nenhum)
set -euo pipefail
EXE="${1:?Informe o .exe}"
PFX="${2:?Informe o certificado .pfx}"
read -rsp "Senha do certificado: " SENHA; echo
osslsigncode sign -pkcs12 "$PFX" -pass "$SENHA" \
  -n "Sintonia Teleprompter" -h sha256 -t http://timestamp.digicert.com \
  -in "$EXE" -out "$EXE.assinado"
mv "$EXE.assinado" "$EXE"
osslsigncode verify "$EXE" | tail -3
