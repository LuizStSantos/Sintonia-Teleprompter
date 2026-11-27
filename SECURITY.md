# Segurança

## Senhas e sessão

- O teleprompter usa o **mesmo login do Sintonia** (`POST /api/auth/login`) e a
  mesma sessão (cookie `controle_tv_token`, válida por 12 horas). As permissões
  são as do servidor: o app só mostra o que o usuário pode ver e só grava o que
  ele pode editar — e o servidor confere de novo.
- **A senha nunca é gravada em arquivo nem em log.**
  - Com "Lembrar usuário e senha" marcado, ela vai para o **cofre de senhas do
    sistema**: Gerenciador de Credenciais (Windows), Chaveiro/Keychain (macOS)
    ou Secret Service/GNOME Keyring (Linux). Desmarcando, ela é apagada do cofre
    no próximo login.
  - Durante o uso, a senha fica **só na memória**, para renovar a sessão quando
    ela expira no meio de um programa. Sai junto com o programa ou ao "Sair do
    Sintonia".
- O `config.json` (pasta de configuração do usuário) guarda só endereço do
  servidor, usuário e preferências de aparência.
- O servidor do Sintonia bloqueia o login por 15 minutos após 5 senhas erradas;
  o app mostra as tentativas restantes e o tempo do bloqueio.

## Conexão

- Use o Sintonia atrás de **HTTPS** quando a rede não for só a interna da
  emissora (o endereço aceita `https://`).
- O app não tem servidor próprio nem abre portas.

## Como relatar um problema de segurança

Não abra uma issue pública. Fale diretamente com o responsável pelo Sintonia
na emissora, com a descrição do problema e como reproduzir.
