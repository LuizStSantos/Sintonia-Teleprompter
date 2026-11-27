//! Barra de baixo: rolagem, velocidade, cronômetros e barra de progresso.

use super::*;

impl App {
    /// Bloco da barra de baixo com título pequeno, no estilo dos cartões do Sintonia.
    pub(super) fn cartao<R>(
        ui: &mut Ui,
        p: Paleta,
        titulo: &str,
        conteudo: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        egui::Frame::new()
            .fill(p.bg)
            .stroke(egui::Stroke::new(1.0, p.borda))
            .corner_radius(8.0)
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(titulo.to_uppercase())
                            .size(10.0)
                            .color(p.texto_mais_fraco),
                    );
                    ui.horizontal(|ui| conteudo(ui)).inner
                })
                .inner
            })
            .inner
    }

    /// Barra de progresso do roteiro: marca o começo de cada bloco; clicar
    /// ou arrastar nela leva o texto para aquele ponto.
    pub(super) fn barra_progresso(&mut self, ui: &mut Ui) {
        let p = self.p();
        let (rect, resp) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), 14.0),
            Sense::click_and_drag(),
        );
        let trilho = Rect::from_center_size(rect.center(), Vec2::new(rect.width(), 6.0));
        let total = self.medidas.total_em.max(0.001);
        let fracao = (self.rolagem_em / total).clamp(0.0, 1.0);
        let painter = ui.painter();
        painter.rect_filled(trilho, 3.0, p.panel_2);
        let mut feito = trilho;
        feito.max.x = trilho.left() + trilho.width() * fracao;
        painter.rect_filled(feito, 3.0, p.vermelho);
        for (i, topo) in self.medidas.topos_em.iter().enumerate() {
            let x = trilho.left() + trilho.width() * (topo / total).clamp(0.0, 1.0);
            let intervalo = self
                .roteiro
                .segmentos
                .get(i)
                .is_some_and(|s| s.tipo == Tipo::Intervalo);
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(
                    if intervalo { 2.0 } else { 1.0 },
                    if intervalo {
                        p.ambar
                    } else {
                        p.texto_mais_fraco
                    },
                ),
            );
        }
        painter.circle_filled(
            egui::pos2(feito.right(), trilho.center().y),
            6.0,
            p.vermelho,
        );
        if let Some(pos) = resp.interact_pointer_pos()
            && (resp.clicked() || resp.dragged())
        {
            let f = ((pos.x - trilho.left()) / trilho.width()).clamp(0.0, 1.0);
            self.rolagem_em = f * total;
        }
        if let Some(pos) = resp.hover_pos() {
            let f = ((pos.x - trilho.left()) / trilho.width()).clamp(0.0, 1.0);
            if let Some(seg) = self
                .medidas
                .bloco_atual(f * total)
                .and_then(|i| self.roteiro.segmentos.get(i))
            {
                resp.on_hover_text(format!(
                    "{} — clique para ir",
                    if seg.titulo.is_empty() {
                        "(sem título)"
                    } else {
                        &seg.titulo
                    }
                ));
            }
        }
    }

    pub(super) fn painel_transporte(&mut self, ui: &mut Ui) {
        let p = self.p();
        let mono = |t: String, tam: f32, c: Color32| {
            RichText::new(t).font(FontId::monospace(tam)).color(c)
        };
        let rotulo = |t: &str| {
            RichText::new(t.to_uppercase())
                .size(10.0)
                .color(p.texto_mais_fraco)
        };
        let cor_resta = |resta: i64| {
            if resta < 0 {
                p.vermelho
            } else if resta <= 60 {
                p.ambar
            } else {
                p.verde
            }
        };

        self.barra_progresso(ui);
        ui.add_space(6.0);
        // Linha 1: rolagem, velocidade e relógio.
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);

            // ROLAGEM
            Self::cartao(ui, p, "Rolagem", |ui| {
                let grande = |t: &str| RichText::new(t).size(17.0);
                if ui.button(grande("⏮")).on_hover_text("Começo — zera o cronômetro (Home)").clicked() {
                    self.ir_inicio();
                }
                let r = ui.button(grande("⏪")).on_hover_text("Clique: bloco anterior (Page Up)\nSegure: volta rápido");
                if self.botao_segurar(ui, &r, -1.0) {
                    self.bloco_relativo(-1);
                }
                let (txt, cor_btn) = if self.rodando { ("⏸  Pausar", p.ambar) } else { ("▶  Rolar", p.vermelho) };
                if ui
                    .add(
                        egui::Button::new(RichText::new(txt).font(tema::forte(17.0)).color(Color32::WHITE))
                            .fill(cor_btn)
                            .min_size(Vec2::new(128.0, 34.0)),
                    )
                    .on_hover_text("Espaço ou clique no texto (End para)")
                    .clicked()
                {
                    self.alternar_rolagem();
                }
                let r = ui.button(grande("⏩")).on_hover_text("Clique: próximo bloco (Page Down)\nSegure: passa rápido");
                if self.botao_segurar(ui, &r, 1.0) {
                    self.bloco_relativo(1);
                }
            });

            // VELOCIDADE
            Self::cartao(ui, p, "Velocidade", |ui| {
                ui.spacing_mut().slider_width = 150.0;
                ui.add(
                    egui::Slider::new(&mut self.cfg.velocidade_pct, 0.0..=100.0)
                        .step_by(1.0)
                        .fixed_decimals(0)
                        .suffix(" %"),
                )
                .on_hover_text("→ / ← ou rodinha do mouse: 5 % por vez (com Shift, 1 %)");
                for (nome, v) in VELOCIDADES_PRONTAS {
                    let ativo = (self.cfg.velocidade_pct - v).abs() < 0.5;
                    if ui.selectable_label(ativo, nome).on_hover_text(format!("{v:.0} %")).clicked() {
                        self.cfg.velocidade_pct = v;
                    }
                }
                ui.label(RichText::new("ⓘ").color(p.texto_mais_fraco)).on_hover_text(
                    "Para passar rápido, segure ⏩ (ou ⏪ para voltar),\no botão direito do mouse sobre o texto, ou Shift + ↓ / ↑.",
                );
            });

            // AGORA (à direita)
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                Self::cartao(ui, p, "Agora · leitura", |ui| {
                    ui.label(mono(chrono::Local::now().format("%H:%M:%S").to_string(), 22.0, p.texto));
                    ui.label(mono(fmt_tempo(self.leitura_restante()), 16.0, p.azul)).on_hover_text(format!(
                        "Leitura que falta (estimada): {} palavras a {} palavras/min",
                        self.roteiro.palavras(),
                        self.cfg.palavras_por_minuto
                    ));
                });
            });
        });
        ui.add_space(6.0);
        // Linha 2: cronômetros.
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
            // PROGRAMA
            Self::cartao(ui, p, "Programa", |ui| {
                let decorrido = self.crono.decorrido().as_secs() as i64;
                ui.label(mono(fmt_tempo(decorrido), 22.0, p.texto))
                    .on_hover_text("Cronômetro do programa");
                let (t, dica) = if self.crono.rodando() {
                    ("⏸", "Pausar (F5)")
                } else {
                    ("▶", "Iniciar (F5)")
                };
                if ui
                    .button(RichText::new(t).size(15.0))
                    .on_hover_text(dica)
                    .clicked()
                {
                    self.crono.alternar();
                }
                if ui
                    .button(RichText::new("↺").size(15.0))
                    .on_hover_text("Zerar")
                    .clicked()
                {
                    self.crono.zerar();
                }
                ui.label(rotulo("Tempo"));
                ui.add(
                    egui::TextEdit::singleline(&mut self.meta_txt)
                        .hint_text("mm:ss")
                        .desired_width(62.0),
                )
                .on_hover_text("Duração do programa (vazio = cronômetro só rodando)");
                if let Some(meta) = self.meta_seg() {
                    let resta = meta - decorrido;
                    ui.label(rotulo(if resta < 0 { "Estourou" } else { "Falta" }));
                    ui.label(mono(fmt_tempo(resta), 22.0, cor_resta(resta)));
                }
                ui.checkbox(
                    &mut self.cfg.mostrar_relogio_apresentador,
                    "No apresentador",
                )
                .on_hover_text(
                    "Exibe (ou oculta) este cronômetro no canto da tela do apresentador",
                );
            });

            // ENTREVISTA
            Self::cartao(ui, p, "Entrevista", |ui| {
                let (txt, fundo, cor_txt) = if self.so_cronometro {
                    ("⏱  Voltar ao texto", p.vermelho, Color32::WHITE)
                } else {
                    ("⏱  Só cronômetro", p.panel, p.texto)
                };
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(txt).font(tema::forte(13.5)).color(cor_txt),
                        )
                        .fill(fundo),
                    )
                    .on_hover_text("Troca o texto do apresentador por um cronômetro grande (F7)")
                    .clicked()
                {
                    self.alternar_so_cronometro();
                }
                let decorrido = self.crono_entrevista.decorrido().as_secs() as i64;
                match self.meta_entrevista_seg() {
                    Some(meta) => ui.label(mono(
                        fmt_tempo(meta - decorrido),
                        22.0,
                        cor_resta(meta - decorrido),
                    )),
                    None => ui.label(mono(fmt_tempo(decorrido), 22.0, p.texto)),
                };
                let (t, dica) = if self.crono_entrevista.rodando() {
                    ("⏸", "Pausar (F6)")
                } else {
                    ("▶", "Iniciar (F6)")
                };
                if ui
                    .button(RichText::new(t).size(15.0))
                    .on_hover_text(dica)
                    .clicked()
                {
                    self.crono_entrevista.alternar();
                }
                if ui
                    .button(RichText::new("↺").size(15.0))
                    .on_hover_text("Zerar")
                    .clicked()
                {
                    self.crono_entrevista.zerar();
                }
                ui.selectable_value(&mut self.cfg.entrevista_com_tempo, false, "Só rodando")
                    .on_hover_text("Conta para cima, sem limite");
                ui.selectable_value(&mut self.cfg.entrevista_com_tempo, true, "Com tempo")
                    .on_hover_text("Regressivo a partir da duração; pisca quando estoura");
                if self.cfg.entrevista_com_tempo {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.meta_entrevista_txt)
                            .hint_text("mm:ss")
                            .desired_width(62.0),
                    );
                }
            });
        });
        ui.add_space(2.0);
    }
}
