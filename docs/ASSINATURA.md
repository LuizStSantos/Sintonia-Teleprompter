# Assinatura digital (Windows e macOS)

Sem assinatura, o programa funciona, mas o sistema avisa na primeira vez:

- **Windows**: "O Windows protegeu o computador" (SmartScreen) → *Mais informações* → *Executar assim mesmo*.
- **macOS**: "não pode ser aberto porque o desenvolvedor não pôde ser verificado" → botão direito → *Abrir*.

Para isso sumir, o executável precisa ser assinado com um **certificado em nome
da emissora** (ou da pessoa responsável). O certificado é comprado de uma
autoridade certificadora e **nunca entra no código**: fica guardado nos
*Secrets* do repositório no GitHub, e o CI assina sozinho a cada versão.

O CI já está pronto (`.github/workflows/build.yml`): **sem os Secrets, ele gera
os pacotes sem assinatura, como hoje; com eles, assina automaticamente.**

## Windows (Authenticode)

1. Compre um **certificado de assinatura de código** ("Code Signing") de uma
   autoridade certificadora (ex.: Certum, Sectigo, DigiCert, GlobalSign). Há
   opções para empresas (validação da empresa) e para pessoas físicas, com
   preços e exigências diferentes; confira os valores atuais no site de cada uma.
   - Desde 2023 a maioria desses certificados vem num token USB ou num serviço
     de assinatura na nuvem, e não mais como arquivo `.pfx`. Se o seu vier
     assim, o passo de assinatura no CI precisa ser adaptado para a ferramenta
     do fornecedor (fale comigo com o nome do fornecedor).
   - Outra opção é o **Azure Trusted Signing** (serviço da Microsoft, por
     assinatura mensal), que também se integra ao GitHub Actions.
2. Se o certificado for um arquivo `.pfx`, cadastre no GitHub, em
   **Settings → Secrets and variables → Actions → New repository secret**:
   - `WINDOWS_CERT_PFX_BASE64` — o arquivo `.pfx` em base64
     (no Windows/PowerShell: `[Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.pfx"))`;
     no Linux/Mac: `base64 -w0 cert.pfx`);
   - `WINDOWS_CERT_PASSWORD` — a senha do `.pfx`.
3. Pronto: a próxima versão sai assinada. O SmartScreen ainda pode avisar nas
   primeiras semanas, até o programa ganhar "reputação" com os downloads.

Para assinar um `.exe` gerado no próprio computador (Linux):
`scripts/assinar-windows.sh programa.exe certificado.pfx`.

## macOS (Developer ID + notarização)

1. Entre no **Apple Developer Program** (anuidade cobrada pela Apple) com a
   conta da emissora.
2. Crie um certificado **Developer ID Application**, exporte como `.p12` com
   senha, e gere uma **senha de app** para o Apple ID (appleid.apple.com).
3. Cadastre nos Secrets do GitHub:
   - `MACOS_CERT_P12_BASE64` — o `.p12` em base64 (`base64 -i cert.p12`);
   - `MACOS_CERT_PASSWORD` — a senha do `.p12`;
   - `MACOS_SIGN_IDENTITY` — o nome do certificado, ex.:
     `Developer ID Application: Emissora LTDA (ABCDE12345)`;
   - `APPLE_ID`, `APPLE_TEAM_ID` e `APPLE_APP_PASSWORD`.
4. O CI assina, envia para a notarização da Apple e "grampeia" o resultado no
   app (`packaging/macos/assinar.sh`). Sem esses Secrets, o app recebe só a
   assinatura local ("ad-hoc"), suficiente para abrir com botão direito → Abrir.

## Linux

Não há aviso de segurança para programas baixados; não é preciso assinar.

## Ícone e dados do programa

Independentemente do certificado, o `.exe` já leva o ícone do Sintonia e os
dados do programa (nome, versão, "Sintonia"), vistos em *Propriedades →
Detalhes* no Windows. Isso é feito pelo `crates/teleprompter/build.rs`.
