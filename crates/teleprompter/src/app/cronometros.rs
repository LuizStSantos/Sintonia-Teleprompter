//! Tempos: cronômetro do programa, da entrevista, leitura estimada e a tela só cronômetro.

use super::*;

impl App {
    pub(super) fn meta_seg(&self) -> Option<i64> {
        parse_duracao(&self.meta_txt).filter(|s| *s > 0)
    }

    /// Estimativa do tempo de leitura que falta, pela posição na rolagem.
    pub(super) fn leitura_restante(&self) -> i64 {
        let total_palavras = self.roteiro.palavras() as f32;
        let fracao = if self.medidas.total_em > 0.0 {
            (self.rolagem_em / self.medidas.total_em).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let restantes = total_palavras * (1.0 - fracao);
        (restantes / self.cfg.palavras_por_minuto.max(1) as f32 * 60.0).round() as i64
    }

    pub(super) fn meta_entrevista_seg(&self) -> Option<i64> {
        if !self.cfg.entrevista_com_tempo {
            return None; // "Só rodando": conta para cima, sem limite.
        }
        parse_duracao(&self.meta_entrevista_txt).filter(|s| *s > 0)
    }

    /// Liga/desliga duas vezes por segundo (tela piscando quando o tempo estoura).
    pub(super) fn piscando(ui: &Ui) -> bool {
        ui.ctx().request_repaint_after(Duration::from_millis(100));
        (ui.input(|i| i.time) * 2.0) as i64 % 2 == 0
    }

    pub(super) fn relogio_apresentador(&self, ui: &Ui, rect: Rect, espelho: Espelho) {
        let decorrido = self.crono.decorrido().as_secs() as i64;
        let (txt, c) = match self.meta_seg() {
            Some(meta) => {
                let resta = meta - decorrido;
                let p = tema::ESCURO;
                let c = if resta < 0 {
                    p.vermelho
                } else if resta <= 60 {
                    p.ambar
                } else {
                    p.verde
                };
                (fmt_tempo(resta), c)
            }
            None => (fmt_tempo(decorrido), Color32::from_gray(220)),
        };
        let tamanho = (rect.height() * 0.06).clamp(18.0, 64.0);
        let painter = ui.painter_at(rect);
        let galley = painter.layout_no_wrap(txt, FontId::monospace(tamanho), c);
        let pos = rect.right_top() + Vec2::new(-galley.size().x - tamanho * 0.5, tamanho * 0.3);
        let fundo = Rect::from_min_size(pos, galley.size()).expand(tamanho * 0.2);
        let a = espelho.ponto(fundo.min);
        let b = espelho.ponto(fundo.max);
        // Estourou: a caixa do relógio pisca em vermelho.
        let estourou = self.meta_seg().is_some_and(|m| m - decorrido < 0);
        let (fundo_cor, c) = if estourou && Self::piscando(ui) {
            (tema::ESCURO.vermelho, Color32::WHITE)
        } else {
            (Color32::from_black_alpha(170), c)
        };
        painter.rect_filled(Rect::from_two_pos(a, b), 6.0, fundo_cor);
        let galley =
            painter.layout_no_wrap(galley.text().to_string(), FontId::monospace(tamanho), c);
        espelho.texto(ui, &painter, pos, galley, c);
    }

    /// Tela "só cronômetro": o tempo grande no meio da tela, sem o texto —
    /// para o apresentador acompanhar o tempo de uma entrevista.
    pub(super) fn desenhar_so_cronometro(&self, ui: &Ui, rect: Rect, espelho: Espelho) {
        let painter = ui.painter_at(rect);
        let p = tema::ESCURO;
        let decorrido = self.crono_entrevista.decorrido().as_secs() as i64;
        // Estourou o tempo: a tela inteira pisca em vermelho.
        let estourou = self
            .meta_entrevista_seg()
            .is_some_and(|m| m - decorrido < 0);
        let aceso = estourou && Self::piscando(ui);
        painter.rect_filled(
            rect,
            0.0,
            if aceso {
                p.vermelho
            } else {
                cor(self.cfg.cor_fundo)
            },
        );
        let (grande, cor_grande, detalhe) = match self.meta_entrevista_seg() {
            Some(meta) => {
                let resta = meta - decorrido;
                let c = if resta < 0 {
                    p.vermelho
                } else if resta <= 60 {
                    p.ambar
                } else {
                    p.verde
                };
                let det = if resta < 0 {
                    format!("ESTOUROU · TOTAL {}", fmt_tempo(decorrido))
                } else {
                    format!("FALTAM · DE {}", fmt_tempo(meta))
                };
                let grande = if resta < 0 {
                    format!("+{}", fmt_tempo(-resta))
                } else {
                    fmt_tempo(resta)
                };
                (grande, c, det)
            }
            None => (
                fmt_tempo(decorrido),
                cor(self.cfg.cor_texto),
                "TEMPO DE ENTREVISTA".to_string(),
            ),
        };
        let h = rect.height();
        let g_grande = painter.layout_no_wrap(
            grande,
            FontId::monospace((h * 0.34).min(rect.width() * 0.2)),
            cor_grande,
        );
        let g_det = painter.layout_no_wrap(
            detalhe,
            FontId::proportional((h * 0.05).max(10.0)),
            cor(self.cfg.cor_titulo),
        );
        let total = g_grande.size().y + g_det.size().y;
        let topo = rect.center().y - total / 2.0;
        let pos_det = egui::pos2(rect.center().x - g_det.size().x / 2.0, topo);
        let pos_grande = egui::pos2(
            rect.center().x - g_grande.size().x / 2.0,
            topo + g_det.size().y,
        );
        let (cd, cg) = if aceso {
            (Color32::WHITE, Color32::WHITE)
        } else {
            (cor(self.cfg.cor_titulo), cor_grande)
        };
        let g_grande = painter.layout_no_wrap(
            g_grande.text().to_string(),
            g_grande.job.sections[0].format.font_id.clone(),
            cg,
        );
        let g_det = painter.layout_no_wrap(
            g_det.text().to_string(),
            g_det.job.sections[0].format.font_id.clone(),
            cd,
        );
        espelho.texto(ui, &painter, pos_det, g_det, cd);
        espelho.texto(ui, &painter, pos_grande, g_grande, cg);
        if self.crono_entrevista.rodando() {
            ui.ctx().request_repaint_after(Duration::from_millis(250));
        }
    }
}
