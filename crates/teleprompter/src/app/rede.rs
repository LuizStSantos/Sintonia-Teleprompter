//! Respostas do servidor do Sintonia e gravação dos textos no Espelho.

use super::*;

impl App {
    pub(super) fn processar_respostas(&mut self) {
        while let Some(r) = self.api.receber() {
            match r {
                Resposta::Logado(u) if self.login.renovando => {
                    // Sessão expirou no meio do uso e foi renovada sozinha:
                    // continua de onde estava e repete o que tinha falhado.
                    self.login.renovando = false;
                    self.usuario = Some(u);
                    self.api.repetir_ultimo();
                    self.avisar("Sessão do Sintonia renovada automaticamente.", false);
                }
                Resposta::Logado(u) => {
                    self.login.erro = None;
                    self.login.entrando = false;
                    // Guardada só na memória, para renovar a sessão (12h) sem parar o programa.
                    self.login.senha_sessao = Some(std::mem::take(&mut self.login.senha));
                    self.tela = Tela::Principal;
                    self.cfg.username = if self.cfg.lembrar_login {
                        u.username.clone()
                    } else {
                        String::new()
                    };
                    // Grava já (não espera o salvamento periódico): o login fica lembrado
                    // mesmo que o programa seja fechado logo em seguida.
                    self.cfg.save();
                    self.cfg_salva = self.cfg.clone();
                    if u.pode("espelho", "view") {
                        self.seletor_aberto = true;
                        self.api.enviar(Comando::ListarEspelhos);
                    } else {
                        self.avisar("Seu usuário não tem acesso à aba Espelho no Sintonia. Peça a um administrador, ou use TXT.", true);
                    }
                    self.usuario = Some(u);
                }
                Resposta::ErroLogin(falha) => {
                    self.login.entrando = false;
                    if self.login.renovando {
                        // Não deu para renovar sozinho (senha trocada?): volta ao login.
                        self.login.renovando = false;
                        self.login.senha_sessao = None;
                        self.tela = Tela::Login;
                    }
                    self.login.mostrar_erro(falha);
                }
                Resposta::SenhaGuardada(senha) => {
                    if self.login.senha.is_empty() && !senha.is_empty() {
                        self.login.senha = senha;
                        if self.login.automatico_pendente {
                            self.fazer_login();
                        }
                    }
                    self.login.automatico_pendente = false;
                }
                Resposta::ServidorStatus { url, resultado } => {
                    self.login
                        .receber_status_servidor(&url, &self.cfg.server_url, resultado);
                }
                Resposta::Espelhos(mut lista) => {
                    lista.sort_by(|a, b| {
                        let k = |v: &Value| {
                            (
                                v["data"].as_str().unwrap_or("").to_string(),
                                v["horarioPrograma"].as_str().unwrap_or("").to_string(),
                            )
                        };
                        k(b).cmp(&k(a))
                    });
                    if let Some(id) = self.recarregar_id.take() {
                        match lista.iter().find(|e| e["id"].as_str() == Some(id.as_str())) {
                            Some(esp) => {
                                let rolagem = self.rolagem_em;
                                self.carregar_espelho(&esp.clone());
                                self.rolagem_em = rolagem;
                                self.avisar("Espelho recarregado do Sintonia.", false);
                            }
                            None => self.avisar("Esse espelho não existe mais no Sintonia.", true),
                        }
                    }
                    self.espelhos = lista;
                }
                Resposta::Salvo(esp) => {
                    self.roteiro.marcar_como_salvo();
                    self.editado = false;
                    let _ = esp;
                    self.avisar("Textos gravados no Espelho do Sintonia.", false);
                }
                Resposta::SessaoExpirada => {
                    // A sessão do Sintonia dura 12h. Se a senha desta sessão está na
                    // memória, renova sozinho sem tirar o operador da tela.
                    match self.login.senha_sessao.clone() {
                        Some(senha) if !self.login.renovando => {
                            self.login.renovando = true;
                            self.api.enviar(Comando::Login {
                                url: self.cfg.server_url.clone(),
                                usuario: self
                                    .usuario
                                    .as_ref()
                                    .map(|u| u.username.clone())
                                    .unwrap_or_default(),
                                senha,
                                lembrar: self.cfg.lembrar_login,
                            });
                        }
                        _ => {
                            self.tela = Tela::Login;
                            self.login.mostrar_erro(FalhaLogin {
                                mensagem: "Sua sessão expirou (12h). Entre de novo — o roteiro aberto continua aqui.".into(),
                                restantes: None,
                                bloqueado_por: None,
                            });
                        }
                    }
                }
                Resposta::Erro(m) => self.avisar(m, true),
                Resposta::Saiu => {
                    self.login.senha_sessao = None;
                    self.usuario = None;
                    self.tela = Tela::Login;
                }
            }
        }
    }

    pub(super) fn salvar_no_sintonia(&mut self) {
        let Origem::Espelho { id, .. } = &self.roteiro.origem else {
            return;
        };
        let textos: Vec<(String, String)> = self
            .roteiro
            .segmentos
            .iter()
            .filter(|s| s.alterado())
            .filter_map(|s| s.item_id.clone().map(|i| (i, s.texto.clone())))
            .collect();
        if textos.is_empty() {
            self.avisar("Nenhum texto do espelho foi alterado.", false);
            return;
        }
        self.api.enviar(Comando::SalvarTextos {
            espelho_id: id.clone(),
            textos,
        });
    }
}
