//! Janela do apresentador (segundo monitor, tela cheia, espelhamento).

use super::*;

impl App {
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
