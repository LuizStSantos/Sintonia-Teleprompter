//! Tela de entrada: login com o usuário do Sintonia (ou uso sem login / só
//! cronômetro), no visual do Sintonia.
//!
//! - testa o endereço do servidor enquanto o operador digita;
//! - mostra quantas tentativas restam e o tempo de bloqueio do servidor;
//! - lembra usuário e senha (senha no cofre do sistema) e pode entrar sozinho;
//! - guarda a senha da sessão só na memória, para renovar o login (12h)
//!   sem tirar o operador da tela no meio do programa.

use super::*;
use sintonia_api::normalizar_url;

/// Situação do endereço do servidor digitado.
#[derive(Clone, PartialEq)]
pub(super) enum StatusServidor {
    NaoTestado,
    Testando,
    Ok,
    Falhou(String),
}

pub(super) struct EstadoLogin {
    pub(super) senha: String,
    pub(super) mostrar_senha: bool,
    pub(super) erro: Option<FalhaLogin>,
    erro_quando: Instant,
    pub(super) entrando: bool,
    /// Entrar sozinho assim que a senha guardada chegar do cofre.
    pub(super) automatico_pendente: bool,
    /// Senha da sessão atual (só na memória) para renovar o login sozinho.
    pub(super) senha_sessao: Option<String>,
    pub(super) renovando: bool,
    servidor: StatusServidor,
    url_testada: String,
    url_mudou_em: Option<Instant>,
    focar_senha: bool,
    /// Usuário e servidor da última tentativa: o erro (e o bloqueio, que o
    /// servidor aplica por usuário) só vale para eles.
    tentativa: (String, String),
}

impl EstadoLogin {
    pub(super) fn novo(automatico: bool) -> Self {
        Self {
            senha: String::new(),
            mostrar_senha: false,
            erro: None,
            erro_quando: Instant::now(),
            entrando: false,
            automatico_pendente: automatico,
            senha_sessao: None,
            renovando: false,
            servidor: StatusServidor::NaoTestado,
            url_testada: String::new(),
            url_mudou_em: None,
            focar_senha: true,
            tentativa: (String::new(), String::new()),
        }
    }

    pub(super) fn mostrar_erro(&mut self, falha: FalhaLogin) {
        self.erro = Some(falha);
        self.erro_quando = Instant::now();
        self.focar_senha = true;
    }

    pub(super) fn receber_status_servidor(
        &mut self,
        url: &str,
        url_atual: &str,
        resultado: Result<(), String>,
    ) {
        // Resposta de um endereço que já não é o digitado: ignora.
        if normalizar_url(url) != normalizar_url(url_atual) {
            return;
        }
        self.servidor = match resultado {
            Ok(()) => StatusServidor::Ok,
            Err(e) => StatusServidor::Falhou(e),
        };
    }

    /// Tempo que falta do bloqueio por excesso de tentativas.
    fn bloqueio_restante(&self) -> Option<Duration> {
        let total = self.erro.as_ref()?.bloqueado_por?;
        total
            .checked_sub(self.erro_quando.elapsed())
            .filter(|d| !d.is_zero())
    }
}

impl App {
    pub(super) fn fazer_login(&mut self) {
        if self.login.entrando || self.login.senha.is_empty() || self.cfg.username.trim().is_empty()
        {
            return;
        }
        self.login.erro = None;
        self.login.entrando = true;
        self.login.tentativa = (
            self.cfg.username.trim().to_string(),
            normalizar_url(&self.cfg.server_url),
        );
        self.api.enviar(Comando::Login {
            url: self.cfg.server_url.clone(),
            usuario: self.cfg.username.trim().to_string(),
            senha: self.login.senha.clone(),
            lembrar: self.cfg.lembrar_login,
        });
    }

    /// Testa o servidor ~0,7 s depois que o operador para de digitar o endereço.
    fn acompanhar_servidor(&mut self, ctx: &egui::Context) {
        let url = self.cfg.server_url.trim().to_string();
        if url == self.login.url_testada {
            return;
        }
        let mudou = *self.login.url_mudou_em.get_or_insert_with(Instant::now);
        if url.is_empty() {
            self.login.servidor = StatusServidor::NaoTestado;
            self.login.url_testada = url;
            self.login.url_mudou_em = None;
        } else if mudou.elapsed() > Duration::from_millis(700) {
            self.api
                .enviar(Comando::TestarServidor { url: url.clone() });
            self.login.servidor = StatusServidor::Testando;
            self.login.url_testada = url;
            self.login.url_mudou_em = None;
        } else {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
    }

    pub(super) fn tela_login(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        self.acompanhar_servidor(&ctx);
        let p = self.p();
        // Mudou o usuário ou o servidor: o erro anterior não vale mais.
        if self.login.erro.is_some()
            && self.login.tentativa
                != (
                    self.cfg.username.trim().to_string(),
                    normalizar_url(&self.cfg.server_url),
                )
            && !self.login.tentativa.0.is_empty()
        {
            self.login.erro = None;
        }
        let bloqueio = self.login.bloqueio_restante();
        if bloqueio.is_some() {
            ctx.request_repaint_after(Duration::from_millis(500));
        }
        let rotulo = |t: &str| {
            RichText::new(t.to_uppercase())
                .size(10.5)
                .color(p.texto_mais_fraco)
        };

        CentralPanel::default().frame(egui::Frame::new().fill(p.bg)).show(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space((ui.available_height() * 0.08).max(24.0));
                    if let Some(t) = &self.marca {
                        ui.add(egui::Image::new(t).fit_to_exact_size(Vec2::splat(64.0)).corner_radius(14.0));
                    }
                    ui.add_space(10.0);
                    ui.label(RichText::new("Sintonia Teleprompter").font(tema::titulo(28.0)).color(p.texto));
                    ui.label(RichText::new("Entre com o mesmo usuário e senha do Sintonia").color(p.texto_fraco));
                    ui.add_space(20.0);

                    let largura = 380.0;
                    let mut enviar = false;
                    let cartao = egui::Frame::new()
                        .fill(p.panel)
                        .stroke(egui::Stroke::new(1.0, p.borda))
                        .corner_radius(12.0)
                        .inner_margin(egui::Margin { left: 24, right: 24, top: 22, bottom: 20 })
                        .show(ui, |ui| {
                            ui.set_width(largura);
                            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                                ui.spacing_mut().item_spacing.y = 6.0;

                                // Servidor + situação da conexão
                                ui.horizontal(|ui| {
                                    ui.label(rotulo("Servidor do Sintonia"));
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        let (cor_ponto, texto, dica) = match &self.login.servidor {
                                            StatusServidor::Ok => (p.verde, "Conectado", String::new()),
                                            StatusServidor::Testando => (p.ambar, "Testando…", String::new()),
                                            StatusServidor::Falhou(e) => (p.vermelho, "Sem conexão", e.clone()),
                                            StatusServidor::NaoTestado => (p.texto_mais_fraco, "", String::new()),
                                        };
                                        if !texto.is_empty() {
                                            let r = ui.label(RichText::new(texto).size(11.5).color(cor_ponto));
                                            if !dica.is_empty() {
                                                r.on_hover_text(dica);
                                            }
                                            let (pt, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                                            ui.painter().circle_filled(pt.center(), 4.0, cor_ponto);
                                        }
                                    });
                                });
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.cfg.server_url)
                                        .hint_text("http://servidor:3001")
                                        .desired_width(f32::INFINITY)
                                        .margin(egui::vec2(8.0, 6.0)),
                                )
                                .on_hover_text("O mesmo endereço que você usa no navegador para abrir o Sintonia");
                                if let StatusServidor::Falhou(e) = &self.login.servidor {
                                    ui.label(RichText::new(e).size(11.5).color(p.vermelho));
                                }
                                ui.add_space(6.0);

                                // Usuário
                                ui.label(rotulo("Usuário"));
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.cfg.username)
                                        .hint_text("seu usuário do Sintonia")
                                        .desired_width(f32::INFINITY)
                                        .margin(egui::vec2(8.0, 6.0)),
                                );
                                ui.add_space(6.0);

                                // Senha + mostrar/ocultar
                                ui.label(rotulo("Senha"));
                                ui.horizontal(|ui| {
                                    let r = ui.add(
                                        egui::TextEdit::singleline(&mut self.login.senha)
                                            .password(!self.login.mostrar_senha)
                                            .hint_text("senha")
                                            .desired_width(ui.available_width() - 82.0)
                                            .margin(egui::vec2(8.0, 6.0)),
                                    );
                                    if self.login.focar_senha && !self.cfg.username.trim().is_empty() {
                                        r.request_focus();
                                        self.login.focar_senha = false;
                                    }
                                    let txt = if self.login.mostrar_senha { "Ocultar" } else { "Mostrar" };
                                    if ui
                                        .add(egui::Button::new(txt).min_size(Vec2::new(66.0, 30.0)))
                                        .on_hover_text("Mostrar ou ocultar a senha digitada")
                                        .clicked()
                                    {
                                        self.login.mostrar_senha = !self.login.mostrar_senha;
                                    }
                                });
                                ui.add_space(4.0);
                                ui.checkbox(&mut self.cfg.lembrar_login, "Lembrar usuário e senha neste computador")
                                    .on_hover_text("A senha fica no cofre de senhas do sistema (Gerenciador de Credenciais do Windows / Chaveiro do Mac), nunca em arquivo.");
                                if !self.cfg.lembrar_login {
                                    self.cfg.entrar_automatico = false;
                                }
                                ui.add_enabled(
                                    self.cfg.lembrar_login,
                                    egui::Checkbox::new(&mut self.cfg.entrar_automatico, "Entrar automaticamente ao abrir"),
                                );

                                // Erro do login (com tentativas restantes e bloqueio)
                                if let Some(falha) = &self.login.erro {
                                    ui.add_space(6.0);
                                    egui::Frame::new()
                                        .fill(p.vermelho.gamma_multiply(0.14))
                                        .stroke(egui::Stroke::new(1.0, p.vermelho.gamma_multiply(0.6)))
                                        .corner_radius(8.0)
                                        .inner_margin(10.0)
                                        .show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            match bloqueio {
                                                Some(resta) => {
                                                    ui.label(RichText::new("Acesso bloqueado temporariamente").font(tema::forte(13.5)).color(p.texto));
                                                    ui.label(RichText::new(format!(
                                                        "Muitas senhas erradas. Tente de novo em {}.",
                                                        fmt_tempo(resta.as_secs() as i64 + 1)
                                                    )).color(p.texto_fraco));
                                                }
                                                None => {
                                                    ui.label(RichText::new(&falha.mensagem).color(p.texto));
                                                    if let Some(n) = falha.restantes.filter(|n| *n > 0) {
                                                        let plural = if n == 1 { "tentativa" } else { "tentativas" };
                                                        ui.label(RichText::new(format!("Restam {n} {plural} antes do bloqueio de 15 minutos."))
                                                            .size(12.0).color(p.ambar));
                                                    }
                                                }
                                            }
                                        });
                                }

                                // Entrar
                                ui.add_space(10.0);
                                let pode = !self.login.entrando
                                    && bloqueio.is_none()
                                    && !self.cfg.username.trim().is_empty()
                                    && !self.login.senha.is_empty();
                                let texto = if self.login.entrando { "Entrando…" } else { "Entrar" };
                                let largura_botao = ui.available_width();
                                if ui
                                    .add_enabled_ui(pode, |ui| {
                                        ui.add_sized(
                                            [largura_botao, 38.0],
                                            egui::Button::new(RichText::new(texto).font(tema::forte(15.0)).color(Color32::WHITE))
                                                .fill(p.vermelho),
                                        )
                                    })
                                    .inner
                                    .clicked()
                                {
                                    enviar = true;
                                }
                                if self.login.entrando {
                                    ui.vertical_centered(|ui| ui.spinner());
                                }

                                // Outras formas de usar
                                ui.add_space(12.0);
                                ui.horizontal(|ui| {
                                    let w = (ui.available_width() - 40.0) / 2.0;
                                    let linha = |ui: &mut Ui, w: f32| {
                                        let (r, _) = ui.allocate_exact_size(Vec2::new(w, 1.0), Sense::hover());
                                        ui.painter().line_segment([r.left_center(), r.right_center()], egui::Stroke::new(1.0, p.borda));
                                    };
                                    linha(ui, w);
                                    ui.label(RichText::new("ou").size(11.5).color(p.texto_mais_fraco));
                                    linha(ui, w);
                                });
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    let w = (ui.available_width() - 8.0) / 2.0;
                                    if ui
                                        .add(egui::Button::new("Sem login (só TXT)").min_size(Vec2::new(w, 32.0)))
                                        .on_hover_text("Abre o teleprompter para usar arquivos TXT, sem o Sintonia")
                                        .clicked()
                                    {
                                        self.tela = Tela::Principal;
                                    }
                                    if ui
                                        .add(egui::Button::new("⏱  Só o cronômetro").min_size(Vec2::new(w, 32.0)))
                                        .on_hover_text("Abre a tela do apresentador já com o cronômetro grande no meio")
                                        .clicked()
                                    {
                                        self.tela = Tela::Principal;
                                        self.so_cronometro = true;
                                        self.apresentador_aberto = true;
                                    }
                                });
                            });
                        });
                    // Faixa vermelha no topo do cartão (detalhe da identidade do Sintonia).
                    let r = cartao.response.rect;
                    ui.painter().rect_filled(
                        Rect::from_min_size(r.min + Vec2::new(24.0, 0.0), Vec2::new(r.width() - 48.0, 3.0)),
                        egui::CornerRadius { nw: 0, ne: 0, sw: 2, se: 2 },
                        p.vermelho,
                    );

                    if enviar || (ui.input(|i| i.key_pressed(Key::Enter)) && bloqueio.is_none()) {
                        self.fazer_login();
                    }

                    ui.add_space(14.0);
                    ui.label(
                        RichText::new(format!(
                            "Versão {} · mesmas permissões do Sintonia",
                            env!("CARGO_PKG_VERSION")
                        ))
                        .size(11.0)
                        .color(p.texto_mais_fraco),
                    );
                });
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn falha(bloqueio: Option<u64>) -> FalhaLogin {
        FalhaLogin {
            mensagem: "x".into(),
            restantes: Some(0),
            bloqueado_por: bloqueio.map(Duration::from_secs),
        }
    }

    #[test]
    fn bloqueio_conta_o_tempo() {
        let mut l = EstadoLogin::novo(false);
        assert!(l.bloqueio_restante().is_none());
        l.mostrar_erro(falha(Some(900)));
        let resta = l.bloqueio_restante().unwrap();
        assert!(resta > Duration::from_secs(890) && resta <= Duration::from_secs(900));
        l.mostrar_erro(falha(None));
        assert!(l.bloqueio_restante().is_none());
    }

    #[test]
    fn status_de_outro_endereco_e_ignorado() {
        let mut l = EstadoLogin::novo(false);
        l.receber_status_servidor("servidor-velho:3001", "servidor:3001", Ok(()));
        assert!(l.servidor == StatusServidor::NaoTestado);
        l.receber_status_servidor("http://servidor:3001/", "servidor:3001", Err("x".into()));
        assert!(l.servidor == StatusServidor::Falhou("x".into()));
    }
}
