//! Confirmação de descarte e avisos flutuantes.

use super::*;

impl App {
    pub(super) fn confirmacao(&mut self, ctx: &egui::Context) {
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

    pub(super) fn mostrar_aviso(&mut self, ctx: &egui::Context) {
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
}
