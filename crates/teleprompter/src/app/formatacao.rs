//! Painel de formatação (escondido por padrão) e predefinições.

use super::*;

impl App {
    pub(super) fn aplicar_predefinicao(&mut self, i: usize) {
        if let Some(pre) = self.cfg.predefinicoes.get(i).cloned() {
            let estava_em_tela_cheia = self.cfg.tela_cheia_apresentador;
            self.cfg.aplicar(&pre);
            // Mudou de monitor com a tela do apresentador já em tela cheia: reabre nele.
            if self.apresentador_aberto && estava_em_tela_cheia && self.cfg.tela_cheia_apresentador
            {
                self.apresentador_aberto = false;
                self.reabrir_apresentador = true;
            }
            self.avisar(format!("Predefinição \"{}\" aplicada.", pre.nome), false);
        }
    }

    /// Predefinições: guardam fonte, tamanho, cores, espelhamento, velocidade,
    /// monitor e tela cheia do apresentador — para trocar tudo de uma vez.
    pub(super) fn secao_predefinicoes(&mut self, ui: &mut Ui) {
        let p = self.p();
        ui.label(
            RichText::new("PREDEFINIÇÕES")
                .size(10.5)
                .color(p.texto_mais_fraco),
        );
        let mut aplicar = None;
        let mut excluir = None;
        for (i, pre) in self.cfg.predefinicoes.iter().enumerate() {
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(&pre.nome)
                            .min_size(Vec2::new((ui.available_width() - 48.0).max(60.0), 24.0)),
                    )
                    .on_hover_text("Aplicar")
                    .clicked()
                {
                    aplicar = Some(i);
                }
                if ui
                    .small_button("🗑")
                    .on_hover_text("Excluir esta predefinição")
                    .clicked()
                {
                    excluir = Some(i);
                }
            });
        }
        if let Some(i) = aplicar {
            self.aplicar_predefinicao(i);
        }
        if let Some(i) = excluir {
            let nome = self.cfg.predefinicoes.remove(i).nome;
            self.avisar(format!("Predefinição \"{nome}\" excluída."), false);
        }
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.predef_nome)
                    .hint_text("Nome (ex.: Jornal da Noite)")
                    .desired_width((ui.available_width() - 90.0).max(80.0)),
            );
            let pode = !self.predef_nome.trim().is_empty();
            if ui
                .add_enabled(pode, egui::Button::new("Salvar"))
                .on_hover_text("Guarda a formatação atual, o monitor e a tela cheia do apresentador. Mesmo nome = atualiza.")
                .clicked()
            {
                let nome = self.predef_nome.trim().to_string();
                self.cfg.guardar_predefinicao(&nome);
                self.predef_nome.clear();
                self.avisar(format!("Predefinição \"{nome}\" salva."), false);
            }
        });
    }

    pub(super) fn painel_formatacao(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading("Formatação");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .small_button("✕")
                        .on_hover_text("Esconder (reabre no menu Formatação)")
                        .clicked()
                    {
                        self.cfg.mostrar_formatacao = false;
                    }
                });
            });
            ui.add_space(4.0);
            self.secao_predefinicoes(ui);
            ui.separator();

            ui.label("Fonte");
            let automatica = self.fontes.automatica().unwrap_or("padrão").to_string();
            let atual = if self.cfg.font_family.is_empty() {
                format!("Automática ({automatica})")
            } else {
                self.cfg.font_family.clone()
            };
            egui::ComboBox::from_id_salt("fonte")
                .selected_text(atual)
                .width(ui.available_width() - 8.0)
                .height(360.0)
                .show_ui(ui, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.busca_fonte)
                            .hint_text("🔍 procurar fonte"),
                    );
                    if ui
                        .selectable_label(
                            self.cfg.font_family.is_empty(),
                            format!("Automática ({automatica})"),
                        )
                        .clicked()
                    {
                        self.cfg.font_family.clear();
                    }
                    let busca = self.busca_fonte.to_lowercase();
                    for f in self
                        .fontes
                        .familias
                        .iter()
                        .filter(|f| busca.is_empty() || f.to_lowercase().contains(&busca))
                    {
                        if ui.selectable_label(&self.cfg.font_family == f, f).clicked() {
                            self.cfg.font_family = f.clone();
                        }
                    }
                });
            ui.checkbox(&mut self.cfg.negrito, "Negrito");
            ui.add_space(4.0);
            ui.label("Tamanho da letra (na tela do apresentador)");
            ui.add(
                egui::Slider::new(&mut self.cfg.font_size, 20.0..=220.0)
                    .suffix(" px")
                    .step_by(1.0),
            );
            ui.horizontal(|ui| {
                for (rot, v) in [("P", 48.0), ("M", 72.0), ("G", 100.0), ("GG", 140.0)] {
                    if ui.small_button(rot).clicked() {
                        self.cfg.font_size = v;
                    }
                }
            });
            ui.label("Espaço entre linhas");
            ui.add(egui::Slider::new(&mut self.cfg.entrelinha, 1.0..=2.5).step_by(0.05));
            ui.label("Margem lateral");
            ui.add(
                egui::Slider::new(&mut self.cfg.margem_pct, 0.0..=30.0)
                    .suffix(" %")
                    .step_by(0.5),
            );
            ui.horizontal(|ui| {
                ui.label("Alinhamento");
                ui.selectable_value(&mut self.cfg.alinhamento, Alinhamento::Esquerda, "Esquerda");
                ui.selectable_value(&mut self.cfg.alinhamento, Alinhamento::Centro, "Centro");
            });
            ui.checkbox(&mut self.cfg.caixa_alta, "CAIXA ALTA");
            ui.checkbox(&mut self.cfg.mostrar_titulos, "Mostrar retrancas (títulos)");

            ui.add_space(6.0);
            egui::Grid::new("cores").num_columns(2).show(ui, |ui| {
                ui.label("Cor do texto");
                ui.color_edit_button_srgb(&mut self.cfg.cor_texto);
                ui.end_row();
                ui.label("Cor do fundo");
                ui.color_edit_button_srgb(&mut self.cfg.cor_fundo);
                ui.end_row();
                ui.label("Cor dos títulos");
                ui.color_edit_button_srgb(&mut self.cfg.cor_titulo);
                ui.end_row();
                ui.label("Cor do destaque")
                    .on_hover_text("Texto entre << e >> no roteiro");
                ui.color_edit_button_srgb(&mut self.cfg.cor_destaque);
                ui.end_row();
            });
            ui.horizontal(|ui| {
                if ui.small_button("Branco no preto").clicked() {
                    self.cfg.cor_texto = [255, 255, 255];
                    self.cfg.cor_fundo = [0, 0, 0];
                }
                if ui.small_button("Amarelo no preto").clicked() {
                    self.cfg.cor_texto = [255, 230, 0];
                    self.cfg.cor_fundo = [0, 0, 0];
                }
                if ui.small_button("Preto no branco").clicked() {
                    self.cfg.cor_texto = [0, 0, 0];
                    self.cfg.cor_fundo = [255, 255, 255];
                }
            });

            ui.separator();
            ui.heading("Leitura");
            ui.checkbox(
                &mut self.cfg.espelhar_horizontal,
                "Espelhar horizontal (vidro)",
            );
            ui.checkbox(&mut self.cfg.espelhar_vertical, "Inverter vertical");
            ui.label(
                RichText::new("O espelhamento vale só na tela do apresentador.")
                    .small()
                    .weak(),
            );
            ui.checkbox(&mut self.cfg.mostrar_marcador, "Seta de leitura");
            ui.label("Altura da linha de leitura");
            ui.add(egui::Slider::new(&mut self.cfg.marcador_pct, 5.0..=70.0).suffix(" %"));
            ui.checkbox(
                &mut self.cfg.crono_automatico,
                "Cronômetro dispara ao rolar (Home zera)",
            );
            ui.label("Ritmo para estimar a leitura");
            ui.add(
                egui::Slider::new(&mut self.cfg.palavras_por_minuto, 80..=250)
                    .suffix(" palavras/min"),
            );

            ui.separator();
            ui.heading("Tela do apresentador");
            let txt = if self.apresentador_aberto {
                "Fechar tela do apresentador"
            } else {
                "Abrir tela do apresentador"
            };
            let p = self.p();
            let (cor_txt, fundo) = if self.apresentador_aberto {
                (p.texto, p.panel_2)
            } else {
                (Color32::WHITE, p.vermelho)
            };
            if ui
                .add(
                    egui::Button::new(RichText::new(txt).font(tema::forte(14.0)).color(cor_txt))
                        .fill(fundo)
                        .min_size(Vec2::new(ui.available_width(), 32.0)),
                )
                .clicked()
            {
                self.apresentador_aberto = !self.apresentador_aberto;
            }
            ui.checkbox(&mut self.cfg.tela_cheia_apresentador, "Tela cheia (F11)");
            ui.horizontal(|ui| {
                ui.label("Monitor");
                ui.add(egui::DragValue::new(&mut self.cfg.monitor_apresentador).range(0..=8));
                ui.label(RichText::new("(0 = principal)").small().weak());
            });
            ui.checkbox(
                &mut self.cfg.mostrar_relogio_apresentador,
                "Mostrar cronômetro para o apresentador",
            );
            ui.label(
                RichText::new(format!(
                    "Tela do apresentador: {:.0} × {:.0}",
                    self.tamanho_apresentador.x, self.tamanho_apresentador.y
                ))
                .small()
                .weak(),
            );
        });
    }
}
