# Arquitetura

O Sintonia Teleprompter é um **workspace Rust** com três crates, cada um com
uma responsabilidade. As dependências vão sempre "para baixo": o aplicativo
usa os outros dois; a lógica do roteiro não depende de nada de interface.

```
sintonia-teleprompter/
├── Cargo.toml                 workspace: versão única, dependências e lints centralizados
├── crates/
│   ├── roteiro/               sintonia-roteiro — lógica pura, sem interface
│   │   └── src/
│   │       ├── lib.rs         API pública do crate
│   │       ├── roteiro.rs     Roteiro, Segmento, Tipo; Espelho → blocos; TXT ↔ blocos
│   │       ├── txt.rs         formato TXT: laudas "//" (Apotec), UTF-8 / Windows-1252
│   │       ├── destaque.rs    trechos em destaque entre << e >>
│   │       └── tempo.rs       Cronometro, fmt_tempo, parse_duracao
│   │
│   ├── sintonia-api/          sintonia-api — cliente do servidor do Sintonia
│   │   └── src/
│   │       ├── lib.rs         Api (thread de rede), Comando/Resposta, login, espelho, teste de conexão
│   │       ├── usuario.rs     Usuario e permissões (mesmas regras do servidor)
│   │       └── cofre.rs       senha no cofre do sistema (Windows/macOS/Linux)
│   │
│   └── teleprompter/          sintonia-teleprompter — o aplicativo (binário)
│       ├── assets/            ícones (PNG/.ico, gerados do SVG) e fontes do Sintonia (OFL)
│       ├── build.rs           embute ícone e versão no .exe do Windows
│       └── src/
│           ├── main.rs        janela principal
│           ├── config.rs      preferências e predefinições (config.json)
│           ├── visual/        identidade visual e desenho
│           │   ├── tema.rs        paleta escura/clara e fontes do Sintonia
│           │   ├── fontes.rs      fontes instaladas no sistema
│           │   ├── icones.rs      ícones desenhados (play, pausa, setas…) e botões com ícone
│           │   └── prompter.rs    texto rolando + espelhamento
│           └── app/           telas — o estado fica em `App` (mod.rs); um arquivo por parte
│               ├── mod.rs             App, inicialização, tela principal
│               ├── login.rs           tela de entrada e EstadoLogin
│               ├── rede.rs            respostas do servidor, renovação da sessão
│               ├── documento.rs       abrir/salvar roteiro (espelho, TXT)
│               ├── controle.rs        rolagem, velocidade, passar rápido, teclado e mouse
│               ├── cronometros.rs     cronômetros e tela só cronômetro
│               ├── menu.rs            barra de cima
│               ├── transporte.rs      barra de baixo
│               ├── painel_roteiro.rs  lista de blocos e edição
│               ├── formatacao.rs      formatação e predefinições
│               ├── previa.rs          prévia da tela do apresentador
│               ├── apresentador.rs    janela do apresentador, escolha do monitor, F11
│               ├── seletor_espelho.rs escolha do espelho
│               └── dialogos.rs        confirmações e avisos
├── packaging/                 .app (macOS), .desktop/instalador (Linux), LEIA-ME (Windows)
├── scripts/                   verificar.sh, build-windows.sh, sintonia-local.sh
├── docs/                      esta documentação e prints
└── .github/                   CI, modelos de issue/PR, dependabot
```

## Por que assim

- **`sintonia-roteiro`** não sabe que existe tela: é testado com `cargo test`
  sem abrir janela, e qualquer mudança no formato do TXT ou na leitura do
  espelho fica isolada ali.
- **`sintonia-api`** fala com o Sintonia numa thread própria e avisa a interface
  por um callback (não depende do egui). Pode ser reaproveitado por outra
  ferramenta da emissora.
- **`sintonia-teleprompter`** só cuida da interface. Em `app/`, `App` tem todo o
  estado e cada arquivo implementa uma parte dele (`impl App` em vários
  arquivos); os métodos são `pub(super)`, visíveis só dentro de `app/`.

## Fluxo

1. **Entrada** (`app/login.rs`): enquanto o operador digita o endereço, o app
   testa o servidor (`Comando::TestarServidor` → `GET /api/auth/me`). O login é
   `POST /api/auth/login`; o cliente HTTP guarda o cookie `controle_tv_token`
   (JWT de 12 h) como um navegador. Erros trazem tentativas restantes (401) e
   tempo de bloqueio (429).
2. **Sessão** (`app/rede.rs`): a senha da sessão fica só na memória. Se o
   servidor responder 401 (sessão expirou), o app faz login de novo sozinho e
   repete a última operação (`Api::repetir_ultimo`).
3. **Espelho**: `GET /api/espelho` → seletor por dia/programa →
   `Roteiro::do_espelho` (cada item vira um bloco). A soma das durações vira o
   tempo do programa.
4. **Desenho**: `visual::prompter::desenhar` diagrama cada bloco e devolve
   `Medidas` (início de cada bloco, em "em"). A prévia usa a mesma função com
   `escala = largura da prévia / largura do apresentador`, então a quebra de
   linhas é idêntica nas duas telas.
5. **Rolagem**: `rolagem_em += (velocidade/100)^1,5 × 8 × dt` a cada quadro
   (100 % = 8 em/s), com `dt` medido no relógio real; "passar rápido" usa
   20 em/s enquanto o botão é segurado. Só o texto do teleprompter é posicionado
   em frações de pixel (malha própria em `Espelho::texto`), para não andar "em
   degraus"; o resto da interface fica alinhado ao pixel, nítido.
6. **Gravar no Sintonia**: busca o espelho atual, troca só o `texto` dos itens
   alterados e faz `PUT /api/espelho/:id` (o servidor registra no Histórico).

## Espelhamento

O egui não desenha texto invertido. `Espelho::texto` transforma o texto em
triângulos com o `Tessellator` do epaint e inverte as coordenadas dos vértices.
Os triângulos de cada bloco ficam guardados entre quadros (só são refeitos
quando o texto ou a formatação mudam).

## Desempenho

- Renderizador **OpenGL** (glow) por padrão — mais leve em PCs simples e
  placas integradas; o wgpu ("Moderno") é opcional (`config.rs → Renderizador`).
- A tela só redesenha sem parar enquanto o texto rola; parada, redesenha 1×/s
  (relógio) ou 4×/s (cronômetro rodando).
A seta de leitura, o relógio e a tela só cronômetro passam pela mesma transformação.

## Segurança

Ver [SECURITY.md](../SECURITY.md): senha só no cofre do sistema (opcional) e
na memória durante a sessão; nada de senha em arquivo ou log.

## Próximos passos

- Atualização em tempo real do espelho pelo WebSocket do Sintonia (`/ws`).
- Controle por pedal/controle remoto USB (teclas configuráveis).
- Instaladores (`.msi` e `.dmg`); a assinatura já está pronta no CI (docs/ASSINATURA.md).
- Registro de eventos (log) em arquivo para suporte.
