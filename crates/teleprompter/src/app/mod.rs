//! Telas do Sintonia Teleprompter: login, operador e apresentador.
//!
//! O estado fica todo em [`App`]; cada arquivo desta pasta cuida de uma parte
//! da tela (veja `docs/ARQUITETURA.md`).

mod apresentador;
mod controle;
mod cronometros;
mod dialogos;
mod documento;
mod formatacao;
mod login;
mod menu;
mod painel_roteiro;
mod previa;
mod rede;
mod seletor_espelho;
mod transporte;

use crate::config::{Alinhamento, Renderizador, Settings};
use crate::visual::fontes::Fontes;
use crate::visual::icones::{self, Icone};
use crate::visual::prompter::{self, Espelho, Medidas, cor};
use crate::visual::tema::{self, Paleta};
use egui::{
    Align, Align2, CentralPanel, Color32, FontId, Key, Layout, Panel, Rect, RichText, ScrollArea,
    Sense, Ui, Vec2, ViewportBuilder, ViewportId,
};
use login::EstadoLogin;
use serde_json::Value;
use sintonia_api::{Api, Comando, FalhaLogin, Resposta, Usuario};
use sintonia_roteiro::tempo::{Cronometro, fmt_tempo, parse_duracao};
use sintonia_roteiro::{EspelhoResumo, Origem, Roteiro, Segmento, Tipo, decodificar_txt};
use std::time::{Duration, Instant};

const ID_APRESENTADOR: &str = "sintonia-apresentador";

/// Velocidade em 100 %, em "em" (altura da letra) por segundo.
const VELOCIDADE_MAXIMA_EM: f32 = 8.0;
/// "Passar rápido" (segurar avançar/voltar, o botão direito do mouse ou Shift+↓/↑).
const VELOCIDADE_TURBO_EM: f32 = 20.0;
/// Velocidades prontas dos botões da barra de baixo.
const VELOCIDADES_PRONTAS: [(&str, f32); 4] = [
    ("Lenta", 8.0),
    ("Normal", 15.0),
    ("Rápida", 35.0),
    ("Máx.", 100.0),
];
const DIAS_SEMANA: [&str; 7] = [
    "Segunda-feira",
    "Terça-feira",
    "Quarta-feira",
    "Quinta-feira",
    "Sexta-feira",
    "Sábado",
    "Domingo",
];

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
    login: EstadoLogin,

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
    /// Monitores conectados (lidos do sistema a cada 2 s).
    monitores: Vec<InfoMonitor>,
    monitores_lidos: Option<Instant>,
    /// Momento do último avanço da rolagem (o tempo real, não o estimado).
    ultimo_avanco: Option<Instant>,
    /// Reabrir a tela do apresentador no próximo quadro (troca de monitor).
    reabrir_apresentador: bool,

    crono: Cronometro,
    meta_txt: String,
    /// Tela do apresentador mostrando só o cronômetro (tempo de entrevista).
    so_cronometro: bool,
    /// "Passar rápido": direção pedida neste quadro (+1 frente, -1 trás, 0 nada)
    /// e a que vale no quadro seguinte.
    turbo_pedido: f32,
    turbo: f32,
    /// O botão segurado já virou "passar rápido" (então soltar não conta como clique).
    turbo_usado: bool,
    /// Espelho marcado no seletor (para ver os itens antes de abrir).
    espelho_marcado: Option<String>,
    filtro_programa: Option<String>,
    crono_entrevista: Cronometro,
    meta_entrevista_txt: String,

    /// Nome digitado para salvar uma predefinição.
    predef_nome: String,

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

/// Um monitor conectado, para o operador escolher onde vai a tela do apresentador.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct InfoMonitor {
    pub(super) nome: String,
    pub(super) largura: u32,
    pub(super) altura: u32,
    pub(super) principal: bool,
    /// É o monitor onde está a janela do operador.
    pub(super) do_operador: bool,
}

fn hoje() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let cfg = Settings::load();
        let fontes = Fontes::carregar_sistema();
        cc.egui_ctx.set_fonts(fontes.definicoes());
        tema::aplicar(&cc.egui_ctx, cfg.tema_claro);
        // Texto rolando em frações de pixel: sem isso ele anda "em degraus"
        // (1 px a cada poucos quadros), o que parece engasgado em velocidade baixa.
        cc.egui_ctx
            .tessellation_options_mut(|o| o.round_text_to_pixels = false);
        let marca = eframe::icon_data::from_png_bytes(tema::ICONE_PNG)
            .ok()
            .map(|ic| {
                let img = egui::ColorImage::from_rgba_unmultiplied(
                    [ic.width as usize, ic.height as usize],
                    &ic.rgba,
                );
                cc.egui_ctx
                    .load_texture("sintonia-marca", img, egui::TextureOptions::LINEAR)
            });
        let mut api = {
            let ctx = cc.egui_ctx.clone();
            Api::iniciar(move || ctx.request_repaint())
        };
        if cfg.lembrar_login && !cfg.username.trim().is_empty() {
            api.enviar(Comando::LerSenhaGuardada {
                url: cfg.server_url.clone(),
                usuario: cfg.username.trim().to_string(),
            });
        }
        let entrar_sozinho =
            cfg.lembrar_login && cfg.entrar_automatico && !cfg.username.trim().is_empty();
        Self {
            marca,
            tema_aplicado: cfg.tema_claro,
            cfg_salva: cfg.clone(),
            cfg,
            cfg_checada: Instant::now(),
            api,
            tela: Tela::Login,
            usuario: None,
            login: EstadoLogin::novo(entrar_sozinho),
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
            monitores: Vec::new(),
            monitores_lidos: None,
            ultimo_avanco: None,
            reabrir_apresentador: false,
            crono: Cronometro::default(),
            meta_txt: String::new(),
            so_cronometro: false,
            turbo_pedido: 0.0,
            turbo: 0.0,
            turbo_usado: false,
            espelho_marcado: None,
            filtro_programa: None,
            crono_entrevista: Cronometro::default(),
            meta_entrevista_txt: String::new(),
            predef_nome: String::new(),
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

    /// Etiqueta do tipo do bloco (MAT, ENT, INT...) desenhada a partir de `x`,
    /// centralizada em `y`. Devolve a largura usada.
    fn etiqueta_tipo(painter: &egui::Painter, x: f32, y: f32, tipo: Tipo, p: Paleta) -> f32 {
        let cor = match tipo {
            Tipo::Intervalo => p.ambar,
            Tipo::Entrevistado => p.azul,
            Tipo::Materia => p.verde,
            _ => p.texto_fraco,
        };
        let g = painter.layout_no_wrap(tipo.sigla().to_string(), FontId::monospace(10.0), cor);
        let caixa = Rect::from_min_size(
            egui::pos2(x, y - g.size().y / 2.0 - 2.0),
            g.size() + Vec2::new(10.0, 4.0),
        );
        painter.rect(
            caixa,
            4.0,
            cor.gamma_multiply(0.12),
            egui::Stroke::new(1.0, cor.gamma_multiply(0.5)),
            egui::StrokeKind::Inside,
        );
        painter.galley(caixa.min + Vec2::new(5.0, 2.0), g, cor);
        caixa.width()
    }

    fn p(&self) -> Paleta {
        tema::paleta(self.cfg.tema_claro)
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
                        .font(tema::titulo(tamanho * 0.48))
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

    fn tela_principal(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        self.atalhos(&ctx);
        self.avancar(&ctx);

        if self.tema_aplicado != self.cfg.tema_claro {
            tema::aplicar(&ctx, self.cfg.tema_claro);
            self.tema_aplicado = self.cfg.tema_claro;
        }
        // F11: tela cheia da janela do operador (a do apresentador tem o seu F11).
        if !ctx.egui_wants_keyboard_input() && ctx.input(|i| i.key_pressed(Key::F11)) {
            self.alternar_tela_cheia_operador(&ctx);
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
        if self.cfg.lateral_recolhida {
            Panel::left("roteiro-recolhido")
                .frame(painel(6.0))
                .resizable(false)
                .exact_size(48.0)
                .show(ui, |ui| self.painel_roteiro_recolhido(ui));
        } else {
            Panel::left("roteiro")
                .frame(painel(12.0))
                .resizable(true)
                .default_size(if Self::tela_baixa(ui) { 290.0 } else { 330.0 })
                .show(ui, |ui| self.painel_roteiro(ui));
        }
        if self.cfg.mostrar_formatacao {
            Panel::right("formatacao")
                .frame(painel(12.0))
                .resizable(true)
                .default_size(300.0)
                .show(ui, |ui| self.painel_formatacao(ui));
        }
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
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        self.fontes.quadro += 1;
        self.ler_monitores(frame);
        // "Passar rápido" vale enquanto alguém segura o botão/tecla: o pedido do
        // quadro anterior vira o efeito deste.
        self.turbo = self.turbo_pedido;
        self.turbo_pedido = 0.0;
        if ui.input(|i| !i.pointer.any_down() && !i.pointer.any_released()) {
            self.turbo_usado = false;
        }
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

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.cfg.save();
    }
}
