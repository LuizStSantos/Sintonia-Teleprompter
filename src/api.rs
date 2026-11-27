//! Comunicação com o servidor do Sintonia (mesma API usada pelo site).
//!
//! Login: `POST /api/auth/login` devolve o usuário e um cookie de sessão
//! (`controle_tv_token`, JWT de 12h). O cliente HTTP guarda o cookie sozinho
//! e o envia em todas as chamadas seguintes — exatamente como o navegador faz.
//! Não é preciso nenhuma mudança no servidor.
//!
//! As chamadas rodam numa thread separada para a tela nunca travar; a tela
//! manda um [`Comando`] e recebe uma [`Resposta`] pelo canal.

use serde::Deserialize;
use serde_json::Value;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

#[derive(Clone, Debug, Deserialize)]
pub struct Usuario {
    #[allow(dead_code)]
    pub id: String,
    pub username: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub permissions: Value,
    #[serde(default, rename = "nomeCompleto")]
    pub nome_completo: String,
}

impl Usuario {
    pub fn nome(&self) -> &str {
        if self.nome_completo.trim().is_empty() {
            &self.username
        } else {
            &self.nome_completo
        }
    }

    /// Mesma regra do servidor: admin pode tudo; os demais seguem `permissions[colecao]`.
    pub fn pode(&self, colecao: &str, nivel: &str) -> bool {
        if self.role == "admin" {
            return true;
        }
        let p = self
            .permissions
            .get(colecao)
            .and_then(Value::as_str)
            .unwrap_or("none");
        match nivel {
            "view" => p == "view" || p == "edit",
            "edit" => p == "edit",
            _ => false,
        }
    }
}

pub enum Comando {
    Login {
        url: String,
        usuario: String,
        senha: String,
    },
    Sair,
    ListarEspelhos,
    /// Grava de volta no Espelho os textos alterados: `(id do item, texto novo)`.
    SalvarTextos {
        espelho_id: String,
        textos: Vec<(String, String)>,
    },
}

pub enum Resposta {
    Logado(Usuario),
    ErroLogin(String),
    Espelhos(Vec<Value>),
    Salvo(Value),
    SessaoExpirada,
    Erro(String),
    Saiu,
}

pub struct Api {
    tx: Sender<Comando>,
    rx: Receiver<Resposta>,
    pub ocupado: bool,
}

impl Api {
    pub fn iniciar(ctx: egui::Context) -> Self {
        let (tx, rx_cmd) = channel::<Comando>();
        let (tx_resp, rx) = channel::<Resposta>();
        std::thread::Builder::new()
            .name("sintonia-api".into())
            .spawn(move || {
                let mut w = Worker::new();
                for cmd in rx_cmd {
                    let resp = w.executar(cmd);
                    if tx_resp.send(resp).is_err() {
                        break;
                    }
                    ctx.request_repaint();
                }
            })
            .expect("não foi possível iniciar a thread de rede");
        Self {
            tx,
            rx,
            ocupado: false,
        }
    }

    pub fn enviar(&mut self, cmd: Comando) {
        self.ocupado = true;
        let _ = self.tx.send(cmd);
    }

    pub fn receber(&mut self) -> Option<Resposta> {
        let r = self.rx.try_recv().ok();
        if r.is_some() {
            self.ocupado = false;
        }
        r
    }
}

struct Worker {
    cliente: reqwest::blocking::Client,
    base: String,
}

enum Falha {
    Sessao,
    Msg(String),
}

impl From<reqwest::Error> for Falha {
    fn from(e: reqwest::Error) -> Self {
        if e.is_connect() || e.is_timeout() {
            Falha::Msg(
                "Não foi possível conectar ao servidor do Sintonia. Confira o endereço e a rede."
                    .into(),
            )
        } else {
            Falha::Msg(format!("Erro de comunicação: {e}"))
        }
    }
}

fn novo_cliente() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .cookie_store(true)
        .timeout(Duration::from_secs(20))
        .user_agent(concat!("SintoniaTeleprompter/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("cliente HTTP")
}

/// Aceita "servidor:3001", "http://servidor:3001/" etc.
pub fn normalizar_url(url: &str) -> String {
    let u = url.trim().trim_end_matches('/');
    if u.starts_with("http://") || u.starts_with("https://") {
        u.to_string()
    } else {
        format!("http://{u}")
    }
}

impl Worker {
    fn new() -> Self {
        Self {
            cliente: novo_cliente(),
            base: String::new(),
        }
    }

    fn executar(&mut self, cmd: Comando) -> Resposta {
        match cmd {
            Comando::Login {
                url,
                usuario,
                senha,
            } => {
                // Cliente novo = cookies zerados (troca de usuário/servidor).
                self.cliente = novo_cliente();
                self.base = normalizar_url(&url);
                match self.login(&usuario, &senha) {
                    Ok(u) => Resposta::Logado(u),
                    Err(Falha::Sessao) => Resposta::ErroLogin("Usuário ou senha inválidos.".into()),
                    Err(Falha::Msg(m)) => Resposta::ErroLogin(m),
                }
            }
            Comando::Sair => {
                let _ = self
                    .cliente
                    .post(format!("{}/api/auth/logout", self.base))
                    .send();
                self.cliente = novo_cliente();
                Resposta::Saiu
            }
            Comando::ListarEspelhos => self.resp(
                self.get("/api/espelho")
                    .map(|v| Resposta::Espelhos(v.as_array().cloned().unwrap_or_default())),
            ),
            Comando::SalvarTextos { espelho_id, textos } => self.resp(
                self.salvar_textos(&espelho_id, &textos)
                    .map(Resposta::Salvo),
            ),
        }
    }

    fn resp(&self, r: Result<Resposta, Falha>) -> Resposta {
        match r {
            Ok(r) => r,
            Err(Falha::Sessao) => Resposta::SessaoExpirada,
            Err(Falha::Msg(m)) => Resposta::Erro(m),
        }
    }

    fn checar(r: reqwest::blocking::Response) -> Result<Value, Falha> {
        let status = r.status();
        if status.as_u16() == 401 {
            return Err(Falha::Sessao);
        }
        let corpo: Value = r.json().unwrap_or(Value::Null);
        if status.is_success() {
            return Ok(corpo);
        }
        let msg = corpo
            .get("error")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!("O servidor respondeu {status}"));
        Err(Falha::Msg(if status.as_u16() == 403 {
            format!("Sem permissão: {msg}")
        } else {
            msg
        }))
    }

    fn login(&self, usuario: &str, senha: &str) -> Result<Usuario, Falha> {
        let r = self
            .cliente
            .post(format!("{}/api/auth/login", self.base))
            .json(&serde_json::json!({ "username": usuario, "password": senha }))
            .send()?;
        if r.status().as_u16() == 401 {
            let corpo: Value = r.json().unwrap_or(Value::Null);
            let msg = corpo
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("Usuário ou senha inválidos.");
            return Err(Falha::Msg(msg.to_string()));
        }
        let v = Self::checar(r)?;
        serde_json::from_value(v)
            .map_err(|e| Falha::Msg(format!("Resposta inesperada do servidor: {e}")))
    }

    fn get(&self, caminho: &str) -> Result<Value, Falha> {
        Self::checar(self.cliente.get(format!("{}{caminho}", self.base)).send()?)
    }

    /// Busca o espelho mais recente no servidor e troca só o `texto` dos itens
    /// alterados — assim não desfaz mudanças que outra pessoa tenha feito no
    /// espelho (ordem, itens novos...) enquanto o teleprompter estava aberto.
    fn salvar_textos(&self, id: &str, textos: &[(String, String)]) -> Result<Value, Falha> {
        let lista = self.get("/api/espelho")?;
        let mut esp = lista
            .as_array()
            .and_then(|l| {
                l.iter()
                    .find(|e| e.get("id").and_then(Value::as_str) == Some(id))
            })
            .cloned()
            .ok_or_else(|| {
                Falha::Msg("Esse espelho não existe mais no Sintonia (foi apagado?).".into())
            })?;
        let mut faltando = Vec::new();
        if let Some(itens) = esp.get_mut("itens").and_then(Value::as_array_mut) {
            for (item_id, texto) in textos {
                match itens
                    .iter_mut()
                    .find(|i| i.get("id").and_then(Value::as_str) == Some(item_id))
                {
                    Some(item) => item["texto"] = Value::String(texto.clone()),
                    None => faltando.push(item_id.clone()),
                }
            }
        }
        if !faltando.is_empty() {
            return Err(Falha::Msg(format!(
                "{} item(ns) foram removidos do espelho por outra pessoa. Recarregue o espelho antes de salvar.",
                faltando.len()
            )));
        }
        let r = self
            .cliente
            .put(format!("{}/api/espelho/{id}", self.base))
            .json(&esp)
            .send()?;
        Self::checar(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url() {
        assert_eq!(normalizar_url("servidor:3001/"), "http://servidor:3001");
        assert_eq!(
            normalizar_url("https://tv.exemplo.com"),
            "https://tv.exemplo.com"
        );
    }

    #[test]
    fn permissoes() {
        let u: Usuario = serde_json::from_value(serde_json::json!({
            "id": "1", "username": "ana", "role": "user", "permissions": {"espelho": "view"}
        }))
        .unwrap();
        assert!(u.pode("espelho", "view"));
        assert!(!u.pode("espelho", "edit"));
        assert!(!u.pode("materias", "view"));
    }
}
