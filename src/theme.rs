//! Identidade visual do Sintonia aplicada ao teleprompter.
//!
//! As cores são as mesmas variáveis do site (`client/src/styles/index.css`):
//! tema escuro padrão e o tema claro opcional. As fontes também são as do
//! site — Space Grotesk (títulos), IBM Plex Sans (texto) e IBM Plex Mono
//! (relógios) — embutidas no executável (licença SIL OFL, ver `assets/fonts`).

use egui::epaint::text::VariationCoords;
use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, FontTweak, Stroke,
    TextStyle,
};
use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct Paleta {
    pub bg: Color32,
    pub panel: Color32,
    pub panel_2: Color32,
    pub borda: Color32,
    pub texto: Color32,
    pub texto_fraco: Color32,
    pub texto_mais_fraco: Color32,
    pub vermelho: Color32,
    pub ambar: Color32,
    pub verde: Color32,
    pub azul: Color32,
}

const fn hex(c: u32) -> Color32 {
    Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

/// `:root` do Sintonia.
pub const ESCURO: Paleta = Paleta {
    bg: hex(0x14161A),
    panel: hex(0x1B1E24),
    panel_2: hex(0x21252C),
    borda: hex(0x2A2E37),
    texto: hex(0xF0F1F3),
    texto_fraco: hex(0x9096A1),
    texto_mais_fraco: hex(0x5D636F),
    vermelho: hex(0xE4372B),
    ambar: hex(0xE8A93A),
    verde: hex(0x4CAF6D),
    azul: hex(0x4C8BE8),
};

/// `:root[data-theme="claro"]` do Sintonia.
pub const CLARO: Paleta = Paleta {
    bg: hex(0xF4F5F7),
    panel: hex(0xFFFFFF),
    panel_2: hex(0xECEEF1),
    borda: hex(0xC7CBD1),
    texto: hex(0x15171B),
    texto_fraco: hex(0x464B54),
    texto_mais_fraco: hex(0x6B7078),
    vermelho: hex(0xC22A1F),
    ambar: hex(0x96650E),
    verde: hex(0x1E7A45),
    azul: hex(0x1D5FC2),
};

pub fn paleta(claro: bool) -> Paleta {
    if claro { CLARO } else { ESCURO }
}

/// Fonte de títulos (Space Grotesk), como o `.brand-title`/`h1` do site.
pub fn titulo(tamanho: f32) -> FontId {
    FontId::new(tamanho, FontFamily::Name("sintonia-titulo".into()))
}

/// Peso forte do IBM Plex Sans (botões principais, rótulos).
pub fn forte(tamanho: f32) -> FontId {
    FontId::new(tamanho, FontFamily::Name("sintonia-forte".into()))
}

/// Nomes das fontes do Sintonia oferecidas também para o texto do teleprompter.
pub const FONTES_EMBUTIDAS: &[&str] = &["IBM Plex Sans (Sintonia)", "Space Grotesk (Sintonia)"];

const PLEX_SANS: &[u8] = include_bytes!("../assets/fonts/IBMPlexSans-Variable.ttf");
const PLEX_MONO: &[u8] = include_bytes!("../assets/fonts/IBMPlexMono-Medium.ttf");
const SPACE_GROTESK: &[u8] = include_bytes!("../assets/fonts/SpaceGrotesk-Variable.ttf");

fn variavel(bytes: &'static [u8], peso: f32) -> FontData {
    FontData::from_static(bytes).tweak(FontTweak {
        coords: VariationCoords::new([(b"wght", peso)]),
        ..Default::default()
    })
}

/// Bytes + peso de uma fonte embutida, para usar no texto do teleprompter.
pub fn fonte_embutida(nome: &str, negrito: bool) -> Option<FontData> {
    match nome {
        "IBM Plex Sans (Sintonia)" => {
            Some(variavel(PLEX_SANS, if negrito { 700.0 } else { 450.0 }))
        }
        "Space Grotesk (Sintonia)" => {
            Some(variavel(SPACE_GROTESK, if negrito { 700.0 } else { 500.0 }))
        }
        _ => None,
    }
}

/// Definições de fonte da interface: as do site primeiro, as padrão do egui de reserva (ícones/emoji).
pub fn fontes() -> FontDefinitions {
    let mut defs = FontDefinitions::default();
    let insere = |defs: &mut FontDefinitions, nome: &str, dados: FontData| {
        defs.font_data.insert(nome.to_string(), Arc::new(dados));
    };
    insere(&mut defs, "IBM Plex Sans", variavel(PLEX_SANS, 400.0));
    insere(
        &mut defs,
        "IBM Plex Sans SemiBold",
        variavel(PLEX_SANS, 600.0),
    );
    insere(&mut defs, "IBM Plex Mono", FontData::from_static(PLEX_MONO));
    insere(&mut defs, "Space Grotesk", variavel(SPACE_GROTESK, 600.0));

    let reserva = defs
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let com_reserva = |primeira: &str| {
        let mut v = vec![primeira.to_string()];
        v.extend(reserva.iter().cloned());
        v
    };
    defs.families
        .insert(FontFamily::Proportional, com_reserva("IBM Plex Sans"));
    defs.families.insert(
        FontFamily::Name("sintonia-forte".into()),
        com_reserva("IBM Plex Sans SemiBold"),
    );
    defs.families.insert(
        FontFamily::Name("sintonia-titulo".into()),
        com_reserva("Space Grotesk"),
    );
    if let Some(mono) = defs.families.get_mut(&FontFamily::Monospace) {
        mono.insert(0, "IBM Plex Mono".into());
    }
    defs
}

/// Aplica cores, cantos e tamanhos de texto do Sintonia no egui.
pub fn aplicar(ctx: &egui::Context, claro: bool) {
    let p = paleta(claro);
    let mut v = if claro {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };
    let raio = CornerRadius::same(7);

    v.panel_fill = p.bg;
    v.window_fill = p.panel;
    v.window_stroke = Stroke::new(1.0, p.borda);
    v.window_corner_radius = CornerRadius::same(10);
    v.menu_corner_radius = CornerRadius::same(8);
    v.extreme_bg_color = p.bg;
    v.text_edit_bg_color = Some(if claro { p.panel } else { p.bg });
    v.faint_bg_color = p.panel_2;
    v.code_bg_color = p.panel_2;
    v.hyperlink_color = p.azul;
    v.warn_fg_color = p.ambar;
    v.error_fg_color = p.vermelho;
    v.selection.bg_fill = p.vermelho.gamma_multiply(if claro { 0.85 } else { 0.75 });
    v.selection.stroke = Stroke::new(1.0, Color32::WHITE);

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = p.panel;
    w.noninteractive.weak_bg_fill = p.panel;
    w.noninteractive.bg_stroke = Stroke::new(1.0, p.borda);
    w.noninteractive.fg_stroke = Stroke::new(1.0, p.texto_fraco);
    w.noninteractive.corner_radius = raio;

    w.inactive.bg_fill = p.panel_2;
    w.inactive.weak_bg_fill = p.panel_2;
    w.inactive.bg_stroke = Stroke::new(1.0, p.borda);
    w.inactive.fg_stroke = Stroke::new(1.0, p.texto);
    w.inactive.corner_radius = raio;

    w.hovered.bg_fill = p.borda;
    w.hovered.weak_bg_fill = p.borda;
    w.hovered.bg_stroke = Stroke::new(1.0, p.texto_mais_fraco);
    w.hovered.fg_stroke = Stroke::new(1.5, p.texto);
    w.hovered.corner_radius = raio;

    w.active.bg_fill = p.borda;
    w.active.weak_bg_fill = p.borda;
    w.active.bg_stroke = Stroke::new(1.0, p.vermelho);
    w.active.fg_stroke = Stroke::new(1.5, p.texto);
    w.active.corner_radius = raio;

    w.open.bg_fill = p.panel_2;
    w.open.weak_bg_fill = p.panel_2;
    w.open.bg_stroke = Stroke::new(1.0, p.borda);
    w.open.fg_stroke = Stroke::new(1.0, p.texto);
    w.open.corner_radius = raio;

    ctx.all_styles_mut(|s| {
        s.visuals = v.clone();
        s.text_styles.insert(TextStyle::Heading, titulo(19.0));
        s.text_styles
            .insert(TextStyle::Body, FontId::proportional(14.0));
        s.text_styles
            .insert(TextStyle::Button, FontId::proportional(14.0));
        s.text_styles
            .insert(TextStyle::Small, FontId::proportional(11.5));
        s.text_styles
            .insert(TextStyle::Monospace, FontId::monospace(13.0));
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(10.0, 5.0);
        s.spacing.interact_size.y = 26.0;
    });
}

/// Ícone do Sintonia (torre de rádio no quadrado vermelho), igual ao do site.
pub const ICONE_PNG: &[u8] = include_bytes!("../assets/sintonia-icone.png");
