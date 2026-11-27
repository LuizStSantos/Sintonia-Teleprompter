# Como contribuir

## Ambiente

- [Rust](https://rustup.rs) estável (o projeto exige 1.88 ou mais novo).
- Linux: `sudo apt install libxkbcommon-dev libwayland-dev libgl1-mesa-dev`
  (e, para rodar, `libxkbcommon-x11-0 libgl1`).
- Para testar com dados reais: um servidor do Sintonia. O script
  `scripts/sintonia-local.sh <pasta-do-Sintonia>` sobe um de teste com
  espelhos de exemplo (login `admin` / `admin123`).

## Dia a dia

```bash
cargo run                  # roda o aplicativo
cargo test --workspace     # testes de todos os crates
./scripts/verificar.sh     # tudo que o CI verifica: formatação, lint e testes
```

O CI (`.github/workflows/build.yml`) roda as mesmas verificações e gera os
executáveis de Windows, Linux e macOS em todo push. Um pull request só entra
com o CI verde.

## Organização

Veja [docs/ARQUITETURA.md](docs/ARQUITETURA.md). Em resumo:

| Onde | O que vai lá |
|---|---|
| `crates/roteiro` | regras do roteiro, formatos de arquivo, tempos — **sem interface** |
| `crates/sintonia-api` | tudo que fala com o servidor do Sintonia — **sem interface** |
| `crates/teleprompter/src/app` | telas; um arquivo por parte da tela |
| `crates/teleprompter/src/visual` | cores, fontes, desenho do texto |

Regra: lógica nova que dá para testar sem tela vai para `roteiro` ou
`sintonia-api`, com teste.

## Padrões

- **Idioma**: interface, mensagens, comentários e documentação em português do
  Brasil, no mesmo tom do Sintonia.
- **Formatação**: `cargo fmt` (config em `rustfmt.toml`). **Lint**: `cargo clippy`
  sem avisos.
- **Cores** só pela `Paleta` (`visual/tema.rs`), funcionando nos temas escuro e claro.
- **Sem `unsafe`** (proibido no workspace).
- **Segurança**: nunca gravar senha em arquivo ou log — ver [SECURITY.md](SECURITY.md).

## Versões e lançamentos

- Versionamento semântico (`MAJOR.MINOR.PATCH`), definido uma vez só em
  `[workspace.package]` no `Cargo.toml` da raiz.
- Toda mudança visível entra no [CHANGELOG.md](CHANGELOG.md).
- **Para lançar**: suba a versão no `Cargo.toml` da raiz, escreva a seção
  `## X.Y.Z — data` no `CHANGELOG.md` e envie para o `main`. O CI testa, gera os
  pacotes de Windows, macOS e Linux e, como a versão é nova, **cria sozinho a
  tag `vX.Y.Z` e a Release** com os pacotes, o código-fonte (`.zip` e
  `.tar.gz`) e as notas daquela seção do CHANGELOG. Versão repetida não gera
  Release nova.

## Agentes de IA

A pasta `.claude/agents` tem agentes para o Claude Code (desenvolvimento,
integração com o Sintonia, design, testes, release e revisão de texto). As
regras gerais para a IA estão em [CLAUDE.md](CLAUDE.md).
