# Changelog

Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/); versões
seguem o [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## 0.5.1 — 2026-11-27
- A prévia usa o tamanho e a orientação (inclusive retrato) do monitor escolhido
  para o apresentador, antes mesmo de abrir; a janela já abre nesse tamanho.
- Espelhamento à vista: botão "Espelhar" em cima da prévia e no menu Apresentador;
  opção "ver espelhado" para a prévia mostrar o que vai para o vidro.
- Prints do README atualizados (galeria com todas as telas).

## 0.5.0 — 2026-11-27
### Telas
- Ícones desenhados (play, pausa, voltar, avançar, zerar, cronômetro…) no lugar
  dos símbolos do sistema; etiquetas MAT/ENT/INT/MAN/TXT na lista de blocos.
- Setinhas para recolher a barra de baixo e o painel do roteiro.
- Barra de baixo mais enxuta sozinha em telas baixas (1366 × 768).
- F11 deixa a janela do operador em tela cheia.
- Tela do apresentador: escolha o monitor pelo nome/resolução e ela já abre em
  tela cheia nele; botão "Abrir apresentador" acima da prévia.
- Títulos longos na lista cortados com "…".
### Desempenho
- Renderizador OpenGL por padrão (mais leve); opção "Moderno" em Formatação.
- Rolagem pelo relógio real e em frações de pixel (sem "degraus").
- Texto espelhado reaproveitado entre quadros, em vez de recalculado.
### Executável
- Ícone do Sintonia e dados do programa embutidos no .exe; .icns em alta resolução.
- Assinatura digital automática no CI quando os certificados estiverem nos
  Secrets (ver docs/ASSINATURA.md). Corrige o arquivo do CI (YAML inválido).

## 0.4.0 — 2026-11-27
### Entrada (login)
- Novo visual no padrão do Sintonia, com campos maiores e faixa vermelha no cartão.
- Situação do servidor enquanto o endereço é digitado ("Conectado" / "Sem conexão").
- Mostrar/ocultar a senha; foco já no campo da senha.
- Tentativas restantes antes do bloqueio e contagem regressiva do bloqueio de 15 min.
- "Entrar automaticamente ao abrir" (com a senha lembrada no cofre do sistema).
- Renovação automática da sessão do Sintonia (12 h) no meio do uso, repetindo a
  última operação.
### Projeto
- Workspace com três crates: `sintonia-roteiro` (lógica), `sintonia-api` (cliente
  do Sintonia) e `sintonia-teleprompter` (aplicativo); telas divididas em arquivos.
- CI com formatação, lint (`-D warnings`) e testes antes dos builds; dependabot;
  modelos de issue e pull request.
- `CONTRIBUTING.md`, `SECURITY.md`, `rustfmt.toml`, `.editorconfig` e scripts
  (`verificar.sh`, `build-windows.sh`, `sintonia-local.sh`).

## 0.3.0 — 2026-11-27
- Cronômetro com tempo: quando estoura, a tela pisca em vermelho e mostra o tempo a mais (+00:15).
- Velocidade: rodinha e setas mudam 5 % (Shift: 1 %), botões Lenta/Normal/Rápida/Máx.
  e curva com controle fino nas velocidades de leitura.
- "Passar rápido": segurar ⏩/⏪, o botão direito do mouse ou Shift + ↓/↑.
- Barra de baixo reorganizada em blocos (Rolagem, Velocidade, Programa, Entrevista,
  Agora) e barra de progresso clicável com a marca de cada bloco.
- Novo seletor de espelho: dia com ◀ ▶, Hoje/Amanhã, filtro por programa, itens
  do espelho antes de abrir e atalho para os dias que têm espelho.

## 0.2.0 — 2026-11-27
- Painel de Formatação escondido; abre pelo botão "Formatação" no menu.
- Predefinições com nome (fonte, tamanho, cores, monitor, tela cheia...).
- Cronômetro do apresentador desligado por padrão ("Mostrar ao apresentador").
- Tela só cronômetro para entrevistas (F7), com cronômetro próprio (F6).
- Mouse: arrastar rola o texto na hora; um clique roda/para.
- Lembrar usuário e senha (senha no cofre do sistema).
- Cronômetro da entrevista "Só rodando" (conta para cima) ou "Com tempo" (regressivo).
- "Usar só o cronômetro" direto da tela de entrada.
- Velocidade de 0 a 100 %, com máximo cerca de 4 vezes mais rápido que antes.

## 0.1.0 — 2026-11-27
Primeira versão.
- Login com usuário e senha do Sintonia; abre o Espelho do dia.
- Tela do operador (roteiro, edição, formatação, prévia) e tela do apresentador
  (tela cheia, escolha de monitor, espelhamento horizontal/vertical).
- Gravar no Espelho os textos corrigidos no teleprompter.
- TXT: `#` títulos, `//` laudas e `<<` `>>` destaque (compatível com o Apotec).
- Formatação: fonte (Sintonia ou do sistema), negrito, tamanho, entrelinha,
  margem, alinhamento, CAIXA ALTA, cores, seta de leitura.
- Cronômetro automático, tempo do programa, estimativa de leitura, relógio.
- Identidade visual do Sintonia, temas escuro e claro.
- Builds para Windows, macOS (universal) e Linux no GitHub Actions.
