//! Prévia da tela do apresentador, na tela do operador.

use super::*;

impl App {
    pub(super) fn previa(&mut self, ui: &mut Ui) {
        let mut disponivel = ui.available_rect_before_wrap();
        disponivel.min.y += 18.0; // espaço para o rótulo acima da prévia
        let proporcao =
            (self.tamanho_apresentador.x / self.tamanho_apresentador.y.max(1.0)).clamp(0.3, 4.0);
        let mut tam = Vec2::new(disponivel.width(), disponivel.width() / proporcao);
        if tam.y > disponivel.height() {
            tam = Vec2::new(disponivel.height() * proporcao, disponivel.height());
        }
        let rect = Rect::from_center_size(disponivel.center(), tam);
        let resp = ui.allocate_rect(rect, Sense::click_and_drag());
        let escala = rect.width() / self.tamanho_apresentador.x.max(1.0);
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
            self.tamanho_apresentador.x
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
            Espelho::nenhum(rect),
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
                Espelho::nenhum(rect),
            );
        }
        // As medidas da tela do apresentador (escala 1) valem igual — tudo é proporcional.
        self.medidas = medidas;

        let px = (self.cfg.font_size * escala).max(4.0);
        self.mouse_no_texto(ui, &resp, px, false);
        if self.so_cronometro {
            self.desenhar_so_cronometro(ui, rect, Espelho::nenhum(rect));
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
    }
}
