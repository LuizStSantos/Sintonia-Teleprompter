//! Janela para escolher o espelho do dia.

use super::*;

impl App {
    /// Escolha do espelho: navega por dia (◀ ▶), filtra por programa e mostra
    /// os itens do espelho marcado antes de abrir.
    pub(super) fn seletor_espelho(&mut self, ctx: &egui::Context) {
        if !self.seletor_aberto {
            return;
        }
        let p = self.p();
        let data = chrono::NaiveDate::parse_from_str(self.filtro_data.trim(), "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        let hoje_d = chrono::Local::now().date_naive();
        let busca = self.filtro_busca.to_lowercase();
        let iso = data.format("%Y-%m-%d").to_string();

        // Espelhos do dia (todos os programas), para os filtros.
        let do_dia: Vec<(EspelhoResumo, &Value)> = self
            .espelhos
            .iter()
            .map(|v| (EspelhoResumo::de(v), v))
            .filter(|(r, _)| r.data == iso)
            .collect();
        let mut programas: Vec<String> = do_dia.iter().map(|(r, _)| r.programa.clone()).collect();
        programas.sort();
        programas.dedup();
        let mut lista: Vec<(EspelhoResumo, Value)> = do_dia
            .iter()
            .filter(|(r, _)| {
                self.filtro_programa
                    .as_ref()
                    .is_none_or(|f| &r.programa == f)
            })
            .filter(|(r, _)| {
                busca.is_empty()
                    || r.titulo.to_lowercase().contains(&busca)
                    || r.programa.to_lowercase().contains(&busca)
            })
            .map(|(r, v)| (r.clone(), (*v).clone()))
            .collect();
        // Na ordem em que vão ao ar.
        lista.sort_by(|a, b| a.0.horario.cmp(&b.0.horario));
        if self
            .espelho_marcado
            .as_ref()
            .is_none_or(|id| !lista.iter().any(|(r, _)| &r.id == id))
        {
            self.espelho_marcado = lista.first().map(|(r, _)| r.id.clone());
        }

        let mut abrir: Option<Value> = None;
        let mut mudar_dia: Option<chrono::NaiveDate> = None;
        let mut fechar = false;
        let tela = ctx.content_rect();
        let tam = Vec2::new(
            (tela.width() - 80.0).clamp(560.0, 980.0),
            (tela.height() - 100.0).clamp(380.0, 620.0),
        );
        let resp = egui::Modal::new(egui::Id::unique("sintonia-seletor-espelho")).show(ctx, |ui| {
            ui.set_width(tam.x);
            ui.set_max_width(tam.x);
            ui.set_height(tam.y);
            // Cabeçalho
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Abrir espelho")
                        .font(tema::titulo(20.0))
                        .color(p.texto),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("✕").on_hover_text("Fechar (Esc)").clicked() {
                        fechar = true;
                    }
                    if ui
                        .button("⟳ Atualizar")
                        .on_hover_text("Buscar de novo no Sintonia")
                        .clicked()
                    {
                        self.api.enviar(Comando::ListarEspelhos);
                    }
                    if self.api.ocupado() {
                        ui.spinner();
                    }
                });
            });
            ui.add_space(6.0);
            // Navegação por dia
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(RichText::new("◀").size(16.0))
                            .min_size(Vec2::splat(32.0)),
                    )
                    .on_hover_text("Dia anterior")
                    .clicked()
                {
                    mudar_dia = data.pred_opt();
                }
                let dia_semana =
                    DIAS_SEMANA[data.format("%u").to_string().parse::<usize>().unwrap_or(1) - 1];
                let rel = match (data - hoje_d).num_days() {
                    0 => " · hoje",
                    1 => " · amanhã",
                    -1 => " · ontem",
                    _ => "",
                };
                ui.label(
                    RichText::new(format!("{dia_semana}, {}{rel}", data.format("%d/%m/%Y")))
                        .font(tema::forte(16.0))
                        .color(p.texto),
                );
                if ui
                    .add(
                        egui::Button::new(RichText::new("▶").size(16.0))
                            .min_size(Vec2::splat(32.0)),
                    )
                    .on_hover_text("Próximo dia")
                    .clicked()
                {
                    mudar_dia = data.succ_opt();
                }
                ui.add_space(8.0);
                if ui.selectable_label(data == hoje_d, "Hoje").clicked() {
                    mudar_dia = Some(hoje_d);
                }
                if ui
                    .selectable_label(Some(data) == hoje_d.succ_opt(), "Amanhã")
                    .clicked()
                {
                    mudar_dia = hoje_d.succ_opt();
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.filtro_busca)
                            .hint_text("🔍 procurar")
                            .desired_width(180.0),
                    );
                });
            });
            // Programas do dia
            if programas.len() > 1 {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("PROGRAMA")
                            .size(10.0)
                            .color(p.texto_mais_fraco),
                    );
                    if ui
                        .selectable_label(self.filtro_programa.is_none(), "Todos")
                        .clicked()
                    {
                        self.filtro_programa = None;
                    }
                    for prog in &programas {
                        let ativo = self.filtro_programa.as_ref() == Some(prog);
                        if ui.selectable_label(ativo, prog).clicked() {
                            self.filtro_programa = if ativo { None } else { Some(prog.clone()) };
                        }
                    }
                });
            }
            ui.separator();

            if lista.is_empty() {
                ui.add_space(30.0);
                ui.vertical_centered(|ui| {
                    let msg = if self.espelhos.is_empty() {
                        "Nenhum espelho carregado do Sintonia."
                    } else {
                        "Nenhum espelho neste dia."
                    };
                    ui.label(RichText::new(msg).size(16.0).color(p.texto_fraco));
                    // Atalhos para os dias mais próximos que têm espelho.
                    let mut dias: Vec<chrono::NaiveDate> = self
                        .espelhos
                        .iter()
                        .filter_map(|v| v["data"].as_str())
                        .filter_map(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                        .collect();
                    dias.sort_by_key(|d| ((*d - data).num_days().abs(), *d));
                    dias.dedup();
                    if !dias.is_empty() {
                        ui.add_space(10.0);
                        ui.label(RichText::new("Dias com espelho:").color(p.texto_mais_fraco));
                        ui.horizontal_wrapped(|ui| {
                            for d in dias.iter().take(8) {
                                let n = self
                                    .espelhos
                                    .iter()
                                    .filter(|v| {
                                        v["data"].as_str()
                                            == Some(&d.format("%Y-%m-%d").to_string())
                                    })
                                    .count();
                                if ui.button(format!("{} ({n})", d.format("%d/%m"))).clicked() {
                                    mudar_dia = Some(*d);
                                }
                            }
                        });
                    }
                });
                return;
            }

            // Lista (esquerda) + itens do marcado (direita)
            let altura = ui.available_height() - 4.0;
            ui.horizontal_top(|ui| {
                let largura_lista = (ui.available_width() * 0.45).max(260.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(largura_lista, altura),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ScrollArea::vertical()
                            .id_salt("lista-espelhos")
                            .show(ui, |ui| {
                                for (r, v) in &lista {
                                    let marcado = self.espelho_marcado.as_ref() == Some(&r.id);
                                    let roteiro = Roteiro::do_espelho(v);
                                    let dur = roteiro.duracao_total();
                                    let resp = egui::Frame::new()
                                        .fill(if marcado { p.panel_2 } else { p.panel })
                                        .stroke(egui::Stroke::new(
                                            1.0,
                                            if marcado { p.vermelho } else { p.borda },
                                        ))
                                        .corner_radius(8.0)
                                        .inner_margin(10.0)
                                        .show(ui, |ui| {
                                            ui.set_width(largura_lista - 28.0);
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new(if r.horario.is_empty() {
                                                        "--:--".into()
                                                    } else {
                                                        r.horario.clone()
                                                    })
                                                    .font(FontId::monospace(18.0))
                                                    .color(p.ambar),
                                                );
                                                ui.vertical(|ui| {
                                                    ui.label(
                                                        RichText::new(if r.programa.is_empty() {
                                                            "Sem programa"
                                                        } else {
                                                            &r.programa
                                                        })
                                                        .font(tema::forte(14.5))
                                                        .color(p.texto),
                                                    );
                                                    ui.label(
                                                        RichText::new(&r.titulo)
                                                            .size(12.0)
                                                            .color(p.texto_fraco),
                                                    );
                                                    ui.label(
                                                        RichText::new(format!(
                                                            "{} itens{}",
                                                            r.itens,
                                                            if dur > 0 {
                                                                format!(" · {}", fmt_tempo(dur))
                                                            } else {
                                                                String::new()
                                                            }
                                                        ))
                                                        .size(11.5)
                                                        .color(p.texto_mais_fraco),
                                                    );
                                                });
                                            });
                                        })
                                        .response
                                        .interact(Sense::click());
                                    if resp.clicked() {
                                        self.espelho_marcado = Some(r.id.clone());
                                    }
                                    if resp.double_clicked() {
                                        abrir = Some(v.clone());
                                    }
                                    resp.on_hover_text(
                                        "Clique para ver os itens · duplo clique abre",
                                    );
                                    ui.add_space(6.0);
                                }
                            });
                    },
                );
                ui.separator();
                // Prévia do espelho marcado
                let marcado = lista
                    .iter()
                    .find(|(r, _)| self.espelho_marcado.as_ref() == Some(&r.id));
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), altura),
                    Layout::top_down(Align::Min),
                    |ui| {
                        let Some((r, v)) = marcado else { return };
                        let roteiro = Roteiro::do_espelho(v);
                        ui.label(
                            RichText::new(if r.programa.is_empty() {
                                &r.titulo
                            } else {
                                &r.programa
                            })
                            .font(tema::titulo(18.0))
                            .color(p.texto),
                        );
                        ui.label(
                            RichText::new(format!(
                                "{} · {} · {}",
                                r.data_br(),
                                if r.horario.is_empty() {
                                    "sem horário"
                                } else {
                                    &r.horario
                                },
                                fmt_tempo(roteiro.duracao_total())
                            ))
                            .color(p.texto_fraco),
                        );
                        if let Some(quem) = v["alteradoPorNome"].as_str().filter(|q| !q.is_empty())
                        {
                            let quando = v["alteradoEm"]
                                .as_str()
                                .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                                .map(|t| {
                                    t.with_timezone(&chrono::Local)
                                        .format(" em %d/%m às %H:%M")
                                        .to_string()
                                })
                                .unwrap_or_default();
                            ui.label(
                                RichText::new(format!("Última alteração: {quem}{quando}"))
                                    .size(11.5)
                                    .color(p.texto_mais_fraco),
                            );
                        }
                        ui.add_space(6.0);
                        let altura_itens = ui.available_height() - 48.0;
                        ScrollArea::vertical()
                            .id_salt("itens-espelho")
                            .max_height(altura_itens)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for (i, seg) in roteiro.segmentos.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        let (icone, c) = match seg.tipo {
                                            Tipo::Intervalo => ("📺", p.ambar),
                                            Tipo::Entrevistado => ("🎤", p.texto),
                                            Tipo::Materia => ("📰", p.texto),
                                            _ => ("📝", p.texto),
                                        };
                                        ui.label(
                                            RichText::new(format!("{:>2}", i + 1))
                                                .font(FontId::monospace(12.0))
                                                .color(p.texto_mais_fraco),
                                        );
                                        ui.label(
                                            RichText::new(format!(
                                                "{icone}  {}",
                                                if seg.titulo.is_empty() {
                                                    "(sem título)"
                                                } else {
                                                    &seg.titulo
                                                }
                                            ))
                                            .color(c),
                                        );
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                ui.label(
                                                    RichText::new(seg.duracao.trim())
                                                        .font(FontId::monospace(12.0))
                                                        .color(p.texto_mais_fraco),
                                                );
                                                if seg.tipo != Tipo::Intervalo
                                                    && seg.texto.trim().is_empty()
                                                {
                                                    ui.label(
                                                        RichText::new("sem texto")
                                                            .size(11.0)
                                                            .color(p.ambar),
                                                    );
                                                }
                                            },
                                        );
                                    });
                                }
                            });
                        ui.add_space(6.0);
                        let largura = ui.available_width();
                        if ui
                            .add_sized(
                                [largura, 36.0],
                                egui::Button::new(
                                    RichText::new("Abrir no teleprompter")
                                        .font(tema::forte(15.0))
                                        .color(Color32::WHITE),
                                )
                                .fill(p.vermelho),
                            )
                            .clicked()
                        {
                            abrir = Some(v.clone());
                        }
                    },
                );
            });
        });
        if resp.should_close() || fechar {
            self.seletor_aberto = false;
        }
        if let Some(d) = mudar_dia {
            self.filtro_data = d.format("%Y-%m-%d").to_string();
            self.filtro_programa = None;
            self.espelho_marcado = None;
        }
        if let Some(esp) = abrir {
            self.seletor_aberto = false;
            self.pedir(Pendente::AbrirEspelho(esp));
        }
    }
}
