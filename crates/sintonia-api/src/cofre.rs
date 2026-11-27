//! Senha guardada no cofre do sistema operacional: Gerenciador de Credenciais
//! (Windows), Chaveiro/Keychain (macOS) ou Secret Service/GNOME Keyring (Linux).
//! Nunca vai para arquivo. Se o cofre não existir, simplesmente não lembra.

const SERVICO: &str = "Sintonia Teleprompter";

fn chave(base: &str, usuario: &str) -> String {
    format!("{usuario}@{base}")
}

pub fn ler(base: &str, usuario: &str) -> Option<String> {
    if usuario.trim().is_empty() {
        return None;
    }
    keyring::Entry::new(SERVICO, &chave(base, usuario))
        .ok()?
        .get_password()
        .ok()
}

/// `Some(senha)` guarda; `None` apaga o que houver.
pub fn guardar(base: &str, usuario: &str, senha: Option<&str>) {
    let Ok(entrada) = keyring::Entry::new(SERVICO, &chave(base, usuario)) else {
        return;
    };
    let _ = match senha {
        Some(s) => entrada.set_password(s),
        None => entrada.delete_credential(),
    };
}
