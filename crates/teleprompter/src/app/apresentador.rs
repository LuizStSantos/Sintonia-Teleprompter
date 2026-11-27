//! Janela do apresentador: o operador escolhe o monitor e ela já abre em
//! tela cheia nele (ou em janela, para testar). Também a tela cheia do operador.

use super::*;

impl App {
    /// Lê os monitores conectados (a cada 2 s: monitor ligado/desligado no meio do uso).
    pub(super) fn ler_monitores(&mut self, frame: &eframe::Frame) {
        if self
            .monitores_lidos
            .is_some_and(|t| t.elapsed() < Duration::from_secs(2))
        {
            return;
        }
        self.monitores_lidos = Some(Instant::now());
        let Some(janela) = frame.winit_window() else {
            return;
        };
        let principal = janela.primary_monitor();
        let do_operador = janela.current_monitor();
        self.monitores = janela
            .available_monitors()
            .enumerate()
            .map(|(i, m)| InfoMonitor {
                nome: m
                    .name()
                    .filter(|n| !n.trim().is_empty())
                    .unwrap_or_else(|| format!("Monitor {}", i + 1)),
                largura: m.size().width,
                altura: m.size().height,
                principal: principal.as_ref() == Some(&m),
                do_operador: do_operador.as_ref() == Some(&m),
                escala: m.scale_factor() as f32,
            })
            .collect();
    }

    /// Tamanho que a tela do apresentador tem (aberta) ou vai ter (fechada):
    /// a prévia usa a mesma proporção, inclusive monitor em pé (retrato).
    pub(super) fn tamanho_alvo_apresentador(&self) -> Vec2 {
        if self.apresentador_aberto {
            return self.tamanho_apresentador;
        }
        if self.cfg.tela_cheia_apresentador
            && let Some(m) = self.monitores.get(self.cfg.monitor_apresentador)
        {
            return m.tamanho_logico();
        }
        self.tamanho_apresentador
    }

    /// Monitor mais indicado para o apresentador: o primeiro que não é o do operador.
    fn monitor_sugerido(&self) -> Option<usize> {
        if self.monitores.len() < 2 {
            return None;
        }
        let escolhido = self.cfg.monitor_apresentador;
        if self
            .monitores
            .get(escolhido)
            .is_some_and(|m| !m.do_operador)
        {
            return Some(escolhido);
        }
        self.monitores.iter().position(|m| !m.do_operador)
    }

    /// Abre (ou move) a tela do apresentador: `Some(monitor)` = tela cheia
    /// naquele monitor; `None` = em janela.
    pub(super) fn abrir_apresentador(&mut self, monitor: Option<usize>) {
        match monitor {
            Some(i) => {
                self.cfg.monitor_apresentador = i;
                self.cfg.tela_cheia_apresentador = true;
                // A prévia já assume o tamanho e a orientação do monitor escolhido.
                if let Some(m) = self.monitores.get(i) {
                    self.tamanho_apresentador = m.tamanho_logico();
                }
            }
            None => self.cfg.tela_cheia_apresentador = false,
        }
        if self.apresentador_aberto {
            // Fecha neste quadro e reabre no próximo, já no monitor novo.
            self.apresentador_aberto = false;
            self.reabrir_apresentador = true;
        } else {
            self.apresentador_aberto = true;
        }
    }

    /// Botão "Abrir/Fechar tela do apresentador": abre direto no monitor indicado.
    pub(super) fn alternar_apresentador(&mut self) {
        if self.apresentador_aberto {
            self.apresentador_aberto = false;
        } else {
            self.abrir_apresentador(self.monitor_sugerido());
        }
    }

    pub(super) fn alternar_tela_cheia_operador(&mut self, ctx: &egui::Context) {
        let cheia = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!cheia));
    }

    /// Escolha do monitor do apresentador: um botão por monitor conectado.
    pub(super) fn seletor_monitor(&mut self, ui: &mut Ui) {
        let p = self.p();
        if self.monitores.is_empty() {
            ui.label(
                RichText::new("Não foi possível listar os monitores.").color(p.texto_mais_fraco),
            );
        }
        let mut escolha: Option<Option<usize>> = None;
        for (i, m) in self.monitores.iter().enumerate() {
            let ativo = self.apresentador_aberto
                && self.cfg.tela_cheia_apresentador
                && self.cfg.monitor_apresentador == i;
            let mut detalhes = vec![format!("{} × {}", m.largura, m.altura)];
            if m.do_operador {
                detalhes.push("onde está esta janela".into());
            } else if m.principal {
                detalhes.push("principal".into());
            }
            let resp = egui::Frame::new()
                .fill(if ativo {
                    p.vermelho.gamma_multiply(0.15)
                } else {
                    p.panel_2
                })
                .stroke(egui::Stroke::new(
                    1.0,
                    if ativo { p.vermelho } else { p.borda },
                ))
                .corner_radius(8.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let (q, _) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::hover());
                        icones::desenhar(
                            ui,
                            q,
                            Icone::Monitor,
                            if ativo { p.vermelho } else { p.texto_fraco },
                        );
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.label(
                                RichText::new(format!("Monitor {} · {}", i + 1, m.nome))
                                    .font(tema::forte(13.0))
                                    .color(p.texto),
                            );
                            ui.label(
                                RichText::new(detalhes.join(" · "))
                                    .size(11.5)
                                    .color(p.texto_mais_fraco),
                            );
                        });
                    });
                })
                .response
                .interact(Sense::click());
            let dica = if ativo {
                "No ar neste monitor"
            } else {
                "Abrir a tela do apresentador em tela cheia neste monitor"
            };
            if resp.on_hover_text(dica).clicked() {
                escolha = Some(Some(i));
            }
            ui.add_space(4.0);
        }
        ui.horizontal(|ui| {
            let em_janela = self.apresentador_aberto && !self.cfg.tela_cheia_apresentador;
            if ui
                .selectable_label(em_janela, "Em janela (para testar)")
                .on_hover_text("Abre como uma janela comum, que pode ser arrastada")
                .clicked()
            {
                escolha = Some(None);
            }
            if self.apresentador_aberto
                && ui
                    .button("Fechar")
                    .on_hover_text("Fechar a tela do apresentador")
                    .clicked()
            {
                self.apresentador_aberto = false;
            }
        });
        if self.monitores.len() == 1 {
            ui.label(
                RichText::new("Só um monitor conectado: em tela cheia, a tela do apresentador cobre esta. Use Esc para sair.")
                    .size(11.5)
                    .color(p.ambar),
            );
        }
        if let Some(m) = escolha {
            self.abrir_apresentador(m);
        }
    }

    pub(super) fn janela_apresentador(&mut self, ctx: &egui::Context) {
        if self.reabrir_apresentador {
            // Fechou num quadro; reabre neste já no monitor novo.
            self.reabrir_apresentador = false;
            self.apresentador_aberto = true;
            ctx.request_repaint();
            return;
        }
        if !self.apresentador_aberto {
            return;
        }
        let mut builder = ViewportBuilder::default()
            .with_title("Sintonia Teleprompter — Apresentador")
            .with_inner_size(if self.cfg.tela_cheia_apresentador {
                self.tamanho_alvo_apresentador()
            } else {
                Vec2::new(1280.0, 720.0)
            });
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
                    if self.so_cronometro {
                        self.desenhar_so_cronometro(ui, rect, espelho);
                        return;
                    }
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
                    self.mouse_no_texto(
                        ui,
                        &resp,
                        self.cfg.font_size.max(4.0),
                        self.cfg.espelhar_vertical,
                    );
                    if self.cfg.mostrar_relogio_apresentador {
                        self.relogio_apresentador(ui, rect, espelho);
                    }
                });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_em_pe_e_com_escala() {
        let m = InfoMonitor {
            nome: "Teleprompter".into(),
            largura: 1080,
            altura: 1920,
            principal: false,
            do_operador: false,
            escala: 1.5,
        };
        let t = m.tamanho_logico();
        assert_eq!((t.x, t.y), (720.0, 1280.0), "retrato e escala de 150 %");
    }
}
