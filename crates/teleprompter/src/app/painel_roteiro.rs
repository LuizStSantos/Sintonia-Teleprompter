//! Painel da esquerda: lista de blocos e edição do texto.

use super::*;

impl App {
    pub(super) fn painel_roteiro(&mut self, ui: &mut Ui) {
        let p = self.p();
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.heading("Roteiro");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .button("➕ Bloco")
                    .on_hover_text("Adiciona um bloco de texto depois do selecionado")
                    .clicked()
                {
                    let pos = self
                        .selecionado
                        .map(|i| i + 1)
                        .unwrap_or(self.roteiro.segmentos.len());
                    self.roteiro
                        .segmentos
                        .insert(pos, Segmento::novo_texto("NOVO BLOCO", ""));
                    self.selecionado = Some(pos);
                    self.editado = true;
                }
            });
        });
        match &self.roteiro.origem {
            Origem::Espelho { titulo, .. } => ui.label(
                RichText::new(format!("Espelho: {titulo}"))
                    .small()
                    .color(p.texto_mais_fraco),
            ),
            Origem::Arquivo(p) => ui.label(
                RichText::new(format!("Arquivo: {}", p.display()))
                    .small()
                    .weak(),
            ),
            Origem::Nenhuma => ui.label(RichText::new("Roteiro digitado").small().weak()),
        };
        ui.separator();

        let atual = self.medidas.bloco_atual(self.rolagem_em);
        let altura_lista = (ui.available_height() * 0.45).max(120.0);
        let mut ir_para = None;
        ScrollArea::vertical()
            .id_salt("lista")
            .max_height(altura_lista)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for (i, seg) in self.roteiro.segmentos.iter().enumerate() {
                    let icone = match seg.tipo {
                        Tipo::Materia => "📰",
                        Tipo::Entrevistado => "🎤",
                        Tipo::Manual => "📝",
                        Tipo::Intervalo => "📺",
                        Tipo::Texto => "📄",
                    };
                    let nome = if seg.titulo.trim().is_empty() {
                        "(sem título)"
                    } else {
                        seg.titulo.trim()
                    };
                    // Linha no estilo do menu lateral do Sintonia (.nav-item / .nav-item.active).
                    let (rect, r) = ui
                        .allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::click());
                    let selecionado = self.selecionado == Some(i);
                    let no_ar = atual == Some(i);
                    if selecionado || r.hovered() {
                        ui.painter().rect_filled(rect, 7.0, p.panel_2);
                    }
                    if selecionado {
                        ui.painter().rect_filled(
                            Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height())),
                            1.0,
                            p.vermelho,
                        );
                    }
                    let cor_txt = if no_ar {
                        p.ambar
                    } else if selecionado || r.hovered() {
                        p.texto
                    } else {
                        p.texto_fraco
                    };
                    let fonte = if no_ar || selecionado {
                        tema::forte(13.5)
                    } else {
                        FontId::proportional(13.5)
                    };
                    let painter = ui.painter_at(rect);
                    painter.text(
                        rect.left_center() + Vec2::new(10.0, 0.0),
                        Align2::LEFT_CENTER,
                        format!("{:>2}  {icone}  {nome}", i + 1),
                        fonte,
                        cor_txt,
                    );
                    let mut x = rect.right() - 8.0;
                    if !seg.duracao.trim().is_empty() {
                        let g = painter.layout_no_wrap(
                            seg.duracao.trim().to_string(),
                            FontId::monospace(11.5),
                            p.texto_mais_fraco,
                        );
                        x -= g.size().x;
                        painter.galley(
                            egui::pos2(x, rect.center().y - g.size().y / 2.0),
                            g,
                            p.texto_mais_fraco,
                        );
                        x -= 8.0;
                    }
                    if seg.alterado() {
                        painter.circle_filled(egui::pos2(x - 4.0, rect.center().y), 3.5, p.ambar);
                    }
                    if r.clicked() {
                        ir_para = Some(i);
                    }
                    let r = if seg.alterado() {
                        r.on_hover_text("Texto alterado (ainda não gravado no Sintonia)")
                    } else {
                        r
                    };
                    let _ = r;
                }
            });
        if let Some(i) = ir_para {
            self.selecionado = Some(i);
            self.ir_para_bloco(i);
        }

        ui.separator();
        let Some(i) = self
            .selecionado
            .filter(|i| *i < self.roteiro.segmentos.len())
        else {
            ui.label(RichText::new("Selecione um bloco para editar.").weak());
            return;
        };
        let total = self.roteiro.segmentos.len();
        let mut mover: Option<(usize, usize)> = None;
        let mut excluir = false;
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "Bloco {} — {}",
                    i + 1,
                    self.roteiro.segmentos[i].tipo.rotulo()
                ))
                .font(tema::forte(13.5)),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .small_button("🗑")
                    .on_hover_text("Tirar do teleprompter (não apaga do Sintonia)")
                    .clicked()
                {
                    excluir = true;
                }
                if ui
                    .add_enabled(i + 1 < total, egui::Button::new("⬇").small())
                    .clicked()
                {
                    mover = Some((i, i + 1));
                }
                if ui
                    .add_enabled(i > 0, egui::Button::new("⬆").small())
                    .clicked()
                {
                    mover = Some((i, i - 1));
                }
            });
        });
        let seg = &mut self.roteiro.segmentos[i];
        let r1 = ui.add(
            egui::TextEdit::singleline(&mut seg.titulo)
                .hint_text("Título / retranca")
                .desired_width(f32::INFINITY),
        );
        if seg.item_id.is_some() && r1.changed() {
            // A retranca não é gravada de volta; só o texto.
            r1.on_hover_text("O título é só para o teleprompter — não é gravado no espelho.");
        }
        if seg.tipo == Tipo::Intervalo {
            ui.label(
                RichText::new("Intervalo comercial (break). O apresentador vê só a marcação.")
                    .weak(),
            );
        }
        let r2 = ScrollArea::vertical()
            .id_salt("editor")
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut seg.texto)
                        .hint_text("Texto que o apresentador vai ler…")
                        .desired_width(f32::INFINITY)
                        .desired_rows(12)
                        .font(FontId::proportional(16.0)),
                )
            })
            .inner;
        if r2.changed() {
            self.editado = true;
        }
        if let Some((a, b)) = mover {
            self.roteiro.segmentos.swap(a, b);
            self.selecionado = Some(b);
            self.editado = true;
        }
        if excluir {
            self.roteiro.segmentos.remove(i);
            self.selecionado = if self.roteiro.segmentos.is_empty() {
                None
            } else {
                Some(i.min(self.roteiro.segmentos.len() - 1))
            };
            self.editado = true;
        }
    }
}
