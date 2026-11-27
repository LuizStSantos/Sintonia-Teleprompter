//! Prévia da tela do apresentador, na tela do operador.

use super::*;

impl App {
    pub(super) fn previa(&mut self, ui: &mut Ui) {
        let alvo = self.tamanho_alvo_apresentador();
        let mut disponivel = ui.available_rect_before_wrap();
        disponivel.min.y += 26.0; // espaço para o rótulo acima da prévia
        let proporcao = (alvo.x / alvo.y.max(1.0)).clamp(0.3, 4.0);
        let mut tam = Vec2::new(disponivel.width(), disponivel.width() / proporcao);
        if tam.y > disponivel.height() {
            tam = Vec2::new(disponivel.height() * proporcao, disponivel.height());
        }
        let rect = Rect::from_center_size(disponivel.center(), tam);
        // "Prévia espelhada": o operador vê exatamente o que vai para o vidro.
        let espelho_previa = if self.cfg.previa_espelhada {
            Espelho {
                rect,
                horizontal: self.cfg.espelhar_horizontal,
                vertical: self.cfg.espelhar_vertical,
            }
        } else {
            Espelho::nenhum(rect)
        };
        let resp = ui.allocate_rect(rect, Sense::click_and_drag());
        let escala = rect.width() / alvo.x.max(1.0);
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
            alvo.x
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
            espelho_previa,
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
                espelho_previa,
            );
        }
        // As medidas da tela do apresentador (escala 1) valem igual — tudo é proporcional.
        self.medidas = medidas;

        let px = (self.cfg.font_size * escala).max(4.0);
        self.mouse_no_texto(ui, &resp, px, espelho_previa.vertical);
        if self.so_cronometro {
            self.desenhar_so_cronometro(ui, rect, espelho_previa);
        }
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
        // Atalho: abrir/fechar a tela do apresentador (direto no monitor indicado).
        let (txt, fundo, cor_txt) = if self.apresentador_aberto {
            ("Fechar apresentador", p.panel_2, p.texto)
        } else {
            ("Abrir apresentador", p.vermelho, Color32::WHITE)
        };
        let area = Rect::from_min_max(
            egui::pos2(rect.right() - 420.0, rect.top() - 28.0),
            egui::pos2(rect.right(), rect.top() - 3.0),
        );
        let resp = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .max_rect(area)
                    .layout(Layout::right_to_left(Align::Center)),
                |ui| {
                    let resp = icones::botao_com(
                        ui,
                        Icone::TelaCheia,
                        Some(RichText::new(txt).size(12.5).color(cor_txt)),
                        Vec2::new(0.0, 24.0),
                        Some(fundo),
                        Some(cor_txt),
                    );
                    // Espelhamento à vista (antes só no painel de Formatação).
                    let ligado = self.cfg.espelhar_horizontal;
                    if icones::botao_com(
                        ui,
                        Icone::Espelhar,
                        Some(
                            RichText::new(if ligado { "Espelhado" } else { "Espelhar" }).size(12.5),
                        ),
                        Vec2::new(0.0, 24.0),
                        Some(if ligado {
                            p.vermelho.gamma_multiply(0.25)
                        } else {
                            p.panel_2
                        }),
                        Some(if ligado { p.vermelho } else { p.texto }),
                    )
                    .on_hover_text(
                        "Espelha o texto na tela do apresentador (para o vidro do teleprompter)",
                    )
                    .clicked()
                    {
                        self.cfg.espelhar_horizontal = !ligado;
                    }
                    ui.checkbox(
                        &mut self.cfg.previa_espelhada,
                        RichText::new("ver espelhado").size(12.0),
                    )
                    .on_hover_text(
                        "Mostra a prévia exatamente como a tela do apresentador (espelhada)",
                    );
                    resp
                },
            )
            .inner;
        if resp
            .on_hover_text("Abre a tela do apresentador em tela cheia no outro monitor (escolha o monitor no menu Apresentador)")
            .clicked()
        {
            self.alternar_apresentador();
        }
    }
}
