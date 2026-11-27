//! Comunicação com o servidor do Sintonia (mesma API usada pelo site).
//!
//! Login: `POST /api/auth/login` devolve o usuário e um cookie de sessão
//! (`controle_tv_token`, JWT de 12h). O cliente HTTP guarda o cookie sozinho
//! e o envia em todas as chamadas seguintes — exatamente como o navegador faz.
//! Não é preciso nenhuma mudança no servidor.
//!
//! As chamadas rodam numa thread separada para a tela nunca travar; a tela
//! manda um [`Comando`] e recebe uma [`Resposta`] pelo canal.

mod cofre;
mod usuario;

pub use usuario::Usuario;

use serde_json::Value;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

#[derive(Clone)]
pub enum Comando {
    /// `lembrar`: guarda (ou apaga) a senha no cofre do sistema se o login der certo.
    Login {
        url: String,
        usuario: String,
        senha: String,
        lembrar: bool,
    },
    /// Busca no cofre do sistema a senha guardada para esse servidor + usuário.
    LerSenhaGuardada {
        url: String,
        usuario: String,
    },
    /// Verifica se o endereço responde como um servidor do Sintonia (não faz login).
    TestarServidor {
        url: String,
    },
    Sair,
    ListarEspelhos,
    /// Grava de volta no Espelho os textos alterados: `(id do item, texto novo)`.
    SalvarTextos {
        espelho_id: String,
        textos: Vec<(String, String)>,
    },
}

/// Login recusado, com os detalhes que o servidor manda.
#[derive(Clone, Debug, PartialEq)]
pub struct FalhaLogin {
    pub mensagem: String,
    /// Tentativas que sobram antes do bloqueio (o servidor bloqueia após 5).
    pub restantes: Option<u32>,
    /// Bloqueado por excesso de tentativas: quanto tempo falta.
    pub bloqueado_por: Option<Duration>,
}

impl FalhaLogin {
    fn simples(mensagem: impl Into<String>) -> Self {
        Self {
            mensagem: mensagem.into(),
            restantes: None,
            bloqueado_por: None,
        }
    }
}

pub enum Resposta {
    Logado(Usuario),
    ErroLogin(FalhaLogin),
    /// Resultado de [`Comando::TestarServidor`] para aquele endereço.
    ServidorStatus {
        url: String,
        resultado: Result<(), String>,
    },
    SenhaGuardada(String),
    Espelhos(Vec<Value>),
    Salvo(Value),
    SessaoExpirada,
    Erro(String),
    Saiu,
}

pub struct Api {
    tx: Sender<Comando>,
    rx: Receiver<Resposta>,
    /// Comandos "de trabalho" ainda sem resposta (o teste de conexão e a
    /// leitura do cofre não contam: rodam em segundo plano sem travar botões).
    pendentes: usize,
    /// Último comando que precisava de sessão — repetido depois de renovar
    /// o login automaticamente (ver [`Api::repetir_ultimo`]).
    ultimo: Option<Comando>,
}

fn conta_como_ocupado(c: &Comando) -> bool {
    !matches!(
        c,
        Comando::TestarServidor { .. } | Comando::LerSenhaGuardada { .. }
    )
}

impl Api {
    /// Inicia a thread de rede. `ao_responder` é chamado a cada resposta
    /// (a interface usa para se redesenhar).
    pub fn iniciar(ao_responder: impl Fn() + Send + 'static) -> Self {
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
                    ao_responder();
                }
            })
            .expect("não foi possível iniciar a thread de rede");
        Self {
            tx,
            rx,
            pendentes: 0,
            ultimo: None,
        }
    }

    pub fn enviar(&mut self, cmd: Comando) {
        if conta_como_ocupado(&cmd) {
            self.pendentes += 1;
        }
        if matches!(cmd, Comando::ListarEspelhos | Comando::SalvarTextos { .. }) {
            self.ultimo = Some(cmd.clone());
        }
        let _ = self.tx.send(cmd);
    }

    pub fn receber(&mut self) -> Option<Resposta> {
        let r = self.rx.try_recv().ok()?;
        if !matches!(
            r,
            Resposta::ServidorStatus { .. } | Resposta::SenhaGuardada(_)
        ) {
            self.pendentes = self.pendentes.saturating_sub(1);
        }
        Some(r)
    }

    /// Há alguma operação (login, carregar, gravar) em andamento.
    pub fn ocupado(&self) -> bool {
        self.pendentes > 0
    }

    /// Repete a última operação que falhou por sessão expirada.
    pub fn repetir_ultimo(&mut self) {
        if let Some(c) = self.ultimo.clone() {
            self.enviar(c);
        }
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

/// Confere se o endereço responde como o Sintonia: `GET /api/auth/me` sem
/// login devolve 401 com `{ "error": ... }` (ou 200, se já houver sessão).
fn testar_servidor(base: &str) -> Result<(), String> {
    let cliente = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(4))
        .build()
        .map_err(|e| e.to_string())?;
    match cliente.get(format!("{base}/api/auth/me")).send() {
        Ok(r) if matches!(r.status().as_u16(), 200 | 401) => {
            let corpo: Value = r.json().unwrap_or(Value::Null);
            if corpo.is_object() {
                Ok(())
            } else {
                Err("Respondeu, mas não parece ser o Sintonia.".into())
            }
        }
        Ok(r) => Err(format!(
            "Respondeu {}, mas não parece ser o Sintonia.",
            r.status().as_u16()
        )),
        Err(e) if e.is_timeout() => Err("O servidor não respondeu a tempo.".into()),
        Err(_) => Err("Sem conexão com esse endereço.".into()),
    }
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
                lembrar,
            } => {
                // Cliente novo = cookies zerados (troca de usuário/servidor).
                self.cliente = novo_cliente();
                self.base = normalizar_url(&url);
                match self.login(&usuario, &senha) {
                    Ok(u) => {
                        cofre::guardar(&self.base, &usuario, lembrar.then_some(senha.as_str()));
                        Resposta::Logado(u)
                    }
                    Err(f) => Resposta::ErroLogin(f),
                }
            }
            Comando::TestarServidor { url } => Resposta::ServidorStatus {
                resultado: testar_servidor(&normalizar_url(&url)),
                url,
            },
            Comando::LerSenhaGuardada { url, usuario } => Resposta::SenhaGuardada(
                cofre::ler(&normalizar_url(&url), &usuario).unwrap_or_default(),
            ),
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

    fn login(&self, usuario: &str, senha: &str) -> Result<Usuario, FalhaLogin> {
        let r = self
            .cliente
            .post(format!("{}/api/auth/login", self.base))
            .json(&serde_json::json!({ "username": usuario, "password": senha }))
            .send()
            .map_err(|e| match Falha::from(e) {
                Falha::Msg(m) => FalhaLogin::simples(m),
                Falha::Sessao => FalhaLogin::simples("Erro de comunicação."),
            })?;
        let status = r.status().as_u16();
        if status == 200 {
            return r
                .json::<Usuario>()
                .map_err(|e| FalhaLogin::simples(format!("Resposta inesperada do servidor: {e}")));
        }
        let corpo: Value = r.json().unwrap_or(Value::Null);
        let mensagem = corpo
            .get("error")
            .and_then(Value::as_str)
            .map(str::to_string);
        Err(match status {
            // Senha errada: o servidor diz quantas tentativas sobram.
            401 => FalhaLogin {
                mensagem: mensagem.unwrap_or_else(|| "Usuário ou senha inválidos.".into()),
                restantes: corpo
                    .get("restantes")
                    .and_then(Value::as_u64)
                    .map(|n| n as u32),
                bloqueado_por: None,
            },
            // Muitas tentativas: bloqueado por alguns minutos.
            429 => FalhaLogin {
                mensagem: mensagem
                    .unwrap_or_else(|| "Muitas tentativas. Aguarde alguns minutos.".into()),
                restantes: Some(0),
                bloqueado_por: corpo
                    .get("segundos")
                    .and_then(Value::as_u64)
                    .map(Duration::from_secs),
            },
            404 => FalhaLogin::simples("Esse endereço não parece ser o servidor do Sintonia."),
            _ => FalhaLogin::simples(
                mensagem.unwrap_or_else(|| format!("O servidor respondeu {status}.")),
            ),
        })
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
    fn teste_de_conexao_nao_trava_os_botoes() {
        let mut api = Api::iniciar(|| {});
        api.enviar(Comando::TestarServidor {
            url: "127.0.0.1:9".into(),
        });
        assert!(!api.ocupado(), "testar o servidor roda em segundo plano");
        api.enviar(Comando::ListarEspelhos);
        assert!(api.ocupado());
        assert!(
            api.ultimo.is_some(),
            "guardado para repetir após renovar a sessão"
        );
    }

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
