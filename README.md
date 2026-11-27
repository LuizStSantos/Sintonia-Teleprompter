# Sintonia Teleprompter

Teleprompter para **Windows, macOS e Linux**, feito em **Rust**, com a cara do
**Sintonia** e integrado a ele: entra com o mesmo usuário e senha do sistema e
abre direto o **Espelho** do programa — cada item do espelho vira um bloco de
leitura (matérias, entrevistados, itens manuais e as marcações de intervalo).
Também abre e salva **TXT**, inclusive os arquivos do teleprompter antigo
(Apotec), com `//` separando laudas e `<<` `>>` marcando destaque.

![Tela do operador](docs/tela-operador.png)

## Como funciona

- **Duas telas**: a do **operador** (roteiro, edição, formatação, prévia e
  controles) e a do **apresentador** (só o texto, grande, que vai no monitor
  do teleprompter — pode ser espelhada para o vidro).
- **Abrir espelho**: escolha o dia com ◀ ▶ (ou Hoje / Amanhã), filtre pelo
  programa e veja os itens do espelho antes de abrir. Num dia sem espelho, o
  programa mostra os dias mais próximos que têm.
- **Login do Sintonia**: as mesmas permissões do site valem aqui — quem só tem
  "Visualizar" no Espelho lê; quem tem "Editar" pode gravar no espelho os
  textos corrigidos no teleprompter.
- **Edição na hora**: o texto pode ser corrigido durante o programa; a tela do
  apresentador atualiza na mesma hora. "Sintonia → Gravar textos alterados no
  espelho" manda as correções de volta (aparece no Histórico do Sintonia).
- **Formatação**: fonte (as do Sintonia ou qualquer fonte instalada no
  computador), negrito, tamanho, espaço entre linhas, margem, alinhamento,
  CAIXA ALTA, cores do texto/fundo/títulos/destaque, seta de leitura.
- **Velocidade** de 0 a 100 %, com botões prontos (Lenta, Normal, Rápida,
  Máx.). Para **passar rápido** um trecho, segure o botão de avançar (ou o de voltar), o
  botão direito do mouse sobre o texto, ou Shift + ↓. A barra vermelha acima
  dos controles mostra onde está o texto; clique nela para ir direto.
- **Formatação** fica escondida para a prévia ter mais espaço: abra pelo
  botão **Formatação** no menu de cima.
- **Predefinições**: salve a formatação com um nome (ex.: "Jornal da Noite") —
  fonte, tamanho, cores, espelhamento, velocidade, **monitor e tela cheia do
  apresentador** — e troque tudo de uma vez pelo menu **Predefinições**.
- **Cronômetro**: dispara sozinho quando o texto começa a rolar (Home zera),
  contagem regressiva do **tempo do programa** (preenchido sozinho com a soma
  das durações do espelho), **estimativa do tempo de leitura** que falta e
  relógio. Para o apresentador ver o cronômetro no canto da tela dele, marque
  **Mostrar ao apresentador** (vem desligado).
- **Tela só cronômetro** (entrevistas): troca o texto do apresentador por um
  cronômetro grande no meio da tela, com cronômetro próprio, de dois jeitos:
  **Só rodando** (conta para cima, sem limite) ou **Com tempo** (regressivo a
  partir da duração — preenchida sozinha quando o bloco atual é um entrevistado
  do espelho; fica verde, âmbar no último minuto e, quando estoura, a tela
  pisca em vermelho mostrando o tempo a mais, ex.: +00:15).
- **Só o cronômetro**: na tela de entrada, "Usar só o cronômetro" abre direto
  a tela do apresentador com o cronômetro, sem login e sem texto.
- **Entrada**: mostra se o servidor do Sintonia está conectado enquanto você
  digita o endereço, permite ver a senha digitada, avisa quantas tentativas
  restam e quanto tempo falta se o Sintonia bloquear o usuário. **Lembrar
  usuário e senha** guarda a senha no cofre do sistema (Gerenciador de
  Credenciais do Windows, Chaveiro do Mac), nunca em arquivo; com **Entrar
  automaticamente**, o programa já abre logado.
- **Sessão sem susto**: a sessão do Sintonia dura 12 horas. Se ela vencer no
  meio do programa, o teleprompter entra de novo sozinho e repete o que estava
  fazendo — o operador não sai da tela.
- **Tema escuro e claro**, iguais aos do Sintonia.
- **Telas pequenas** (ex.: 1366 × 768): a barra de baixo fica mais enxuta
  sozinha; as **setinhas** recolhem a barra de baixo (fica só o essencial numa
  linha) e o painel do roteiro (ficam só os números dos blocos). **F11** deixa
  a janela do operador em tela cheia.
- **Computadores simples**: o padrão é o renderizador **OpenGL**, mais leve; dá
  para trocar em Formatação → Desempenho.

## Telas

Prints da versão atual, numa tela de 1366 × 768.

| | |
|---|---|
| ![Entrada](docs/tela-login.png) **Entrada** — mesmo login do Sintonia, com situação do servidor | ![Abrir espelho](docs/tela-seletor-espelho.png) **Abrir espelho** — escolha o dia, o programa e veja os itens antes |
| ![Operador](docs/tela-operador.png) **Operador** — roteiro, prévia e controles | ![Barra recolhida](docs/tela-recolhida.png) **Recolhido** — setinhas deixam a prévia quase na tela toda |
| ![Apresentador espelhado](docs/tela-apresentador-espelhada.png) **Apresentador** — texto espelhado para o vidro | ![Só cronômetro](docs/tela-so-cronometro.png) **Só cronômetro** — tempo de entrevista no meio da tela |
| ![Escolha do monitor](docs/tela-monitores.png) **Apresentador** — escolha o monitor e o espelhamento | ![Tema claro](docs/tela-tema-claro.png) **Tema claro** — igual à opção do Sintonia |

## Atalhos (os mesmos do Apotec)

| Tecla / mouse | O que faz |
|---|---|
| Espaço · clique no texto | rolar / parar |
| End · clique na rodinha | para a rolagem |
| ↑ / ↓ (segurar) · arrastar o texto | rola o texto na mão |
| → / ← · rodinha do mouse | velocidade +5 % / −5 % (com Shift, 1 %; vai de 0 a 100 %) |
| Segurar avançar / voltar · segurar o botão direito no texto · Shift + ↓ / ↑ | passa rápido (para a frente / para trás) |
| Home | volta ao começo e zera o cronômetro |
| Page Up / Page Down | lauda (bloco) anterior / próxima |
| F5 | inicia / pausa o cronômetro |
| F6 | inicia / pausa o cronômetro da entrevista |
| F7 | tela só cronômetro (liga / desliga) |
| F11 / Esc | tela cheia do apresentador (liga / desliga), com a janela dele ativa |
| F11 | tela cheia da janela do operador |

## Formato do TXT

```
# ABERTURA
Boa noite! Começa agora o Sucesso no Campo.
<<
Este trecho aparece na cor de destaque.
>>
# SAFRA DE SOJA
A colheita da soja avança no estado...
```

- `# TÍTULO` abre um bloco novo com aquele título (retranca).
- `//` também separa blocos (laudas), como no Apotec — os blocos sem título
  viram "LAUDA 1", "LAUDA 2"... (`http://` não conta como separador).
- `<<` e `>>` marcam texto em destaque.
- Arquivos em UTF-8 ou no formato antigo do Windows (ANSI/Latin-1) abrem com
  os acentos certos. Ao salvar, o app grava em UTF-8 usando `#`.

## Uso com dois monitores

1. Ligue o monitor do teleprompter como **tela estendida**.
2. No menu **Apresentador**, clique no monitor do teleprompter (o programa
   mostra o nome e a resolução de cada um): a tela do apresentador já abre
   **em tela cheia nele**. O botão **Abrir apresentador**, acima da prévia,
   abre direto no monitor que não é o do operador.
3. Se o monitor fica atrás de um vidro, clique em **Espelhar** (em cima da
   prévia ou no menu Apresentador). Marque **ver espelhado** para a prévia
   mostrar exatamente o que vai para o vidro.

A prévia já assume o **tamanho e a orientação do monitor escolhido** — inclusive
monitor em pé (retrato) — antes mesmo de abrir a tela do apresentador, então a
quebra de linhas na prévia é a mesma do teleprompter.

A prévia na tela do operador mostra exatamente a mesma quebra de linhas da
tela do apresentador (mesma proporção), sem espelhar.

## Instalação

Baixe o pacote do seu sistema na página de **[Releases](https://github.com/LuizStSantos/Sintonia-Teleprompter/releases)**
(publicada automaticamente pelo GitHub a cada versão, junto com o código-fonte):

- **Windows**: `sintonia-teleprompter-windows-x64.zip` → descompacte e abra
  `Sintonia Teleprompter.exe`.
- **macOS** (Intel e Apple Silicon): `sintonia-teleprompter-macos-universal.zip`
  → arraste o app para Aplicativos. Na primeira vez, clique com o botão
  direito → Abrir (o app ainda não é assinado pela Apple).
- **Linux**: `sintonia-teleprompter-linux-x64.tar.gz` → descompacte e rode
  `./instalar.sh`. Precisa de `libxkbcommon-x11-0` e `libgl1`
  (`sudo apt install libxkbcommon-x11-0 libgl1`).

Na primeira tela, informe o endereço do servidor do Sintonia (o mesmo do
navegador, ex.: `http://servidor:3001`), usuário e senha. O endereço e o
usuário ficam salvos; a senha só no cofre do sistema, se você pedir.


## Desenvolvimento

Precisa do [Rust](https://rustup.rs) (stable). No Linux, também:
`sudo apt install libxkbcommon-dev libwayland-dev libgl1-mesa-dev`.

```bash
cargo run                  # roda o aplicativo
./scripts/verificar.sh     # formatação, lint e testes (o mesmo que o CI)
cargo build --release      # executável final em target/release/
./scripts/build-windows.sh # .exe do Windows a partir do Linux
```

O projeto é um workspace Rust com três crates:

| Crate | Pasta | Responsabilidade |
|---|---|---|
| `sintonia-roteiro` | `crates/roteiro` | roteiro, TXT, destaque e tempos — lógica pura, sem interface |
| `sintonia-api` | `crates/sintonia-api` | cliente do Sintonia: login, espelho, cofre de senha |
| `sintonia-teleprompter` | `crates/teleprompter` | o aplicativo: telas, visual, configurações |

Detalhes em [docs/ARQUITETURA.md](docs/ARQUITETURA.md); padrões e como lançar
uma versão em [CONTRIBUTING.md](CONTRIBUTING.md); senhas e sessão em
[SECURITY.md](SECURITY.md); assinatura digital do `.exe`/`.app` em
[docs/ASSINATURA.md](docs/ASSINATURA.md).

## Agentes de IA do projeto

A pasta [`.claude/agents`](.claude/agents) tem agentes prontos para usar com o
Claude Code neste repositório:

| Agente | Para quê |
|---|---|
| `dev-rust` | implementar funcionalidades e corrigir bugs no código Rust |
| `integracao-sintonia` | tudo que envolve a API do Sintonia (login, espelho, permissões) |
| `design-sintonia` | manter a identidade visual do Sintonia nas telas |
| `qa-teleprompter` | testar de verdade: sobe um Sintonia local, abre o app e tira prints |
| `release-multiplataforma` | builds de Windows/macOS/Linux, pacotes e versões |
| `revisor-lauda` | revisar textos para leitura no ar (ritmo, números, siglas, tempo) |

Regras gerais do projeto para a IA: [`CLAUDE.md`](CLAUDE.md). Organização do
código: [`docs/ARQUITETURA.md`](docs/ARQUITETURA.md).

## Fontes e ícone

As fontes IBM Plex Sans, IBM Plex Mono e Space Grotesk (as mesmas do site do
Sintonia) vão embutidas no programa, sob a licença SIL Open Font License
(`crates/teleprompter/assets/fonts/OFL-*.txt`). O ícone é o mesmo do Sintonia.
