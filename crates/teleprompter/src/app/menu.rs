//! Barra de cima: marca do Sintonia e menus.

use super::*;

impl App {
    pub(super) fn barra_menu(&mut self, ui: &mut Ui) {
        let p = self.p();
        egui::MenuBar::new().ui(ui, |ui| {
            self.marca(ui, 34.0);
            ui.add_space(14.0);
            ui.menu_button("Arquivo", |ui| {
                if ui.button("Novo roteiro").clicked() {
                    self.pedir(Pendente::Novo);
                    ui.close();
                }
                if ui.button("Abrir TXT…").clicked() {
                    ui.close();
                    self.pedir(Pendente::AbrirTxt);
                }
                if ui.button("Salvar como TXT…").clicked() {
                    ui.close();
                    self.salvar_txt();
                }
            });
            ui.menu_button("Sintonia", |ui| {
                let logado = self.usuario.is_some();
                if ui.add_enabled(logado, egui::Button::new("Abrir espelho…")).clicked() {
                    self.seletor_aberto = true;
                    self.api.enviar(Comando::ListarEspelhos);
                    ui.close();
                }
                let id_espelho = match &self.roteiro.origem {
                    Origem::Espelho { id, .. } => Some(id.clone()),
                    _ => None,
                };
                if ui.add_enabled(logado && id_espelho.is_some(), egui::Button::new("Recarregar espelho atual")).clicked() {
                    ui.close();
                    if self.editado {
                        self.avisar("Há textos alterados. Salve no Sintonia (ou em TXT) antes de recarregar.", true);
                    } else {
                        self.recarregar_id = id_espelho.clone();
                        self.api.enviar(Comando::ListarEspelhos);
                    }
                }
                let alterados = self.roteiro.alterados();
                let pode_salvar = logado && id_espelho.is_some() && self.pode_editar_espelho() && alterados > 0;
                if ui
                    .add_enabled(pode_salvar, egui::Button::new(format!("Gravar textos alterados no espelho ({alterados})")))
                    .on_disabled_hover_text("Precisa estar logado, com um espelho aberto, permissão de Editar no Espelho e algum texto alterado.")
                    .clicked()
                {
                    ui.close();
                    self.salvar_no_sintonia();
                }
                ui.separator();
                if logado {
                    if ui.button("Sair do Sintonia").clicked() {
                        ui.close();
                        self.api.enviar(Comando::Sair);
                    }
                } else if ui.button("Entrar no Sintonia…").clicked() {
                    ui.close();
                    self.tela = Tela::Login;
                }
            });
            ui.menu_button("Apresentador", |ui| {
                ui.set_width(320.0);
                ui.label(RichText::new("Em qual tela vai o texto do apresentador?").font(tema::forte(13.0)));
                ui.label(RichText::new("Clique no monitor: a tela já abre em tela cheia nele.").size(11.5).color(p.texto_mais_fraco));
                ui.add_space(4.0);
                self.seletor_monitor(ui);
                ui.separator();
                ui.label(RichText::new("Espelhamento (vidro do teleprompter)").font(tema::forte(13.0)));
                ui.checkbox(&mut self.cfg.espelhar_horizontal, "Espelhar horizontal");
                ui.checkbox(&mut self.cfg.espelhar_vertical, "Inverter vertical");
                ui.checkbox(&mut self.cfg.previa_espelhada, "Prévia espelhada também");
            });
            ui.menu_button("Predefinições", |ui| {
                if self.cfg.predefinicoes.is_empty() {
                    ui.label(RichText::new("Nenhuma predefinição salva ainda.").weak());
                }
                let mut aplicar = None;
                for (i, pre) in self.cfg.predefinicoes.iter().enumerate() {
                    if ui.button(&pre.nome).on_hover_text("Aplicar esta predefinição").clicked() {
                        aplicar = Some(i);
                    }
                }
                if let Some(i) = aplicar {
                    self.aplicar_predefinicao(i);
                    ui.close();
                }
                ui.separator();
                if ui.button("Salvar a configuração atual…").clicked() {
                    self.cfg.mostrar_formatacao = true;
                    ui.close();
                }
            });
            if ui
                .selectable_label(self.cfg.mostrar_formatacao, "Formatação")
                .on_hover_text("Mostrar / esconder o painel de formatação")
                .clicked()
            {
                self.cfg.mostrar_formatacao = !self.cfg.mostrar_formatacao;
            }
            ui.menu_button("Aparência", |ui| {
                let cheia = ui.ctx().input(|i| i.viewport().fullscreen.unwrap_or(false));
                if ui.selectable_label(cheia, "Tela cheia do operador (F11)").clicked() {
                    let ctx = ui.ctx().clone();
                    self.alternar_tela_cheia_operador(&ctx);
                    ui.close();
                }
                ui.separator();
                ui.selectable_value(&mut self.cfg.tema_claro, false, "Tema escuro");
                ui.selectable_value(&mut self.cfg.tema_claro, true, "Tema claro (fundo branco)");
                ui.label(RichText::new("Só a tela do operador. As cores do texto do\napresentador ficam em Formatação.").small().weak());
            });
            ui.menu_button("Ajuda", |ui| {
                ui.label(RichText::new("Atalhos").strong());
                for (k, d) in [
                    ("Espaço", "rolar / pausar"),
                    ("Segurar avançar / voltar", "passa rápido para a frente / para trás"),
                    ("F11 (operador)", "tela cheia desta janela"),
                    ("Botão direito (segurar)", "passa rápido sobre o texto"),
                    ("Shift + ↓ / ↑", "passa rápido com o teclado"),
                    ("End", "para a rolagem"),
                    ("↑ / ↓ (segurar)", "move o texto para cima / baixo"),
                    ("→ / ←", "velocidade +5 % / −5 % (Shift: 1 %)"),
                    ("Home", "volta ao começo e zera o cronômetro"),
                    ("Page Up / Page Down", "lauda (bloco) anterior / próxima"),
                    ("F5", "inicia / pausa o cronômetro"),
                    ("F11 / Esc", "tela cheia do apresentador"),
                    ("F6", "inicia / pausa o cronômetro da entrevista"),
                    ("F7", "tela só cronômetro (liga / desliga)"),
                    ("Arrastar o texto", "rola o texto"),
                    ("Clique no texto", "rolar / parar"),
                    ("Rodinha do mouse", "velocidade ±5 % (Shift: 1 %)"),
                    ("Clique na rodinha", "para a rolagem"),
                ] {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(k).monospace());
                        ui.label(d);
                    });
                }
                ui.separator();
                ui.label(format!("Sintonia Teleprompter {}", env!("CARGO_PKG_VERSION")));
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                match &self.usuario {
                    Some(u) => {
                        let r = ui.label(RichText::new(u.nome()).font(tema::forte(13.5)).color(p.texto));
                        let (q, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
                        icones::desenhar(ui, q, Icone::Usuario, p.texto_fraco);
                        r
                    }
                    None => ui.label(RichText::new("sem login (modo TXT)").color(p.texto_mais_fraco)),
                };
                // "NO AR" pulsando, como o relógio da barra lateral do Sintonia.
                if self.rodando {
                    let t = ui.input(|i| i.time) as f32;
                    let alfa = 0.35 + 0.65 * (t * std::f32::consts::PI / 1.6).cos().abs();
                    ui.label(RichText::new("NO AR").size(11.5).strong().color(p.vermelho));
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                    ui.painter().circle_filled(r.center(), 3.5, p.vermelho.gamma_multiply(alfa));
                    ui.separator();
                }
                if self.api.ocupado() {
                    ui.spinner();
                }
                let titulo = if self.editado { format!("{} •", self.roteiro.titulo) } else { self.roteiro.titulo.clone() };
                ui.label(RichText::new(titulo).color(p.texto_fraco));
            });
        });
    }
}
