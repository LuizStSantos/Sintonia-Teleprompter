# Sintonia Teleprompter — guia para a IA

Teleprompter multiplataforma (Windows, macOS, Linux) em Rust + eframe/egui
0.37, integrado ao Sintonia (sistema de rotina de TV: Node/Express + SQLite,
repositório separado). Público: operador de teleprompter e apresentador de TV
no Brasil — **toda a interface, mensagens e documentação em português do Brasil**.

## Estrutura (workspace)

- `crates/roteiro` — lógica pura (roteiro, TXT, destaque, tempos). Sem egui.
- `crates/sintonia-api` — cliente do Sintonia (login, espelho, cofre). Sem egui.
- `crates/teleprompter` — o aplicativo: `src/app/` (uma tela/parte por arquivo),
  `src/visual/` (tema, fontes, desenho), `src/config.rs`.

Detalhes e fluxo: `docs/ARQUITETURA.md`. Como contribuir: `CONTRIBUTING.md`.

## Comandos

- `cargo run` — roda o app · `cargo test --workspace` — testes
- `./scripts/verificar.sh` — o mesmo que o CI (fmt, clippy `-D warnings`, testes). Rode antes de entregar.
- `./scripts/build-windows.sh` — `.exe` do Windows a partir do Linux.
- `./scripts/sintonia-local.sh <repo Sintonia>` — servidor de teste (`admin`/`admin123`).
- Teste de interface: agente `qa-teleprompter` (Xvfb + xdotool).

## Regras que não podem quebrar

1. **Rolagem em "em"** (múltiplos do tamanho da letra), não em pixels: é o que
   faz a prévia e a tela do apresentador mostrarem a mesma linha.
2. **Nada de rede na thread da interface** — sempre `Comando`/`Resposta` (`sintonia-api`).
3. **Dependências só para baixo**: `roteiro` e `sintonia-api` nunca dependem de egui
   nem do app.
4. **Gravar no Sintonia só o que mudou**, buscando o espelho mais recente antes.
5. **Senha nunca em arquivo ou log.** Só no cofre do sistema (se "lembrar") e na
   memória durante a sessão. Em `config.json` só servidor e usuário.
6. **Compatibilidade com o Apotec** (teleprompter antigo da equipe): TXT com `//`
   separando laudas, `<<` `>>` para destaque e os mesmos atalhos (↑/↓ movem, →/←
   velocidade, End para, Home início + zera cronômetro, rodinha = velocidade).
7. **Cores só pela `Paleta`** (`visual/tema.rs`), temas escuro e claro. Tudo na
   tela do apresentador respeita o espelhamento (`Espelho`).
8. Fonte nova só pode ser usada no quadro seguinte ao `set_fonts` (`visual/fontes.rs`).
9. Não exigir mudanças no servidor do Sintonia sem combinar com o usuário.
10. Toda mudança de uso: `CHANGELOG.md` + `README.md`. Versão única no `Cargo.toml` da raiz.
