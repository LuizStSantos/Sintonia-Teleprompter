#!/bin/bash
# Assina o .app do macOS. Com os dados do Apple Developer nas variáveis de
# ambiente, assina com o certificado "Developer ID Application" e envia para
# notarização da Apple; sem eles, faz a assinatura local ("ad-hoc").
#
# Variáveis: MACOS_CERT_P12_BASE64, MACOS_CERT_PASSWORD, MACOS_SIGN_IDENTITY
#            (ex.: "Developer ID Application: Emissora LTDA (ABCDE12345)"),
#            APPLE_ID, APPLE_TEAM_ID, APPLE_APP_PASSWORD (senha de app).
set -euo pipefail
APP="$1"

if [ -z "${MACOS_CERT_P12_BASE64:-}" ]; then
  echo "Sem certificado da Apple: assinatura local (ad-hoc)."
  codesign --force --deep --sign - "$APP"
  exit 0
fi

CHAVEIRO="$RUNNER_TEMP/assinatura.keychain-db"
SENHA_CHAVEIRO="$(openssl rand -base64 24)"
echo "$MACOS_CERT_P12_BASE64" | base64 --decode > "$RUNNER_TEMP/certificado.p12"
security create-keychain -p "$SENHA_CHAVEIRO" "$CHAVEIRO"
security set-keychain-settings -lut 21600 "$CHAVEIRO"
security unlock-keychain -p "$SENHA_CHAVEIRO" "$CHAVEIRO"
security import "$RUNNER_TEMP/certificado.p12" -P "$MACOS_CERT_PASSWORD" -A -t cert -f pkcs12 -k "$CHAVEIRO"
security list-keychain -d user -s "$CHAVEIRO"
rm -f "$RUNNER_TEMP/certificado.p12"

codesign --force --deep --options runtime --timestamp --sign "$MACOS_SIGN_IDENTITY" "$APP"
codesign --verify --deep --strict --verbose=2 "$APP"

ditto -c -k --keepParent "$APP" "$RUNNER_TEMP/notarizar.zip"
xcrun notarytool submit "$RUNNER_TEMP/notarizar.zip" \
  --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" --password "$APPLE_APP_PASSWORD" --wait
xcrun stapler staple "$APP"
echo "Assinado e notarizado: $APP"
