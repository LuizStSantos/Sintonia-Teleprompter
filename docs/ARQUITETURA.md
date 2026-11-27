# Arquitetura

```
src/
  main.rs      janela principal (tamanho, ícone) e início do app
  app.rs       estado e telas: login, operador (menu, roteiro, formatação,
               prévia, transporte/cronômetro), apresentador, seletor de
               espelho, confirmações e avisos; atalhos de teclado e mouse
  prompter.rs  desenho do texto rolando (usado na prévia e no apresentador),
               seta de leitura e espelhamento (horizontal/vertical)
  script.rs    roteiro: Espelho do Sintonia → blocos; TXT (# títulos,
               // laudas, << >> destaque) ida e volta; Latin-1 → UTF-8
  api.rs       cliente HTTP do Sintonia numa thread separada (login por
               cookie, listar espelhos, gravar textos alterados)
  theme.rs     identidade visual do Sintonia: paleta escura/clara, fontes
               embutidas (IBM Plex Sans/Mono, Space Grotesk), ícone
  fonts.rs     fontes instaladas no sistema (fontdb) carregadas sob demanda
  settings.rs  preferências salvas em config.json (pasta de config do SO)
  timer.rs     cronômetro, formatação de tempo, leitura de durações
assets/        fontes (OFL) e ícone do Sintonia
packaging/     .app do macOS, .desktop/instalador do Linux
.github/       CI: testes + builds Windows/macOS/Linux; release nas tags v*
.claude/agents agentes de IA do projeto
```

## Fluxo

1. **Login** → `Comando::Login` → `POST /api/auth/login`. O cliente HTTP guarda
   o cookie `controle_tv_token` (JWT 12h) como um navegador faria.
2. Com permissão de ver o Espelho, o app pede `GET /api/espelho` e abre o
   seletor já filtrado na data de hoje.
3. **Abrir espelho** → `Roteiro::do_espelho`: cada item vira um `Segmento`
   (`materia`, `entrevistado`, `custom` → texto; `passagem` → intervalo). A soma
   das durações vira o "tempo do programa".
4. **Desenho**: `prompter::desenhar` diagrama cada bloco com a fonte/tamanho
   escolhidos e devolve `Medidas` (onde começa cada bloco, em "em"). A prévia
   usa a mesma função com `escala = largura da prévia / largura do apresentador`.
5. **Rolagem**: `rolagem_em += velocidade × 0,12 × dt` a cada quadro.
   Trocar fonte/tamanho/margem reposiciona a rolagem no mesmo ponto do bloco
   atual (assinatura de layout em `previa`).
6. **Gravar no Sintonia** → `Comando::SalvarTextos`: busca o espelho atual no
   servidor, troca só o `texto` dos itens alterados e faz `PUT /api/espelho/:id`.
   O servidor registra a alteração no Histórico.

## Espelhamento

O egui não desenha texto invertido. `Espelho::texto` transforma o texto em
triângulos com o `Tessellator` do epaint e inverte as coordenadas dos vértices
(o que a placa de vídeo faria). A seta de leitura e o relógio do apresentador
passam pela mesma transformação.

## Ideias para as próximas versões

- Atualização em tempo real do espelho pelo WebSocket do Sintonia (`/ws`),
  com aviso quando outra pessoa mudar o espelho aberto.
- Controle por pedal/controle remoto USB (teclas configuráveis).
- Impressão das laudas (o Apotec tinha).
- Instaladores assinados (`.msi` e `.dmg` notarizado).
