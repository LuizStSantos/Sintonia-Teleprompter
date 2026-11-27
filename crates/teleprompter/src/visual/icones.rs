//! Ícones desenhados (vetoriais), no lugar dos símbolos de emoji do sistema,
//! que mudam de cara de um computador para outro. Traço arredondado, no
//! estilo dos ícones do site do Sintonia.

use egui::{Color32, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, pos2, vec2};
use std::f32::consts::PI;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icone {
    Play,
    Pausa,
    Inicio,
    Voltar,
    Avancar,
    Zerar,
    Cronometro,
    SetaEsquerda,
    SetaDireita,
    SetaCima,
    SetaBaixo,
    TelaCheia,
    Monitor,
    Fechar,
    Mais,
    Lixeira,
    Usuario,
}

/// Desenha o ícone ocupando o quadrado `r`.
pub fn desenhar(ui: &Ui, r: Rect, icone: Icone, cor: Color32) {
    let p = ui.painter();
    let c = r.center();
    let s = r.width().min(r.height());
    let traco = Stroke::new((s * 0.11).max(1.5), cor);
    let pt = |x: f32, y: f32| pos2(c.x + x * s, c.y + y * s);
    let triangulo = |pts: [Pos2; 3]| Shape::convex_polygon(pts.to_vec(), cor, Stroke::NONE);
    match icone {
        Icone::Play => {
            p.add(triangulo([
                pt(-0.28, -0.36),
                pt(0.38, 0.0),
                pt(-0.28, 0.36),
            ]));
        }
        Icone::Pausa => {
            let barra = |x: f32| Rect::from_center_size(pt(x, 0.0), vec2(s * 0.2, s * 0.7));
            p.rect_filled(barra(-0.18), s * 0.05, cor);
            p.rect_filled(barra(0.18), s * 0.05, cor);
        }
        Icone::Inicio => {
            p.rect_filled(
                Rect::from_center_size(pt(-0.3, 0.0), vec2(s * 0.12, s * 0.62)),
                s * 0.03,
                cor,
            );
            p.add(triangulo([pt(0.34, -0.31), pt(-0.2, 0.0), pt(0.34, 0.31)]));
        }
        Icone::Voltar => {
            p.add(triangulo([pt(0.04, -0.3), pt(-0.4, 0.0), pt(0.04, 0.3)]));
            p.add(triangulo([pt(0.44, -0.3), pt(0.0, 0.0), pt(0.44, 0.3)]));
        }
        Icone::Avancar => {
            p.add(triangulo([pt(-0.04, -0.3), pt(0.4, 0.0), pt(-0.04, 0.3)]));
            p.add(triangulo([pt(-0.44, -0.3), pt(0.0, 0.0), pt(-0.44, 0.3)]));
        }
        Icone::Zerar => {
            // Arco de 300° com a ponta da seta no fim.
            let raio = s * 0.32;
            let n = 24;
            let (ini, fim) = (-PI * 0.35, PI * 1.35);
            let pontos: Vec<Pos2> = (0..=n)
                .map(|i| {
                    let a = ini + (fim - ini) * i as f32 / n as f32;
                    pos2(c.x + raio * a.cos(), c.y + raio * a.sin())
                })
                .collect();
            let ponta = *pontos.first().unwrap();
            p.add(Shape::line(pontos, traco));
            let a = ini;
            let tangente = vec2(-a.sin(), a.cos()) * -1.0;
            let normal = vec2(a.cos(), a.sin());
            let t = s * 0.2;
            p.add(triangulo([
                ponta + tangente * t * 0.9,
                ponta + normal * t * 0.6,
                ponta - normal * t * 0.6,
            ]));
        }
        Icone::Cronometro => {
            let centro = pt(0.0, 0.07);
            p.circle_stroke(centro, s * 0.33, traco);
            p.line_segment([pt(-0.1, -0.42), pt(0.1, -0.42)], traco);
            p.line_segment([pt(0.0, -0.42), pt(0.0, -0.27)], traco);
            p.line_segment([centro, pt(0.12, -0.07)], traco);
        }
        Icone::SetaEsquerda => {
            p.add(Shape::line(
                vec![pt(0.12, -0.28), pt(-0.16, 0.0), pt(0.12, 0.28)],
                traco,
            ));
        }
        Icone::SetaDireita => {
            p.add(Shape::line(
                vec![pt(-0.12, -0.28), pt(0.16, 0.0), pt(-0.12, 0.28)],
                traco,
            ));
        }
        Icone::SetaCima => {
            p.add(Shape::line(
                vec![pt(-0.28, 0.12), pt(0.0, -0.16), pt(0.28, 0.12)],
                traco,
            ));
        }
        Icone::SetaBaixo => {
            p.add(Shape::line(
                vec![pt(-0.28, -0.12), pt(0.0, 0.16), pt(0.28, -0.12)],
                traco,
            ));
        }
        Icone::TelaCheia => {
            let (a, b) = (0.36, 0.16);
            for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                p.add(Shape::line(
                    vec![pt(sx * a, sy * b), pt(sx * a, sy * a), pt(sx * b, sy * a)],
                    traco,
                ));
            }
        }
        Icone::Monitor => {
            let tela = Rect::from_center_size(pt(0.0, -0.08), vec2(s * 0.8, s * 0.52));
            p.rect_stroke(tela, s * 0.06, traco, egui::StrokeKind::Middle);
            p.line_segment([pt(0.0, 0.18), pt(0.0, 0.34)], traco);
            p.line_segment([pt(-0.2, 0.36), pt(0.2, 0.36)], traco);
        }
        Icone::Fechar => {
            p.line_segment([pt(-0.26, -0.26), pt(0.26, 0.26)], traco);
            p.line_segment([pt(0.26, -0.26), pt(-0.26, 0.26)], traco);
        }
        Icone::Mais => {
            p.line_segment([pt(-0.3, 0.0), pt(0.3, 0.0)], traco);
            p.line_segment([pt(0.0, -0.3), pt(0.0, 0.3)], traco);
        }
        Icone::Lixeira => {
            p.line_segment([pt(-0.32, -0.26), pt(0.32, -0.26)], traco);
            p.line_segment([pt(-0.1, -0.36), pt(0.1, -0.36)], traco);
            p.add(Shape::line(
                vec![
                    pt(-0.24, -0.18),
                    pt(-0.18, 0.36),
                    pt(0.18, 0.36),
                    pt(0.24, -0.18),
                ],
                traco,
            ));
        }
        Icone::Usuario => {
            p.circle_stroke(pt(0.0, -0.16), s * 0.17, traco);
            p.add(Shape::line(
                (0..=16)
                    .map(|i| {
                        let a = PI + PI * i as f32 / 16.0;
                        pos2(c.x + s * 0.32 * a.cos(), c.y + s * 0.38 + s * 0.3 * a.sin())
                    })
                    .collect(),
                traco,
            ));
        }
    }
}

/// Botão só com ícone, no mesmo estilo dos botões do egui/Sintonia.
pub fn botao(ui: &mut Ui, icone: Icone, tamanho: f32) -> Response {
    botao_com(ui, icone, None, Vec2::splat(tamanho), None, None)
}

/// Botão com ícone e (opcionalmente) texto, cor de fundo e cor do conteúdo.
pub fn botao_com(
    ui: &mut Ui,
    icone: Icone,
    texto: Option<egui::RichText>,
    tamanho_min: Vec2,
    fundo: Option<Color32>,
    cor: Option<Color32>,
) -> Response {
    let lado_icone = (tamanho_min.y * 0.46).clamp(10.0, 26.0);
    let galley = texto.map(|t| {
        egui::WidgetText::from(t).into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Button,
        )
    });
    let largura_texto = galley.as_ref().map_or(0.0, |g| g.size().x + 8.0);
    let largura = tamanho_min.x.max(lado_icone + largura_texto + 16.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(largura, tamanho_min.y), Sense::click());
    if ui.is_rect_visible(rect) {
        let v = ui.style().interact(&resp);
        let preenchimento = match fundo {
            Some(f) if resp.hovered() => f.gamma_multiply(0.85),
            Some(f) => f,
            None => v.weak_bg_fill,
        };
        ui.painter().rect(
            rect,
            v.corner_radius,
            preenchimento,
            v.bg_stroke,
            egui::StrokeKind::Inside,
        );
        let cor = cor.unwrap_or(v.fg_stroke.color);
        let conteudo = lado_icone + largura_texto;
        let x0 = rect.center().x - conteudo / 2.0;
        let r_icone = Rect::from_center_size(
            pos2(x0 + lado_icone / 2.0, rect.center().y),
            Vec2::splat(lado_icone),
        );
        desenhar(ui, r_icone, icone, cor);
        if let Some(g) = galley {
            let pos = pos2(r_icone.right() + 8.0, rect.center().y - g.size().y / 2.0);
            ui.painter().galley(pos, g, cor);
        }
    }
    resp
}
