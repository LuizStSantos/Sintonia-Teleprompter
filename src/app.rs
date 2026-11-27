//! Telas do Sintonia Teleprompter: login, operador e apresentador.

use crate::api::{Api, Comando, Resposta, Usuario};
use crate::fonts::Fontes;
use crate::prompter::{self, Espelho, Medidas, cor};
use crate::script::{EspelhoResumo, Origem, Roteiro, Segmento, Tipo, data_br, decodificar_txt};
use crate::settings::{Alinhamento, Settings};
use crate::theme::{self, Paleta};
use crate::timer::{Cronometro, fmt_tempo, parse_duracao};
use egui::{
    Align, Align2, CentralPanel, Color32, FontId, Key, Layout, Panel, Rect, RichText, ScrollArea,
    Sense, Ui, Vec2, ViewportBuilder, ViewportId,
};
use serde_json::Value;
use std::time::{Duration, Instant};

const ID_APRESENTADOR: &str = "sintonia-apresentador";

#[derive(PartialEq)]
enum Tela {
    Login,
    Principal,
}

/// Ação que depende de confirmação (há alterações não salvas no roteiro).
enum Pendente {
    AbrirEspelho(Value),
    AbrirTxt,
    Novo,
}

struct Aviso {
    texto: String,
    erro: bool,
    quando: Instant,
}

pub struct App {
    /// Ícone do Sintonia (torre de rádio) já carregado como textura.
    marca: Option<egui::TextureHandle>,
    tema_aplicado: bool,
    cfg: Settings,
    cfg_salva: Settings,
    cfg_checada: Instant,

    api: Api,
    tela: Tela,
    usuario: Option<Usuario>,
    senha: String,
    erro_login: Option<String>,

    roteiro: Roteiro,
    editado: bool,
    selecionado: Option<usize>,

    rolagem_em: f32,
    rodando: bool,
    medidas: Medidas,
    /// Resumo da formatação usada no último desenho; quando muda (fonte,
    /// tamanho, margem...), a rolagem é reposicionada no mesmo trecho do texto.
    assinatura_layout: String,
    tamanho_apresentador: Vec2,
    apresentador_aberto: bool,

    crono: Cronometro,
    meta_txt: String,

    espelhos: Vec<Value>,
    seletor_aberto: bool,
    filtro_data: String,
    filtro_busca: String,
    recarregar_id: Option<String>,

    fontes: Fontes,
    busca_fonte: String,

    aviso: Option<Aviso>,
    pendente: Option<Pendente>,
}

fn hoje() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let cfg = Settings::load();
        let fontes = Fontes::carregar_sistema();
        cc.egui_ctx.set_fonts(fontes.definicoes());
        theme::aplicar(&cc.egui_ctx, cfg.tema_claro);
        let marca = eframe::icon_data::from_png_bytes(theme::ICONE_PNG)
            .ok()
            .map(|ic| {
                let img = egui::ColorImage::from_rgba_unmultiplied(
                    [ic.width as usize, ic.height as usize],
                    &ic.rgba,
                );
                cc.egui_ctx
                    .load_texture("sintonia-marca", img, egui::TextureOptions::LINEAR)
            });
        Self {
            marca,
            tema_aplicado: cfg.tema_claro,
            cfg_salva: cfg.clone(),
            cfg,
            cfg_checada: Instant::now(),
            api: Api::iniciar(cc.egui_ctx.clone()),
            tela: Tela::Login,
            usuario: None,
            senha: String::new(),
            erro_login: None,
            roteiro: Roteiro {
                titulo: "Roteiro novo".into(),
                segmentos: vec![Segmento::novo_texto(
                    "BEM-VINDO",
                    "Abra um espelho do Sintonia, um arquivo TXT, ou digite o texto aqui ao lado.",
                )],
                ..Default::default()
            },
            editado: false,
            selecionado: Some(0),
            rolagem_em: 0.0,
            rodando: false,
            medidas: Medidas::default(),
            assinatura_layout: String::new(),
            tamanho_apresentador: Vec2::new(1280.0, 720.0),
            apresentador_aberto: false,
            crono: Cronometro::default(),
            meta_txt: String::new(),
            espelhos: Vec::new(),
            seletor_aberto: false,
            filtro_data: hoje(),
            filtro_busca: String::new(),
            recarregar_id: None,
            fontes,
            busca_fonte: String::new(),
            aviso: None,
            pendente: None,
        }
    }

    fn p(&self) -> Paleta {
        theme::paleta(self.cfg.tema_claro)
    }

    /// Marca do Sintonia: ícone + "Sintonia" em Space Grotesk + subtítulo.
    fn marca(&self, ui: &mut Ui, tamanho: f32) {
        let p = self.p();
        ui.horizontal(|ui| {
            if let Some(t) = &self.marca {
                ui.add(
                    egui::Image::new(t)
                        .fit_to_exact_size(Vec2::splat(tamanho))
                        .corner_radius(tamanho * 0.22),
                );
            }
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(
                    RichText::new("Sintonia")
                        .font(theme::titulo(tamanho * 0.48))
                        .color(p.texto),
                );
                ui.label(
                    RichText::new("Teleprompter")
                        .size(tamanho * 0.32)
                        .color(p.texto_mais_fraco),
                );
            });
        });
    }

    fn avisar(&mut self, texto: impl Into<String>, erro: bool) {
        self.aviso = Some(Aviso {
            texto: texto.into(),
            erro,
            quando: Instant::now(),
        });
    }

    fn pode_editar_espelho(&self) -> bool {
        self.usuario
            .as_ref()
            .is_some_and(|u| u.pode("espelho", "edit"))
    }

    // ------------------------------------------------------------------
    // Rede
    // ------------------------------------------------------------------

    fn processar_respostas(&mut self) {
        while let Some(r) = self.api.receber() {
            match r {
                Resposta::Logado(u) => {
                    self.erro_login = None;
                    self.senha.clear();
                    self.tela = Tela::Principal;
                    self.cfg.username = u.username.clone();
                    if u.pode("espelho", "view") {
                        self.seletor_aberto = true;
                        self.api.enviar(Comando::ListarEspelhos);
                    } else {
                        self.avisar("Seu usuário não tem acesso à aba Espelho no Sintonia. Peça a um administrador, ou use TXT.", true);
                    }
                    self.usuario = Some(u);
                }
                Resposta::ErroLogin(m) => self.erro_login = Some(m),
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
                    self.tela = Tela::Login;
                    self.erro_login = Some(
                        "Sua sessão expirou (12h). Entre de novo — o roteiro aberto continua aqui."
                            .into(),
                    );
                }
                Resposta::Erro(m) => self.avisar(m, true),
                Resposta::Saiu => {
                    self.usuario = None;
                    self.tela = Tela::Login;
                }
            }
        }
    }

    fn carregar_espelho(&mut self, esp: &Value) {
        self.roteiro = Roteiro::do_espelho(esp);
        self.depois_de_carregar();
        let total = self.roteiro.duracao_total();
        if total > 0 {
            self.meta_txt = fmt_tempo(total);
        }
    }

    fn depois_de_carregar(&mut self) {
        self.editado = false;
        self.rolagem_em = 0.0;
        self.rodando = false;
        self.selecionado = if self.roteiro.segmentos.is_empty() {
            None
        } else {
            Some(0)
        };
        self.medidas = Medidas::default();
    }

    fn executar(&mut self, acao: Pendente) {
        match acao {
            Pendente::AbrirEspelho(esp) => {
                self.carregar_espelho(&esp);
                self.seletor_aberto = false;
            }
            Pendente::AbrirTxt => self.abrir_txt(),
            Pendente::Novo => {
                self.roteiro = Roteiro {
                    titulo: "Roteiro novo".into(),
                    segmentos: vec![Segmento::novo_texto("", "")],
                    ..Default::default()
                };
                self.depois_de_carregar();
            }
        }
    }

    fn pedir(&mut self, acao: Pendente) {
        if self.editado {
            self.pendente = Some(acao);
        } else {
            self.executar(acao);
        }
    }

    fn abrir_txt(&mut self) {
        let Some(caminho) = rfd::FileDialog::new()
            .add_filter("Texto", &["txt", "TXT"])
            .pick_file()
        else {
            return;
        };
        match std::fs::read(&caminho) {
            Ok(bytes) => {
                self.roteiro = Roteiro::do_txt(&decodificar_txt(&bytes), Some(caminho));
                self.depois_de_carregar();
            }
            Err(e) => self.avisar(format!("Não foi possível abrir o arquivo: {e}"), true),
        }
    }

    fn salvar_txt(&mut self) {
        let nome = format!(
            "{}.txt",
            self.roteiro
                .titulo
                .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-")
        );
        let Some(caminho) = rfd::FileDialog::new()
            .add_filter("Texto", &["txt"])
            .set_file_name(nome)
            .save_file()
        else {
            return;
        };
        match std::fs::write(&caminho, self.roteiro.para_txt()) {
            Ok(()) => {
                if matches!(self.roteiro.origem, Origem::Arquivo(_) | Origem::Nenhuma) {
                    self.editado = false;
                }
                self.avisar(format!("Salvo em {}", caminho.display()), false)
            }
            Err(e) => self.avisar(format!("Não foi possível salvar: {e}"), true),
        }
    }

    fn salvar_no_sintonia(&mut self) {
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

    // ------------------------------------------------------------------
    // Rolagem e atalhos
    // ------------------------------------------------------------------

    fn em_por_segundo(&self) -> f32 {
        self.cfg.velocidade * 0.12
    }

    fn ir_para_bloco(&mut self, i: usize) {
        if let Some(t) = self.medidas.topos_em.get(i) {
            self.rolagem_em = *t;
            self.selecionado = Some(i);
        }
    }

    fn bloco_relativo(&mut self, passo: i32) {
        let atual = self.medidas.bloco_atual(self.rolagem_em).unwrap_or(0) as i32;
        // "Anterior" no meio de um bloco volta para o começo dele.
        let alvo = if passo < 0
            && self
                .medidas
                .topos_em
                .get(atual as usize)
                .is_some_and(|t| self.rolagem_em - t > 0.5)
        {
            atual
        } else {
            atual + passo
        };
        let max = self.medidas.topos_em.len().saturating_sub(1) as i32;
        self.ir_para_bloco(alvo.clamp(0, max) as usize);
    }

    /// Liga/desliga a rolagem. Como no teleprompter antigo (Apotec), o
    /// cronômetro dispara sozinho quando o texto começa a rolar.
    fn alternar_rolagem(&mut self) {
        self.rodando = !self.rodando;
        if self.rodando && self.cfg.crono_automatico {
            self.crono.iniciar();
        }
    }

    /// Volta ao começo do texto e zera o cronômetro (tecla Home, igual ao Apotec).
    fn ir_inicio(&mut self) {
        self.rolagem_em = 0.0;
        self.rodando = false;
        if self.cfg.crono_automatico {
            self.crono.zerar();
        }
    }

    fn mudar_velocidade(&mut self, delta: f32) {
        self.cfg.velocidade = (self.cfg.velocidade + delta).clamp(0.0, 10.0);
    }

    /// Mouse sobre o texto (prévia ou apresentador), no mesmo jeito do Apotec:
    /// arrastar move o texto, a rodinha muda a velocidade, clique na rodinha para.
    fn mouse_no_texto(&mut self, ui: &Ui, resp: &egui::Response, px: f32) {
        if resp.dragged_by(egui::PointerButton::Primary) {
            self.rolagem_em -= resp.drag_delta().y / px;
        }
        if resp.hovered() {
            // Cada "clique" da rodinha: para cima acelera, para baixo freia.
            let passos: f32 = ui.input(|i| {
                i.events
                    .iter()
                    .filter_map(|e| match e {
                        egui::Event::MouseWheel { delta, .. } if delta.y != 0.0 => {
                            Some(delta.y.signum())
                        }
                        _ => None,
                    })
                    .sum()
            });
            if passos != 0.0 {
                self.mudar_velocidade(passos * 0.25);
            }
        }
        if resp.clicked_by(egui::PointerButton::Middle) {
            self.rodando = false;
        }
        if resp.double_clicked() {
            self.alternar_rolagem();
        }
    }

    fn atalhos(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        // Mesmas teclas do Apotec, para a equipe não precisar reaprender.
        let (espaco, cima, baixo, dir, esq, fim, pgup, pgdn, home, f5) = ctx.input(|i| {
            (
                i.key_pressed(Key::Space),
                i.key_down(Key::ArrowUp),
                i.key_down(Key::ArrowDown),
                i.key_pressed(Key::ArrowRight),
                i.key_pressed(Key::ArrowLeft),
                i.key_pressed(Key::End),
                i.key_pressed(Key::PageUp),
                i.key_pressed(Key::PageDown),
                i.key_pressed(Key::Home),
                i.key_pressed(Key::F5),
            )
        });
        if espaco {
            self.alternar_rolagem();
        }
        // Setas ↑/↓ seguradas movem o texto (≈ 3 linhas por segundo).
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        if cima {
            self.rolagem_em -= 4.0 * dt;
            ctx.request_repaint();
        }
        if baixo {
            self.rolagem_em += 4.0 * dt;
            ctx.request_repaint();
        }
        if dir {
            self.mudar_velocidade(0.25);
        }
        if esq {
            self.mudar_velocidade(-0.25);
        }
        if fim {
            self.rodando = false;
        }
        if pgup {
            self.bloco_relativo(-1);
        }
        if pgdn {
            self.bloco_relativo(1);
        }
        if home {
            self.ir_inicio();
        }
        if f5 {
            self.crono.alternar();
        }
    }

    fn avancar(&mut self, ctx: &egui::Context) {
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        if self.rodando {
            self.rolagem_em += self.em_por_segundo() * dt;
            if self.medidas.total_em > 0.0 && self.rolagem_em >= self.medidas.total_em {
                self.rolagem_em = self.medidas.total_em;
                self.rodando = false;
            }
            ctx.request_repaint();
        }
        self.rolagem_em = self.rolagem_em.max(0.0);
        if self.crono.rodando() {
            ctx.request_repaint_after(Duration::from_millis(250));
        } else {
            // Relógio de parede na tela.
            ctx.request_repaint_after(Duration::from_secs(1));
        }
    }

    fn salvar_cfg_se_mudou(&mut self) {
        if self.cfg_checada.elapsed() < Duration::from_secs(3) {
            return;
        }
        self.cfg_checada = Instant::now();
        if self.cfg != self.cfg_salva {
            self.cfg.save();
            self.cfg_salva = self.cfg.clone();
        }
    }

    // ------------------------------------------------------------------
    // Tempos
    // ------------------------------------------------------------------

    fn meta_seg(&self) -> Option<i64> {
        parse_duracao(&self.meta_txt).filter(|s| *s > 0)
    }

    /// Estimativa do tempo de leitura que falta, pela posição na rolagem.
    fn leitura_restante(&self) -> i64 {
        let total_palavras = self.roteiro.palavras() as f32;
        let fracao = if self.medidas.total_em > 0.0 {
            (self.rolagem_em / self.medidas.total_em).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let restantes = total_palavras * (1.0 - fracao);
        (restantes / self.cfg.palavras_por_minuto.max(1) as f32 * 60.0).round() as i64
    }

    // ------------------------------------------------------------------
    // Telas
    // ------------------------------------------------------------------

    fn tela_login(&mut self, ui: &mut Ui) {
        CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                let p = self.p();
                ui.add_space((ui.available_height() * 0.16).max(20.0));
                if let Some(t) = &self.marca {
                    ui.add(
                        egui::Image::new(t)
                            .fit_to_exact_size(Vec2::splat(64.0))
                            .corner_radius(14.0),
                    );
                }
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Sintonia Teleprompter")
                        .font(theme::titulo(30.0))
                        .color(p.texto),
                );
                ui.label(
                    RichText::new("Entre com o mesmo usuário e senha do Sintonia")
                        .color(p.texto_fraco),
                );
                ui.add_space(22.0);
                let mut enviar = false;
                egui::Frame::new()
                    .fill(p.panel)
                    .stroke(egui::Stroke::new(1.0, p.borda))
                    .corner_radius(12.0)
                    .inner_margin(22.0)
                    .show(ui, |ui| {
                        ui.set_max_width(380.0);
                        egui::Grid::new("login")
                            .num_columns(2)
                            .spacing([10.0, 10.0])
                            .show(ui, |ui| {
                                ui.label("Servidor");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.cfg.server_url)
                                        .hint_text("http://servidor:3001")
                                        .desired_width(240.0),
                                );
                                ui.end_row();
                                ui.label("Usuário");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.cfg.username)
                                        .desired_width(240.0),
                                );
                                ui.end_row();
                                ui.label("Senha");
                                let r = ui.add(
                                    egui::TextEdit::singleline(&mut self.senha)
                                        .password(true)
                                        .desired_width(240.0),
                                );
                                if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                                    enviar = true;
                                }
                                ui.end_row();
                            });
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            let pode = !self.api.ocupado
                                && !self.cfg.username.trim().is_empty()
                                && !self.senha.is_empty();
                            if ui
                                .add_enabled(
                                    pode,
                                    egui::Button::new(
                                        RichText::new("Entrar")
                                            .font(theme::forte(14.0))
                                            .color(Color32::WHITE),
                                    )
                                    .fill(p.vermelho)
                                    .min_size(Vec2::new(110.0, 32.0)),
                                )
                                .clicked()
                            {
                                enviar = true;
                            }
                            if self.api.ocupado {
                                ui.spinner();
                            }
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button("Usar sem login (só TXT)").clicked() {
                                    self.tela = Tela::Principal;
                                }
                            });
                        });
                        if let Some(e) = &self.erro_login {
                            ui.add_space(8.0);
                            ui.colored_label(p.vermelho, e);
                        }
                    });
                if enviar && !self.api.ocupado && !self.senha.is_empty() {
                    self.erro_login = None;
                    self.api.enviar(Comando::Login {
                        url: self.cfg.server_url.clone(),
                        usuario: self.cfg.username.trim().to_string(),
                        senha: self.senha.clone(),
                    });
                }
            });
        });
    }

    fn barra_menu(&mut self, ui: &mut Ui) {
        let p = self.p();
        egui::MenuBar::new().ui(ui, |ui| {
            self.marca(ui, 34.0);
            ui.add_space(14.0);
            ui.menu_button("Arquivo", |ui| {
                if ui.button("Novo roteiro").clicked() {
                    self.pedir(Pendente::Novo);
                    ui.close();
                }
                if ui.button("Abrir TXT…").clicked() {
                    ui.close();
                    self.pedir(Pendente::AbrirTxt);
                }
                if ui.button("Salvar como TXT…").clicked() {
                    ui.close();
                    self.salvar_txt();
                }
            });
            ui.menu_button("Sintonia", |ui| {
                let logado = self.usuario.is_some();
                if ui.add_enabled(logado, egui::Button::new("Abrir espelho…")).clicked() {
                    self.seletor_aberto = true;
                    self.api.enviar(Comando::ListarEspelhos);
                    ui.close();
                }
                let id_espelho = match &self.roteiro.origem {
                    Origem::Espelho { id, .. } => Some(id.clone()),
                    _ => None,
                };
                if ui.add_enabled(logado && id_espelho.is_some(), egui::Button::new("Recarregar espelho atual")).clicked() {
                    ui.close();
                    if self.editado {
                        self.avisar("Há textos alterados. Salve no Sintonia (ou em TXT) antes de recarregar.", true);
                    } else {
                        self.recarregar_id = id_espelho.clone();
                        self.api.enviar(Comando::ListarEspelhos);
                    }
                }
                let alterados = self.roteiro.alterados();
                let pode_salvar = logado && id_espelho.is_some() && self.pode_editar_espelho() && alterados > 0;
                if ui
                    .add_enabled(pode_salvar, egui::Button::new(format!("Gravar textos alterados no espelho ({alterados})")))
                    .on_disabled_hover_text("Precisa estar logado, com um espelho aberto, permissão de Editar no Espelho e algum texto alterado.")
                    .clicked()
                {
                    ui.close();
                    self.salvar_no_sintonia();
                }
                ui.separator();
                if logado {
                    if ui.button("Sair do Sintonia").clicked() {
                        ui.close();
                        self.api.enviar(Comando::Sair);
                    }
                } else if ui.button("Entrar no Sintonia…").clicked() {
                    ui.close();
                    self.tela = Tela::Login;
                }
            });
            ui.menu_button("Apresentador", |ui| {
                let txt = if self.apresentador_aberto { "Fechar tela do apresentador" } else { "Abrir tela do apresentador" };
                if ui.button(txt).clicked() {
                    self.apresentador_aberto = !self.apresentador_aberto;
                    ui.close();
                }
                ui.checkbox(&mut self.cfg.tela_cheia_apresentador, "Tela cheia (F11)");
                ui.horizontal(|ui| {
                    ui.label("Monitor:");
                    ui.add(egui::DragValue::new(&mut self.cfg.monitor_apresentador).range(0..=8));
                });
                ui.label(RichText::new("0 = primeiro monitor, 1 = segundo…\nVale ao entrar em tela cheia.").small().weak());
            });
            ui.menu_button("Aparência", |ui| {
                ui.selectable_value(&mut self.cfg.tema_claro, false, "Tema escuro");
                ui.selectable_value(&mut self.cfg.tema_claro, true, "Tema claro (fundo branco)");
                ui.label(RichText::new("Só a tela do operador. As cores do texto do\napresentador ficam em Formatação.").small().weak());
            });
            ui.menu_button("Ajuda", |ui| {
                ui.label(RichText::new("Atalhos").strong());
                for (k, d) in [
                    ("Espaço", "rolar / pausar"),
                    ("End", "para a rolagem"),
                    ("↑ / ↓ (segurar)", "move o texto para cima / baixo"),
                    ("→ / ←", "mais rápido / mais devagar"),
                    ("Home", "volta ao começo e zera o cronômetro"),
                    ("Page Up / Page Down", "lauda (bloco) anterior / próxima"),
                    ("F5", "inicia / pausa o cronômetro"),
                    ("F11 / Esc", "tela cheia do apresentador"),
                    ("Arrastar o texto", "move o texto"),
                    ("Rodinha do mouse", "muda a velocidade"),
                    ("Clique na rodinha", "para a rolagem"),
                    ("Duplo clique", "rolar / pausar"),
                ] {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(k).monospace());
                        ui.label(d);
                    });
                }
                ui.separator();
                ui.label(format!("Sintonia Teleprompter {}", env!("CARGO_PKG_VERSION")));
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                match &self.usuario {
                    Some(u) => ui.label(RichText::new(format!("👤 {}", u.nome())).font(theme::forte(13.5)).color(p.texto)),
                    None => ui.label(RichText::new("sem login (modo TXT)").color(p.texto_mais_fraco)),
                };
                // "NO AR" pulsando, como o relógio da barra lateral do Sintonia.
                if self.rodando {
                    let t = ui.input(|i| i.time) as f32;
                    let alfa = 0.35 + 0.65 * (t * std::f32::consts::PI / 1.6).cos().abs();
                    ui.label(RichText::new("NO AR").size(11.5).strong().color(p.vermelho));
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                    ui.painter().circle_filled(r.center(), 3.5, p.vermelho.gamma_multiply(alfa));
                    ui.separator();
                }
                if self.api.ocupado {
                    ui.spinner();
                }
                let titulo = if self.editado { format!("{} •", self.roteiro.titulo) } else { self.roteiro.titulo.clone() };
                ui.label(RichText::new(titulo).color(p.texto_fraco));
            });
        });
    }

    fn painel_transporte(&mut self, ui: &mut Ui) {
        let p = self.p();
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let grande = |t: &str| RichText::new(t).size(18.0);
            if ui
                .button(grande("⏮"))
                .on_hover_text("Começo — zera o cronômetro (Home)")
                .clicked()
            {
                self.ir_inicio();
            }
            if ui
                .button(grande("⏪"))
                .on_hover_text("Bloco anterior (Page Up)")
                .clicked()
            {
                self.bloco_relativo(-1);
            }
            // Botão principal no vermelho do Sintonia (.btn-primary); pausado vira âmbar.
            let (txt, cor_btn) = if self.rodando {
                ("⏸  Pausar", p.ambar)
            } else {
                ("▶  Rolar", p.vermelho)
            };
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(txt)
                            .font(theme::forte(17.0))
                            .color(Color32::WHITE),
                    )
                    .fill(cor_btn)
                    .min_size(Vec2::new(130.0, 34.0)),
                )
                .on_hover_text("Espaço (End para)")
                .clicked()
            {
                self.alternar_rolagem();
            }
            if ui
                .button(grande("⏩"))
                .on_hover_text("Próximo bloco (Page Down)")
                .clicked()
            {
                self.bloco_relativo(1);
            }
            ui.separator();
            ui.label(RichText::new("Velocidade").color(p.texto_fraco))
                .on_hover_text("→ / ← ou rodinha do mouse");
            ui.add(
                egui::Slider::new(&mut self.cfg.velocidade, 0.0..=10.0)
                    .step_by(0.05)
                    .fixed_decimals(1),
            );
            ui.separator();
            let progresso = if self.medidas.total_em > 0.0 {
                self.rolagem_em / self.medidas.total_em
            } else {
                0.0
            };
            ui.add(
                egui::ProgressBar::new(progresso.clamp(0.0, 1.0))
                    .desired_width(140.0)
                    .show_percentage(),
            );
        });
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            let mono =
                |t: String, c: Color32| RichText::new(t).font(FontId::monospace(24.0)).color(c);
            let rotulo = |t: &str| {
                RichText::new(t.to_uppercase())
                    .size(10.5)
                    .color(p.texto_mais_fraco)
            };
            ui.label(rotulo("Cronômetro"));
            let decorrido = self.crono.decorrido().as_secs() as i64;
            ui.label(mono(fmt_tempo(decorrido), p.texto));
            let (t, dica) = if self.crono.rodando() {
                ("⏸", "Pausar (F5)")
            } else {
                ("▶", "Iniciar (F5)")
            };
            if ui
                .button(RichText::new(t).size(18.0))
                .on_hover_text(dica)
                .clicked()
            {
                self.crono.alternar();
            }
            if ui
                .button(RichText::new("↺").size(18.0))
                .on_hover_text("Zerar")
                .clicked()
            {
                self.crono.zerar();
            }
            ui.separator();
            ui.label(rotulo("Tempo do programa"));
            ui.add(
                egui::TextEdit::singleline(&mut self.meta_txt)
                    .hint_text("mm:ss")
                    .desired_width(64.0),
            );
            if let Some(meta) = self.meta_seg() {
                let resta = meta - decorrido;
                let c = if resta < 0 {
                    p.vermelho
                } else if resta <= 60 {
                    p.ambar
                } else {
                    p.verde
                };
                ui.label(rotulo("Falta"));
                ui.label(mono(fmt_tempo(resta), c));
            }
            ui.separator();
            ui.label(rotulo("Leitura (estimada)"));
            ui.label(mono(fmt_tempo(self.leitura_restante()), p.azul))
                .on_hover_text(format!(
                    "{} palavras a {} palavras/min",
                    self.roteiro.palavras(),
                    self.cfg.palavras_por_minuto
                ));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(mono(
                    chrono::Local::now().format("%H:%M:%S").to_string(),
                    p.texto,
                ));
                ui.label(rotulo("Agora"));
            });
        });
        ui.add_space(4.0);
    }

    fn painel_roteiro(&mut self, ui: &mut Ui) {
        let p = self.p();
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.heading("Roteiro");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .button("➕ Bloco")
                    .on_hover_text("Adiciona um bloco de texto depois do selecionado")
                    .clicked()
                {
                    let pos = self
                        .selecionado
                        .map(|i| i + 1)
                        .unwrap_or(self.roteiro.segmentos.len());
                    self.roteiro
                        .segmentos
                        .insert(pos, Segmento::novo_texto("NOVO BLOCO", ""));
                    self.selecionado = Some(pos);
                    self.editado = true;
                }
            });
        });
        match &self.roteiro.origem {
            Origem::Espelho { titulo, .. } => ui.label(
                RichText::new(format!("Espelho: {titulo}"))
                    .small()
                    .color(p.texto_mais_fraco),
            ),
            Origem::Arquivo(p) => ui.label(
                RichText::new(format!("Arquivo: {}", p.display()))
                    .small()
                    .weak(),
            ),
            Origem::Nenhuma => ui.label(RichText::new("Roteiro digitado").small().weak()),
        };
        ui.separator();

        let atual = self.medidas.bloco_atual(self.rolagem_em);
        let altura_lista = (ui.available_height() * 0.45).max(120.0);
        let mut ir_para = None;
        ScrollArea::vertical()
            .id_salt("lista")
            .max_height(altura_lista)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for (i, seg) in self.roteiro.segmentos.iter().enumerate() {
                    let icone = match seg.tipo {
                        Tipo::Materia => "📰",
                        Tipo::Entrevistado => "🎤",
                        Tipo::Manual => "📝",
                        Tipo::Intervalo => "📺",
                        Tipo::Texto => "📄",
                    };
                    let nome = if seg.titulo.trim().is_empty() {
                        "(sem título)"
                    } else {
                        seg.titulo.trim()
                    };
                    // Linha no estilo do menu lateral do Sintonia (.nav-item / .nav-item.active).
                    let (rect, r) = ui
                        .allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::click());
                    let selecionado = self.selecionado == Some(i);
                    let no_ar = atual == Some(i);
                    if selecionado || r.hovered() {
                        ui.painter().rect_filled(rect, 7.0, p.panel_2);
                    }
                    if selecionado {
                        ui.painter().rect_filled(
                            Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height())),
                            1.0,
                            p.vermelho,
                        );
                    }
                    let cor_txt = if no_ar {
                        p.ambar
                    } else if selecionado || r.hovered() {
                        p.texto
                    } else {
                        p.texto_fraco
                    };
                    let fonte = if no_ar || selecionado {
                        theme::forte(13.5)
                    } else {
                        FontId::proportional(13.5)
                    };
                    let painter = ui.painter_at(rect);
                    painter.text(
                        rect.left_center() + Vec2::new(10.0, 0.0),
                        Align2::LEFT_CENTER,
                        format!("{:>2}  {icone}  {nome}", i + 1),
                        fonte,
                        cor_txt,
                    );
                    let mut x = rect.right() - 8.0;
                    if !seg.duracao.trim().is_empty() {
                        let g = painter.layout_no_wrap(
                            seg.duracao.trim().to_string(),
                            FontId::monospace(11.5),
                            p.texto_mais_fraco,
                        );
                        x -= g.size().x;
                        painter.galley(
                            egui::pos2(x, rect.center().y - g.size().y / 2.0),
                            g,
                            p.texto_mais_fraco,
                        );
                        x -= 8.0;
                    }
                    if seg.alterado() {
                        painter.circle_filled(egui::pos2(x - 4.0, rect.center().y), 3.5, p.ambar);
                    }
                    if r.clicked() {
                        ir_para = Some(i);
                    }
                    let r = if seg.alterado() {
                        r.on_hover_text("Texto alterado (ainda não gravado no Sintonia)")
                    } else {
                        r
                    };
                    let _ = r;
                }
            });
        if let Some(i) = ir_para {
            self.selecionado = Some(i);
            self.ir_para_bloco(i);
        }

        ui.separator();
        let Some(i) = self
            .selecionado
            .filter(|i| *i < self.roteiro.segmentos.len())
        else {
            ui.label(RichText::new("Selecione um bloco para editar.").weak());
            return;
        };
        let total = self.roteiro.segmentos.len();
        let mut mover: Option<(usize, usize)> = None;
        let mut excluir = false;
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "Bloco {} — {}",
                    i + 1,
                    self.roteiro.segmentos[i].tipo.rotulo()
                ))
                .font(theme::forte(13.5)),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .small_button("🗑")
                    .on_hover_text("Tirar do teleprompter (não apaga do Sintonia)")
                    .clicked()
                {
                    excluir = true;
                }
                if ui
                    .add_enabled(i + 1 < total, egui::Button::new("⬇").small())
                    .clicked()
                {
                    mover = Some((i, i + 1));
                }
                if ui
                    .add_enabled(i > 0, egui::Button::new("⬆").small())
                    .clicked()
                {
                    mover = Some((i, i - 1));
                }
            });
        });
        let seg = &mut self.roteiro.segmentos[i];
        let r1 = ui.add(
            egui::TextEdit::singleline(&mut seg.titulo)
                .hint_text("Título / retranca")
                .desired_width(f32::INFINITY),
        );
        if seg.item_id.is_some() && r1.changed() {
            // A retranca não é gravada de volta; só o texto.
            r1.on_hover_text("O título é só para o teleprompter — não é gravado no espelho.");
        }
        if seg.tipo == Tipo::Intervalo {
            ui.label(
                RichText::new("Intervalo comercial (break). O apresentador vê só a marcação.")
                    .weak(),
            );
        }
        let r2 = ScrollArea::vertical()
            .id_salt("editor")
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut seg.texto)
                        .hint_text("Texto que o apresentador vai ler…")
                        .desired_width(f32::INFINITY)
                        .desired_rows(12)
                        .font(FontId::proportional(16.0)),
                )
            })
            .inner;
        if r2.changed() {
            self.editado = true;
        }
        if let Some((a, b)) = mover {
            self.roteiro.segmentos.swap(a, b);
            self.selecionado = Some(b);
            self.editado = true;
        }
        if excluir {
            self.roteiro.segmentos.remove(i);
            self.selecionado = if self.roteiro.segmentos.is_empty() {
                None
            } else {
                Some(i.min(self.roteiro.segmentos.len() - 1))
            };
            self.editado = true;
        }
    }

    fn painel_formatacao(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(6.0);
            ui.heading("Formatação");
            ui.add_space(4.0);

            ui.label("Fonte");
            let automatica = self.fontes.automatica().unwrap_or("padrão").to_string();
            let atual = if self.cfg.font_family.is_empty() {
                format!("Automática ({automatica})")
            } else {
                self.cfg.font_family.clone()
            };
            egui::ComboBox::from_id_salt("fonte")
                .selected_text(atual)
                .width(ui.available_width() - 8.0)
                .height(360.0)
                .show_ui(ui, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.busca_fonte)
                            .hint_text("🔍 procurar fonte"),
                    );
                    if ui
                        .selectable_label(
                            self.cfg.font_family.is_empty(),
                            format!("Automática ({automatica})"),
                        )
                        .clicked()
                    {
                        self.cfg.font_family.clear();
                    }
                    let busca = self.busca_fonte.to_lowercase();
                    for f in self
                        .fontes
                        .familias
                        .iter()
                        .filter(|f| busca.is_empty() || f.to_lowercase().contains(&busca))
                    {
                        if ui.selectable_label(&self.cfg.font_family == f, f).clicked() {
                            self.cfg.font_family = f.clone();
                        }
                    }
                });
            ui.checkbox(&mut self.cfg.negrito, "Negrito");
            ui.add_space(4.0);
            ui.label("Tamanho da letra (na tela do apresentador)");
            ui.add(
                egui::Slider::new(&mut self.cfg.font_size, 20.0..=220.0)
                    .suffix(" px")
                    .step_by(1.0),
            );
            ui.horizontal(|ui| {
                for (rot, v) in [("P", 48.0), ("M", 72.0), ("G", 100.0), ("GG", 140.0)] {
                    if ui.small_button(rot).clicked() {
                        self.cfg.font_size = v;
                    }
                }
            });
            ui.label("Espaço entre linhas");
            ui.add(egui::Slider::new(&mut self.cfg.entrelinha, 1.0..=2.5).step_by(0.05));
            ui.label("Margem lateral");
            ui.add(
                egui::Slider::new(&mut self.cfg.margem_pct, 0.0..=30.0)
                    .suffix(" %")
                    .step_by(0.5),
            );
            ui.horizontal(|ui| {
                ui.label("Alinhamento");
                ui.selectable_value(&mut self.cfg.alinhamento, Alinhamento::Esquerda, "Esquerda");
                ui.selectable_value(&mut self.cfg.alinhamento, Alinhamento::Centro, "Centro");
            });
            ui.checkbox(&mut self.cfg.caixa_alta, "CAIXA ALTA");
            ui.checkbox(&mut self.cfg.mostrar_titulos, "Mostrar retrancas (títulos)");

            ui.add_space(6.0);
            egui::Grid::new("cores").num_columns(2).show(ui, |ui| {
                ui.label("Cor do texto");
                ui.color_edit_button_srgb(&mut self.cfg.cor_texto);
                ui.end_row();
                ui.label("Cor do fundo");
                ui.color_edit_button_srgb(&mut self.cfg.cor_fundo);
                ui.end_row();
                ui.label("Cor dos títulos");
                ui.color_edit_button_srgb(&mut self.cfg.cor_titulo);
                ui.end_row();
                ui.label("Cor do destaque")
                    .on_hover_text("Texto entre << e >> no roteiro");
                ui.color_edit_button_srgb(&mut self.cfg.cor_destaque);
                ui.end_row();
            });
            ui.horizontal(|ui| {
                if ui.small_button("Branco no preto").clicked() {
                    self.cfg.cor_texto = [255, 255, 255];
                    self.cfg.cor_fundo = [0, 0, 0];
                }
                if ui.small_button("Amarelo no preto").clicked() {
                    self.cfg.cor_texto = [255, 230, 0];
                    self.cfg.cor_fundo = [0, 0, 0];
                }
                if ui.small_button("Preto no branco").clicked() {
                    self.cfg.cor_texto = [0, 0, 0];
                    self.cfg.cor_fundo = [255, 255, 255];
                }
            });

            ui.separator();
            ui.heading("Leitura");
            ui.checkbox(
                &mut self.cfg.espelhar_horizontal,
                "Espelhar horizontal (vidro)",
            );
            ui.checkbox(&mut self.cfg.espelhar_vertical, "Inverter vertical");
            ui.label(
                RichText::new("O espelhamento vale só na tela do apresentador.")
                    .small()
                    .weak(),
            );
            ui.checkbox(&mut self.cfg.mostrar_marcador, "Seta de leitura");
            ui.label("Altura da linha de leitura");
            ui.add(egui::Slider::new(&mut self.cfg.marcador_pct, 5.0..=70.0).suffix(" %"));
            ui.checkbox(
                &mut self.cfg.crono_automatico,
                "Cronômetro dispara ao rolar (Home zera)",
            );
            ui.label("Ritmo para estimar a leitura");
            ui.add(
                egui::Slider::new(&mut self.cfg.palavras_por_minuto, 80..=250)
                    .suffix(" palavras/min"),
            );

            ui.separator();
            ui.heading("Tela do apresentador");
            let txt = if self.apresentador_aberto {
                "Fechar tela do apresentador"
            } else {
                "Abrir tela do apresentador"
            };
            let p = self.p();
            let (cor_txt, fundo) = if self.apresentador_aberto {
                (p.texto, p.panel_2)
            } else {
                (Color32::WHITE, p.vermelho)
            };
            if ui
                .add(
                    egui::Button::new(RichText::new(txt).font(theme::forte(14.0)).color(cor_txt))
                        .fill(fundo)
                        .min_size(Vec2::new(ui.available_width(), 32.0)),
                )
                .clicked()
            {
                self.apresentador_aberto = !self.apresentador_aberto;
            }
            ui.checkbox(&mut self.cfg.tela_cheia_apresentador, "Tela cheia (F11)");
            ui.horizontal(|ui| {
                ui.label("Monitor");
                ui.add(egui::DragValue::new(&mut self.cfg.monitor_apresentador).range(0..=8));
                ui.label(RichText::new("(0 = principal)").small().weak());
            });
            ui.checkbox(
                &mut self.cfg.mostrar_relogio_apresentador,
                "Mostrar cronômetro para o apresentador",
            );
            ui.label(
                RichText::new(format!(
                    "Tela do apresentador: {:.0} × {:.0}",
                    self.tamanho_apresentador.x, self.tamanho_apresentador.y
                ))
                .small()
                .weak(),
            );
        });
    }

    fn previa(&mut self, ui: &mut Ui) {
        let mut disponivel = ui.available_rect_before_wrap();
        disponivel.min.y += 18.0; // espaço para o rótulo acima da prévia
        let proporcao =
            (self.tamanho_apresentador.x / self.tamanho_apresentador.y.max(1.0)).clamp(0.3, 4.0);
        let mut tam = Vec2::new(disponivel.width(), disponivel.width() / proporcao);
        if tam.y > disponivel.height() {
            tam = Vec2::new(disponivel.height() * proporcao, disponivel.height());
        }
        let rect = Rect::from_center_size(disponivel.center(), tam);
        let resp = ui.allocate_rect(rect, Sense::click_and_drag());
        let escala = rect.width() / self.tamanho_apresentador.x.max(1.0);
        let familia = self
            .fontes
            .familia(ui.ctx(), &self.cfg.font_family, self.cfg.negrito);
        let c = &self.cfg;
        let assinatura = format!(
            "{familia:?}|{}|{}|{}|{:?}|{}|{}|{}",
            c.font_size,
            c.entrelinha,
            c.margem_pct,
            c.alinhamento,
            c.caixa_alta,
            c.mostrar_titulos,
            self.tamanho_apresentador.x
        );
        // Onde estava a leitura (bloco + fração dentro dele), antes da formatação mudar.
        let ancora = if assinatura != self.assinatura_layout {
            self.assinatura_layout = assinatura;
            self.medidas.bloco_atual(self.rolagem_em).map(|b| {
                let ini = self.medidas.topos_em[b];
                let fim = self
                    .medidas
                    .topos_em
                    .get(b + 1)
                    .copied()
                    .unwrap_or(self.medidas.total_em);
                (
                    b,
                    ((self.rolagem_em - ini) / (fim - ini).max(0.01)).clamp(0.0, 1.0),
                )
            })
        } else {
            None
        };
        let mut medidas = prompter::desenhar(
            ui,
            rect,
            &self.roteiro,
            &self.cfg,
            &familia,
            self.rolagem_em,
            escala,
            Espelho::nenhum(rect),
        );
        if let Some((b, fracao)) = ancora
            && let Some(ini) = medidas.topos_em.get(b).copied()
        {
            let fim = medidas
                .topos_em
                .get(b + 1)
                .copied()
                .unwrap_or(medidas.total_em);
            self.rolagem_em = ini + fracao * (fim - ini);
            medidas = prompter::desenhar(
                ui,
                rect,
                &self.roteiro,
                &self.cfg,
                &familia,
                self.rolagem_em,
                escala,
                Espelho::nenhum(rect),
            );
        }
        // As medidas da tela do apresentador (escala 1) valem igual — tudo é proporcional.
        self.medidas = medidas;

        let px = (self.cfg.font_size * escala).max(4.0);
        self.mouse_no_texto(ui, &resp, px);
        let p = self.p();
        ui.painter().rect_stroke(
            rect,
            4.0,
            egui::Stroke::new(
                1.0,
                if self.apresentador_aberto {
                    p.vermelho
                } else {
                    p.borda
                },
            ),
            egui::StrokeKind::Outside,
        );
        ui.painter().text(
            rect.left_top() + Vec2::new(0.0, -4.0),
            Align2::LEFT_BOTTOM,
            if self.apresentador_aberto {
                "● PRÉVIA — NO AR NA TELA DO APRESENTADOR"
            } else {
                "PRÉVIA — TELA DO APRESENTADOR FECHADA"
            },
            FontId::proportional(11.0),
            if self.apresentador_aberto {
                p.vermelho
            } else {
                p.texto_mais_fraco
            },
        );
    }

    fn janela_apresentador(&mut self, ctx: &egui::Context) {
        if !self.apresentador_aberto {
            return;
        }
        let mut builder = ViewportBuilder::default()
            .with_title("Sintonia Teleprompter — Apresentador")
            .with_inner_size([1280.0, 720.0]);
        if self.cfg.tela_cheia_apresentador {
            builder = builder
                .with_fullscreen(true)
                .with_monitor(self.cfg.monitor_apresentador);
        }
        let id = ViewportId::from_hash_of(ID_APRESENTADOR);
        ctx.show_viewport_immediate(id, builder, |ui, _classe| {
            let ctx = ui.ctx().clone();
            if ctx.input(|i| i.viewport().close_requested()) {
                self.apresentador_aberto = false;
                return;
            }
            self.atalhos(&ctx);
            let (f11, esc) = ctx.input(|i| (i.key_pressed(Key::F11), i.key_pressed(Key::Escape)));
            if f11 {
                self.cfg.tela_cheia_apresentador = !self.cfg.tela_cheia_apresentador;
            }
            if esc {
                self.cfg.tela_cheia_apresentador = false;
            }
            CentralPanel::default()
                .frame(egui::Frame::NONE.fill(cor(self.cfg.cor_fundo)))
                .show(ui, |ui| {
                    let rect = ui.max_rect();
                    self.tamanho_apresentador = rect.size();
                    let resp = ui.allocate_rect(rect, Sense::click_and_drag());
                    let familia =
                        self.fontes
                            .familia(&ctx, &self.cfg.font_family, self.cfg.negrito);
                    let espelho = Espelho {
                        rect,
                        horizontal: self.cfg.espelhar_horizontal,
                        vertical: self.cfg.espelhar_vertical,
                    };
                    prompter::desenhar(
                        ui,
                        rect,
                        &self.roteiro,
                        &self.cfg,
                        &familia,
                        self.rolagem_em,
                        1.0,
                        espelho,
                    );
                    self.mouse_no_texto(ui, &resp, self.cfg.font_size.max(4.0));
                    if self.cfg.mostrar_relogio_apresentador {
                        self.relogio_apresentador(ui, rect, espelho);
                    }
                });
        });
    }

    fn relogio_apresentador(&self, ui: &Ui, rect: Rect, espelho: Espelho) {
        let decorrido = self.crono.decorrido().as_secs() as i64;
        let (txt, c) = match self.meta_seg() {
            Some(meta) => {
                let resta = meta - decorrido;
                let p = theme::ESCURO;
                let c = if resta < 0 {
                    p.vermelho
                } else if resta <= 60 {
                    p.ambar
                } else {
                    p.verde
                };
                (fmt_tempo(resta), c)
            }
            None => (fmt_tempo(decorrido), Color32::from_gray(220)),
        };
        let tamanho = (rect.height() * 0.06).clamp(18.0, 64.0);
        let painter = ui.painter_at(rect);
        let galley = painter.layout_no_wrap(txt, FontId::monospace(tamanho), c);
        let pos = rect.right_top() + Vec2::new(-galley.size().x - tamanho * 0.5, tamanho * 0.3);
        let fundo = Rect::from_min_size(pos, galley.size()).expand(tamanho * 0.2);
        let a = espelho.ponto(fundo.min);
        let b = espelho.ponto(fundo.max);
        painter.rect_filled(
            Rect::from_two_pos(a, b),
            6.0,
            Color32::from_black_alpha(170),
        );
        espelho.texto(ui, &painter, pos, galley, c);
    }

    fn seletor_espelho(&mut self, ctx: &egui::Context) {
        if !self.seletor_aberto {
            return;
        }
        let mut aberto = true;
        let mut escolhido: Option<Value> = None;
        egui::Window::new("Abrir espelho do Sintonia")
            .open(&mut aberto)
            .default_size([720.0, 460.0])
            .collapsible(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Data");
                    ui.add(egui::TextEdit::singleline(&mut self.filtro_data).hint_text("aaaa-mm-dd").desired_width(100.0));
                    if ui.button("Hoje").clicked() {
                        self.filtro_data = hoje();
                    }
                    if ui.button("Todas").clicked() {
                        self.filtro_data.clear();
                    }
                    ui.separator();
                    ui.add(egui::TextEdit::singleline(&mut self.filtro_busca).hint_text("🔍 programa ou título").desired_width(200.0));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("⟳ Atualizar").clicked() {
                            self.api.enviar(Comando::ListarEspelhos);
                        }
                        if self.api.ocupado {
                            ui.spinner();
                        }
                    });
                });
                ui.separator();
                let busca = self.filtro_busca.to_lowercase();
                let lista: Vec<(EspelhoResumo, &Value)> = self
                    .espelhos
                    .iter()
                    .map(|v| (EspelhoResumo::de(v), v))
                    .filter(|(r, _)| self.filtro_data.trim().is_empty() || r.data == self.filtro_data.trim())
                    .filter(|(r, _)| busca.is_empty() || r.titulo.to_lowercase().contains(&busca) || r.programa.to_lowercase().contains(&busca))
                    .collect();
                if lista.is_empty() {
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        let msg = if self.espelhos.is_empty() {
                            "Nenhum espelho carregado."
                        } else if self.filtro_data.is_empty() {
                            "Nenhum espelho encontrado."
                        } else {
                            "Nenhum espelho nessa data. Clique em \"Todas\" para ver os outros dias."
                        };
                        ui.label(RichText::new(msg).weak());
                    });
                    return;
                }
                ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("espelhos").striped(true).num_columns(5).spacing([14.0, 8.0]).show(ui, |ui| {
                        ui.label(RichText::new("Data").strong());
                        ui.label(RichText::new("Programa").strong());
                        ui.label(RichText::new("Título").strong());
                        ui.label(RichText::new("Itens").strong());
                        ui.label("");
                        ui.end_row();
                        for (r, v) in lista {
                            ui.label(format!("{} {}", data_br(&r.data), r.horario));
                            ui.label(&r.programa);
                            ui.label(&r.titulo);
                            ui.label(r.itens.to_string());
                            if ui.button("Abrir").clicked() {
                                escolhido = Some(v.clone());
                            }
                            ui.end_row();
                        }
                    });
                });
            });
        if !aberto {
            self.seletor_aberto = false;
        }
        if let Some(esp) = escolhido {
            self.seletor_aberto = false;
            self.pedir(Pendente::AbrirEspelho(esp));
        }
    }

    fn confirmacao(&mut self, ctx: &egui::Context) {
        if self.pendente.is_none() {
            return;
        }
        let mut decisao: Option<bool> = None;
        egui::Modal::new(egui::Id::unique("sintonia-confirmar")).show(ctx, |ui| {
            ui.set_width(360.0);
            ui.heading("Descartar alterações?");
            ui.label("O roteiro aberto tem textos alterados que ainda não foram salvos (nem no Sintonia, nem em TXT).");
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("Cancelar").clicked() {
                    decisao = Some(false);
                }
                if ui.button(RichText::new("Descartar e continuar").color(Color32::from_rgb(255, 120, 120))).clicked() {
                    decisao = Some(true);
                }
            });
        });
        match decisao {
            Some(true) => {
                if let Some(a) = self.pendente.take() {
                    self.editado = false;
                    self.executar(a);
                }
            }
            Some(false) => self.pendente = None,
            None => {}
        }
    }

    fn mostrar_aviso(&mut self, ctx: &egui::Context) {
        let Some(a) = &self.aviso else { return };
        let dur = if a.erro { 8 } else { 4 };
        if a.quando.elapsed() > Duration::from_secs(dur) {
            self.aviso = None;
            return;
        }
        let p = self.p();
        let (fundo, texto) = if a.erro {
            (p.vermelho, Color32::WHITE)
        } else {
            (p.verde, Color32::WHITE)
        };
        egui::Area::new(egui::Id::unique("sintonia-aviso"))
            .anchor(Align2::CENTER_TOP, Vec2::new(0.0, 40.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(fundo)
                    .corner_radius(6.0)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_max_width(520.0);
                        ui.label(RichText::new(&a.texto).color(texto));
                    });
            });
        ctx.request_repaint_after(Duration::from_millis(500));
    }

    fn tela_principal(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        self.atalhos(&ctx);
        self.avancar(&ctx);

        if self.tema_aplicado != self.cfg.tema_claro {
            theme::aplicar(&ctx, self.cfg.tema_claro);
            self.tema_aplicado = self.cfg.tema_claro;
        }
        let p = self.p();
        let painel = |m: f32| {
            egui::Frame::new()
                .fill(p.panel)
                .inner_margin(m)
                .stroke(egui::Stroke::new(1.0, p.borda))
        };
        Panel::top("menu")
            .frame(painel(8.0))
            .show(ui, |ui| self.barra_menu(ui));
        Panel::bottom("transporte")
            .frame(painel(10.0))
            .show(ui, |ui| self.painel_transporte(ui));
        Panel::left("roteiro")
            .frame(painel(12.0))
            .resizable(true)
            .default_size(340.0)
            .show(ui, |ui| self.painel_roteiro(ui));
        Panel::right("formatacao")
            .frame(painel(12.0))
            .resizable(true)
            .default_size(290.0)
            .show(ui, |ui| self.painel_formatacao(ui));
        CentralPanel::default()
            .frame(egui::Frame::new().fill(p.bg).inner_margin(16.0))
            .show(ui, |ui| self.previa(ui));

        self.janela_apresentador(&ctx);
        self.seletor_espelho(&ctx);
        self.confirmacao(&ctx);
        self.mostrar_aviso(&ctx);
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.fontes.quadro += 1;
        self.processar_respostas();
        match self.tela {
            Tela::Login => {
                let ctx = ui.ctx().clone();
                self.tela_login(ui);
                // Mesmo na tela de login, a tela do apresentador continua no ar (sessão expirada no meio do programa).
                self.avancar(&ctx);
                self.janela_apresentador(&ctx);
            }
            Tela::Principal => self.tela_principal(ui),
        }
        self.salvar_cfg_se_mudou();
    }

    fn on_exit(&mut self) {
        self.cfg.save();
    }
}
