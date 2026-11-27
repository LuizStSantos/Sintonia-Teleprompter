# Sintonia Teleprompter — guia para a IA

Teleprompter multiplataforma (Windows, macOS, Linux) em Rust + eframe/egui
0.37, integrado ao Sintonia (sistema de rotina de TV: Node/Express + SQLite,
repositório separado). Público: operador de teleprompter e apresentador de TV
no Brasil — **toda a interface, mensagens e documentação em português do Brasil**.

## Comandos

- `cargo build` / `cargo run` / `cargo test`
- `cargo fmt` e `cargo clippy --all-targets` antes de entregar
- Teste de verdade da interface: siga o agente `qa-teleprompter`
  (Xvfb + xdotool + servidor Sintonia local com `admin`/`admin123`).

## Onde mexer

Ver `docs/ARQUITETURA.md`. Resumo: telas em `src/app.rs`, desenho do texto em
`src/prompter.rs`, roteiro/TXT em `src/script.rs`, API em `src/api.rs`,
identidade visual em `src/theme.rs`.

## Regras que não podem quebrar

1. **Rolagem em "em"** (múltiplos do tamanho da letra), não em pixels: é o que
   faz a prévia e a tela do apresentador mostrarem a mesma linha.
2. **Nada de rede na thread da interface** — sempre `Comando`/`Resposta` (`api.rs`).
3. **Gravar no Sintonia só o que mudou**, buscando o espelho mais recente antes
   (outra pessoa pode estar editando no site).
4. **Senha nunca em disco ou log.** Em `config.json` só servidor e usuário.
5. **Compatibilidade com o Apotec** (teleprompter antigo da equipe): TXT com
   `//` separando laudas, `<<` `>>` para destaque e os mesmos atalhos
   (↑/↓ movem, →/← velocidade, End para, Home início + zera cronômetro,
   rodinha = velocidade). Não mude esses atalhos sem pedir.
6. **Cores só pela `Paleta`** (`theme.rs`), nos temas escuro e claro. A tela do
   apresentador usa as cores escolhidas em Formatação e tudo nela respeita o
   espelhamento (`Espelho`).
7. Fonte nova só pode ser usada no quadro seguinte ao `set_fonts` (`fonts.rs`).
8. Não exigir mudanças no servidor do Sintonia sem combinar com o usuário.
