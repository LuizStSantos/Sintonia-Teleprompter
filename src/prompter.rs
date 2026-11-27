//! Desenho do texto rolando — usado tanto na tela do apresentador quanto na
//! prévia da tela do operador.
//!
//! A posição da rolagem é guardada em **"em"** (múltiplos do tamanho da
//! letra), não em pixels. Como a prévia do operador é a mesma tela do
//! apresentador em escala menor (mesma proporção, letra proporcionalmente
//! menor), a quebra de linhas é idêntica nas duas e o operador vê exatamente
//! a mesma linha que o apresentador está lendo.

use crate::script::{Roteiro, Tipo, trechos_destaque};
use crate::settings::{Alinhamento, Settings};
use egui::epaint::{Mesh, TessellationOptions, Tessellator, TextShape};
use egui::text::{LayoutJob, TextFormat};
use egui::{Align, Color32, FontFamily, FontId, Galley, Painter, Pos2, Rect, Shape, Stroke, Ui};
use std::sync::Arc;

/// Posições calculadas no último desenho (em "em", a partir do começo do texto).
#[derive(Clone, Debug, Default)]
pub struct Medidas {
    /// Onde começa cada bloco.
    pub topos_em: Vec<f32>,
    /// Altura total do texto.
    pub total_em: f32,
}

impl Medidas {
    /// Índice do bloco que está na linha de leitura.
    pub fn bloco_atual(&self, rolagem_em: f32) -> Option<usize> {
        if self.topos_em.is_empty() {
            return None;
        }
        Some(
            self.topos_em
                .iter()
                .rposition(|&t| t <= rolagem_em + 0.01)
                .unwrap_or(0),
        )
    }
}

pub fn cor(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

/// Espelhamento (para o vidro do teleprompter, que inverte a imagem do monitor).
#[derive(Clone, Copy)]
pub struct Espelho {
    pub rect: Rect,
    pub horizontal: bool,
    pub vertical: bool,
}

impl Espelho {
    pub fn nenhum(rect: Rect) -> Self {
        Self {
            rect,
            horizontal: false,
            vertical: false,
        }
    }

    pub fn ativo(&self) -> bool {
        self.horizontal || self.vertical
    }

    pub fn ponto(&self, p: Pos2) -> Pos2 {
        Pos2::new(
            if self.horizontal {
                self.rect.left() + self.rect.right() - p.x
            } else {
                p.x
            },
            if self.vertical {
                self.rect.top() + self.rect.bottom() - p.y
            } else {
                p.y
            },
        )
    }

    /// Desenha um bloco de texto já diagramado, espelhando se preciso.
    pub fn texto(&self, ui: &Ui, painter: &Painter, pos: Pos2, galley: Arc<Galley>, cor: Color32) {
        if !self.ativo() {
            painter.galley(pos, galley, cor);
            return;
        }
        // O egui não sabe desenhar texto invertido; então transformamos o texto
        // em triângulos (como a placa de vídeo faria) e invertemos os vértices.
        let ctx = ui.ctx();
        let tamanho_atlas = ctx.fonts(|f| f.font_image_size());
        let mut tess = Tessellator::new(
            ctx.pixels_per_point(),
            TessellationOptions::default(),
            tamanho_atlas,
            vec![],
        );
        let mut mesh = Mesh::default();
        tess.tessellate_text(&TextShape::new(pos, galley, cor), &mut mesh);
        for v in &mut mesh.vertices {
            v.pos = self.ponto(v.pos);
        }
        painter.add(Shape::mesh(mesh));
    }

    pub fn triangulo(&self, painter: &Painter, pts: [Pos2; 3], cor: Color32) {
        painter.add(Shape::convex_polygon(
            pts.iter().map(|p| self.ponto(*p)).collect(),
            cor,
            Stroke::NONE,
        ));
    }
}

fn job(
    texto: String,
    fonte: FontId,
    cor: Color32,
    entrelinha: f32,
    largura: f32,
    centro: bool,
) -> LayoutJob {
    let altura_linha = fonte.size * entrelinha;
    let mut job = LayoutJob::single_section(
        texto,
        TextFormat {
            font_id: fonte,
            color: cor,
            line_height: Some(altura_linha),
            ..Default::default()
        },
    );
    job.wrap.max_width = largura;
    if centro {
        job.halign = Align::Center;
    }
    job
}

/// Desenha o roteiro dentro de `rect`.
///
/// `escala` = 1.0 na tela do apresentador; na prévia, a razão entre a largura
/// da prévia e a largura da tela do apresentador.
#[allow(clippy::too_many_arguments)]
pub fn desenhar(
    ui: &Ui,
    rect: Rect,
    roteiro: &Roteiro,
    cfg: &Settings,
    familia: &FontFamily,
    rolagem_em: f32,
    escala: f32,
    espelho: Espelho,
) -> Medidas {
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, cor(cfg.cor_fundo));

    let px = (cfg.font_size * escala).max(4.0);
    let margem = rect.width() * cfg.margem_pct.clamp(0.0, 40.0) / 100.0;
    let largura = (rect.width() - 2.0 * margem).max(px * 3.0);
    let centro = cfg.alinhamento == Alinhamento::Centro;
    let x = if centro {
        rect.left() + margem + largura / 2.0
    } else {
        rect.left() + margem
    };
    let y_marcador = rect.top() + rect.height() * cfg.marcador_pct.clamp(0.0, 90.0) / 100.0;
    let inicio = y_marcador - rolagem_em * px;

    let fonte = FontId::new(px, familia.clone());
    let fonte_titulo = FontId::new(px * 0.55, familia.clone());
    let cor_texto = cor(cfg.cor_texto);
    let cor_titulo = cor(cfg.cor_titulo);

    let mut medidas = Medidas::default();
    let mut y = inicio;
    for seg in &roteiro.segmentos {
        medidas.topos_em.push((y - inicio) / px);

        let titulo = if seg.tipo == Tipo::Intervalo {
            let dur = if seg.duracao.trim().is_empty() {
                String::new()
            } else {
                format!(" ({})", seg.duracao.trim())
            };
            Some(format!("——— {}{dur} ———", seg.titulo))
        } else if cfg.mostrar_titulos && !seg.titulo.trim().is_empty() {
            Some(seg.titulo.trim().to_string())
        } else {
            None
        };
        if let Some(t) = titulo {
            let g = painter.layout_job(job(
                t,
                fonte_titulo.clone(),
                cor_titulo,
                1.2,
                largura,
                centro || seg.tipo == Tipo::Intervalo,
            ));
            let gx = if seg.tipo == Tipo::Intervalo && !centro {
                x + largura / 2.0
            } else {
                x
            };
            let h = g.size().y;
            if y + h >= rect.top() && y <= rect.bottom() {
                espelho.texto(ui, &painter, Pos2::new(gx, y), g, cor_titulo);
            }
            y += h + px * 0.25;
        }

        if !seg.texto.trim().is_empty() {
            let mut j = job(
                String::new(),
                fonte.clone(),
                cor_texto,
                cfg.entrelinha,
                largura,
                centro,
            );
            j.sections.clear();
            for (trecho, destaque) in trechos_destaque(&seg.texto) {
                let trecho = if cfg.caixa_alta {
                    trecho.to_uppercase()
                } else {
                    trecho
                };
                let cor_trecho = if destaque {
                    cor(cfg.cor_destaque)
                } else {
                    cor_texto
                };
                j.append(
                    &trecho,
                    0.0,
                    TextFormat {
                        font_id: fonte.clone(),
                        color: cor_trecho,
                        line_height: Some(px * cfg.entrelinha),
                        ..Default::default()
                    },
                );
            }
            let g = painter.layout_job(j);
            let h = g.size().y;
            if y + h >= rect.top() && y <= rect.bottom() {
                espelho.texto(ui, &painter, Pos2::new(x, y), g, cor_texto);
            }
            y += h;
        }
        y += px * 1.2;
    }
    medidas.total_em = ((y - inicio) / px).max(0.0);

    if cfg.mostrar_marcador {
        // Seta no meio da primeira linha que está sendo lida.
        let meio = y_marcador + px * cfg.entrelinha * 0.5;
        let tam = (px * 0.45).max(8.0);
        let l = rect.left() + 2.0;
        espelho.triangulo(
            &painter,
            [
                Pos2::new(l, meio - tam / 2.0),
                Pos2::new(l + tam * 0.9, meio),
                Pos2::new(l, meio + tam / 2.0),
            ],
            cor_titulo,
        );
        // Seta igual do lado direito (o apresentador acha a linha com o canto do olho).
        let r = rect.right() - 2.0;
        espelho.triangulo(
            &painter,
            [
                Pos2::new(r, meio - tam / 2.0),
                Pos2::new(r - tam * 0.9, meio),
                Pos2::new(r, meio + tam / 2.0),
            ],
            cor_titulo,
        );
    }
    medidas
}
