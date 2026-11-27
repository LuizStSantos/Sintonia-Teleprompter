#!/usr/bin/env bash
# Sobe um servidor do Sintonia local para testes, com espelhos de exemplo.
# Uso: scripts/sintonia-local.sh <pasta-do-repositório-Sintonia> [porta]
# Login: admin / admin123. Os dados ficam em /tmp/sintonia-teste (apagados a cada execução).
set -euo pipefail
REPO="${1:?Informe a pasta do repositório Sintonia}"
PORTA="${2:-3901}"
DADOS=/tmp/sintonia-teste
rm -rf "$DADOS" && mkdir -p "$DADOS"
(cd "$REPO/server" && npm install --no-audit --no-fund >/dev/null)
(cd "$REPO/server" && DB_PATH="$DADOS/data.db" PORT="$PORTA" node index.js >"$DADOS/servidor.log" 2>&1 &)
for _ in $(seq 1 30); do curl -s "localhost:$PORTA/api/auth/me" >/dev/null && break; sleep 0.5; done
CJ="$DADOS/cookies"
curl -s -c "$CJ" -H 'content-type: application/json' -d '{"username":"admin","password":"admin123"}' "localhost:$PORTA/api/auth/login" >/dev/null
HOJE="$(date +%F)"
curl -s -b "$CJ" -H 'content-type: application/json' "localhost:$PORTA/api/espelho" -d @- >/dev/null <<JSON
{"titulo":"ESPELHO DO PROGRAMA: EXEMPLO","programaNome":"Programa Exemplo","programaSigla":"EX","horarioPrograma":"19:00","data":"$HOJE","itens":[
 {"id":"i1","tipo":"custom","retranca":"ABERTURA","duracao":"0:30","texto":"Boa noite! Começa agora o Programa Exemplo.\n<<\nTrecho em destaque.\n>>\nVamos às notícias."},
 {"id":"i2","tipo":"materia","retranca":"MATÉRIA 1","duracao":"2:00","texto":"Texto da cabeça da primeira matéria."},
 {"id":"i3","tipo":"passagem","duracao":"2:00"},
 {"id":"i4","tipo":"entrevistado","retranca":"ENTR Convidado","duracao":"5:00","texto":"Recebemos agora o nosso convidado."},
 {"id":"i5","tipo":"custom","retranca":"ENCERRAMENTO","duracao":"0:20","texto":"Até amanhã!"}]}
JSON
echo "Sintonia de teste em http://localhost:$PORTA (admin / admin123). Log: $DADOS/servidor.log"
