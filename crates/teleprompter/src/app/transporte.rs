//! Barra de baixo: rolagem, velocidade, cronômetros e barra de progresso.

use super::*;

impl App {
    /// Bloco da barra de baixo com título pequeno, no estilo dos cartões do Sintonia.
    /// `denso`: telas baixas (ex.: 1366×768) — sem título e com menos margem.
    pub(super) fn cartao<R>(
        ui: &mut Ui,
        p: Paleta,
        titulo: &str,
        denso: bool,
        conteudo: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        egui::Frame::new()
            .fill(p.bg)
            .stroke(egui::Stroke::new(1.0, p.borda))
            .corner_radius(8.0)
            .inner_margin(if denso {
                egui::Margin::symmetric(8, 4)
            } else {
                egui::Margin::symmetric(10, 6)
            })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    if !denso {
                        ui.label(
                            RichText::new(titulo.to_uppercase())
                                .size(10.0)
                                .color(p.texto_mais_fraco),
                        );
                    }
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

    /// Tela baixa (ex.: 1366×768): barra de baixo mais enxuta.
    pub(super) fn tela_baixa(ui: &Ui) -> bool {
        ui.ctx().content_rect().height() < 860.0
    }

    /// Botões início, voltar, Rolar/Pausar e avançar (barra cheia e recolhida).
    fn botoes_rolagem(&mut self, ui: &mut Ui, altura: f32) {
        let p = self.p();
        if icones::botao(ui, Icone::Inicio, altura)
            .on_hover_text("Começo — zera o cronômetro (Home)")
            .clicked()
        {
            self.ir_inicio();
        }
        let r = icones::botao(ui, Icone::Voltar, altura)
            .on_hover_text("Clique: bloco anterior (Page Up)\nSegure: volta rápido");
        if self.botao_segurar(ui, &r, -1.0) {
            self.bloco_relativo(-1);
        }
        let (icone, txt, cor_btn) = if self.rodando {
            (Icone::Pausa, "Pausar", p.ambar)
        } else {
            (Icone::Play, "Rolar", p.vermelho)
        };
        if icones::botao_com(
            ui,
            icone,
            Some(
                RichText::new(txt)
                    .font(tema::forte(16.0))
                    .color(Color32::WHITE),
            ),
            Vec2::new(124.0, altura),
            Some(cor_btn),
            Some(Color32::WHITE),
        )
        .on_hover_text("Espaço ou clique no texto (End para)")
        .clicked()
        {
            self.alternar_rolagem();
        }
        let r = icones::botao(ui, Icone::Avancar, altura)
            .on_hover_text("Clique: próximo bloco (Page Down)\nSegure: passa rápido");
        if self.botao_segurar(ui, &r, 1.0) {
            self.bloco_relativo(1);
        }
    }

    pub(super) fn painel_transporte(&mut self, ui: &mut Ui) {
        let p = self.p();
        let denso = Self::tela_baixa(ui);
        let h = if denso { 28.0 } else { 34.0 };
        let tam_mono = if denso { 19.0 } else { 22.0 };
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

        // Setinha para recolher/mostrar + barra de progresso.
        ui.horizontal(|ui| {
            let (icone, dica) = if self.cfg.barra_recolhida {
                (Icone::SetaCima, "Mostrar todos os controles")
            } else {
                (
                    Icone::SetaBaixo,
                    "Recolher a barra (mais espaço para a prévia)",
                )
            };
            if icones::botao(ui, icone, 22.0).on_hover_text(dica).clicked() {
                self.cfg.barra_recolhida = !self.cfg.barra_recolhida;
            }
            self.barra_progresso(ui);
        });
        ui.add_space(if denso { 3.0 } else { 6.0 });

        if self.cfg.barra_recolhida {
            // Barra recolhida: uma linha só com o essencial.
            ui.horizontal(|ui| {
                self.botoes_rolagem(ui, 28.0);
                ui.separator();
                ui.spacing_mut().slider_width = 110.0;
                ui.add(
                    egui::Slider::new(&mut self.cfg.velocidade_pct, 0.0..=100.0)
                        .step_by(1.0)
                        .fixed_decimals(0)
                        .suffix(" %"),
                );
                ui.separator();
                let decorrido = self.crono.decorrido().as_secs() as i64;
                ui.label(rotulo("Programa"));
                match self.meta_seg() {
                    Some(meta) => ui.label(mono(
                        fmt_tempo(meta - decorrido),
                        18.0,
                        cor_resta(meta - decorrido),
                    )),
                    None => ui.label(mono(fmt_tempo(decorrido), 18.0, p.texto)),
                };
                ui.separator();
                ui.label(rotulo("Entrevista"));
                let d = self.crono_entrevista.decorrido().as_secs() as i64;
                match self.meta_entrevista_seg() {
                    Some(meta) => ui.label(mono(fmt_tempo(meta - d), 18.0, cor_resta(meta - d))),
                    None => ui.label(mono(fmt_tempo(d), 18.0, p.texto)),
                };
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(mono(
                        chrono::Local::now().format("%H:%M:%S").to_string(),
                        18.0,
                        p.texto,
                    ));
                });
            });
            ui.add_space(2.0);
            return;
        }

        // Linha 1: rolagem, velocidade e relógio.
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
            Self::cartao(ui, p, "Rolagem", denso, |ui| self.botoes_rolagem(ui, h));

            Self::cartao(ui, p, "Velocidade", denso, |ui| {
                ui.spacing_mut().slider_width = if denso { 120.0 } else { 150.0 };
                ui.add(
                    egui::Slider::new(&mut self.cfg.velocidade_pct, 0.0..=100.0)
                        .step_by(1.0)
                        .fixed_decimals(0)
                        .suffix(" %"),
                )
                .on_hover_text(
                    "→ / ← ou rodinha do mouse: 5 % por vez (com Shift, 1 %).\nPara passar rápido, segure o botão de avançar (ou voltar),\no botão direito do mouse sobre o texto, ou Shift + ↓ / ↑.",
                );
                for (nome, v) in VELOCIDADES_PRONTAS {
                    let ativo = (self.cfg.velocidade_pct - v).abs() < 0.5;
                    if ui.selectable_label(ativo, nome).on_hover_text(format!("{v:.0} %")).clicked() {
                        self.cfg.velocidade_pct = v;
                    }
                }
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                Self::cartao(ui, p, "Agora · leitura", denso, |ui| {
                    ui.label(mono(chrono::Local::now().format("%H:%M:%S").to_string(), tam_mono, p.texto));
                    ui.label(mono(fmt_tempo(self.leitura_restante()), 15.0, p.azul)).on_hover_text(format!(
                        "Leitura que falta (estimada): {} palavras a {} palavras/min",
                        self.roteiro.palavras(),
                        self.cfg.palavras_por_minuto
                    ));
                });
            });
        });
        ui.add_space(if denso { 4.0 } else { 6.0 });

        // Linha 2: cronômetros.
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
            Self::cartao(ui, p, "Programa", denso, |ui| {
                let decorrido = self.crono.decorrido().as_secs() as i64;
                ui.label(mono(fmt_tempo(decorrido), tam_mono, p.texto))
                    .on_hover_text("Cronômetro do programa");
                let (icone, dica) = if self.crono.rodando() {
                    (Icone::Pausa, "Pausar (F5)")
                } else {
                    (Icone::Play, "Iniciar (F5)")
                };
                if icones::botao(ui, icone, h - 4.0)
                    .on_hover_text(dica)
                    .clicked()
                {
                    self.crono.alternar();
                }
                if icones::botao(ui, Icone::Zerar, h - 4.0)
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
                    ui.label(mono(fmt_tempo(resta), tam_mono, cor_resta(resta)));
                }
                ui.checkbox(
                    &mut self.cfg.mostrar_relogio_apresentador,
                    "No apresentador",
                )
                .on_hover_text(
                    "Exibe (ou oculta) este cronômetro no canto da tela do apresentador",
                );
            });

            Self::cartao(ui, p, "Entrevista", denso, |ui| {
                let (txt, fundo, cor_txt) = if self.so_cronometro {
                    ("Voltar ao texto", p.vermelho, Color32::WHITE)
                } else {
                    ("Só cronômetro", p.panel, p.texto)
                };
                if icones::botao_com(
                    ui,
                    Icone::Cronometro,
                    Some(RichText::new(txt).font(tema::forte(13.5)).color(cor_txt)),
                    Vec2::new(0.0, h - 4.0),
                    Some(fundo),
                    Some(cor_txt),
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
                        tam_mono,
                        cor_resta(meta - decorrido),
                    )),
                    None => ui.label(mono(fmt_tempo(decorrido), tam_mono, p.texto)),
                };
                let (icone, dica) = if self.crono_entrevista.rodando() {
                    (Icone::Pausa, "Pausar (F6)")
                } else {
                    (Icone::Play, "Iniciar (F6)")
                };
                if icones::botao(ui, icone, h - 4.0)
                    .on_hover_text(dica)
                    .clicked()
                {
                    self.crono_entrevista.alternar();
                }
                if icones::botao(ui, Icone::Zerar, h - 4.0)
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
