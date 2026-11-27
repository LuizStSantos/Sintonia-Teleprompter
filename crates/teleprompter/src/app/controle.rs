//! Rolagem, velocidade, "passar rápido", atalhos de teclado e mouse.

use super::*;

impl App {
    pub(super) fn em_por_segundo(&self) -> f32 {
        // Curva: dá controle fino nas velocidades de leitura (10–20 %) e ainda
        // chega bem rápido em 100 %. 15 % ≈ leitura normal.
        (self.cfg.velocidade_pct / 100.0).powf(1.5) * VELOCIDADE_MAXIMA_EM
    }

    pub(super) fn ir_para_bloco(&mut self, i: usize) {
        if let Some(t) = self.medidas.topos_em.get(i) {
            self.rolagem_em = *t;
            self.selecionado = Some(i);
        }
    }

    pub(super) fn bloco_relativo(&mut self, passo: i32) {
        let atual = self.medidas.bloco_atual(self.rolagem_em).unwrap_or(0) as i32;
        // "Anterior" no meio de um bloco volta para o começo dele.
        let alvo = if passo < 0
            && self
                .medidas
                .topos_em
                .get(atual as usize)
                .is_some_and(|t| self.rolagem_em - t > 0.5)
        {
            atual
        } else {
            atual + passo
        };
        let max = self.medidas.topos_em.len().saturating_sub(1) as i32;
        self.ir_para_bloco(alvo.clamp(0, max) as usize);
    }

    /// Liga/desliga a rolagem. Como no teleprompter antigo (Apotec), o
    /// cronômetro dispara sozinho quando o texto começa a rolar.
    pub(super) fn alternar_rolagem(&mut self) {
        self.rodando = !self.rodando;
        if self.rodando && self.cfg.crono_automatico {
            self.crono.iniciar();
        }
    }

    /// Volta ao começo do texto e zera o cronômetro (tecla Home, igual ao Apotec).
    pub(super) fn ir_inicio(&mut self) {
        self.rolagem_em = 0.0;
        self.rodando = false;
        if self.cfg.crono_automatico {
            self.crono.zerar();
        }
    }

    pub(super) fn mudar_velocidade(&mut self, delta: f32) {
        self.cfg.velocidade_pct = (self.cfg.velocidade_pct + delta).clamp(0.0, 100.0).round();
    }

    /// Passo da rodinha/setas: 5 %; com Shift, ajuste fino de 1 %.
    pub(super) fn passo_velocidade(ctx: &egui::Context) -> f32 {
        if ctx.input(|i| i.modifiers.shift) {
            1.0
        } else {
            5.0
        }
    }

    /// Botão que faz uma coisa no clique e "passa rápido" enquanto segurado.
    /// Devolve `true` no clique curto.
    pub(super) fn botao_segurar(
        &mut self,
        ui: &mut Ui,
        resp: &egui::Response,
        direcao: f32,
    ) -> bool {
        if resp.is_pointer_button_down_on() {
            let segurando = ui
                .input(|i| i.pointer.press_start_time().map(|t| i.time - t))
                .unwrap_or(0.0);
            if segurando > 0.3 {
                self.turbo_pedido = direcao;
                self.turbo_usado = true;
            }
        }
        resp.clicked() && !self.turbo_usado
    }

    /// Mouse sobre o texto (prévia ou apresentador), no jeito do Apotec:
    /// arrastar rola o texto na hora, um clique roda/para, a rodinha muda a
    /// velocidade e o clique na rodinha para.
    /// `invertido`: a tela está espelhada na vertical (o arrasto acompanha a imagem).
    pub(super) fn mouse_no_texto(
        &mut self,
        ui: &Ui,
        resp: &egui::Response,
        px: f32,
        invertido: bool,
    ) {
        // Arrasto imediato: não espera o mouse andar alguns pixels para "virar arrasto".
        if resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_down()) {
            let dy = ui.input(|i| i.pointer.delta().y);
            if dy != 0.0 {
                let sinal = if invertido { -1.0 } else { 1.0 };
                self.rolagem_em -= sinal * dy / px;
                ui.ctx().request_repaint();
            }
        }
        // Clique simples (sem arrastar): roda / para.
        if resp.clicked() {
            self.alternar_rolagem();
        }
        if resp.hovered() {
            // Cada "clique" da rodinha: para cima acelera, para baixo freia.
            let passos: f32 = ui.input(|i| {
                i.events
                    .iter()
                    .filter_map(|e| match e {
                        egui::Event::MouseWheel { delta, .. } if delta.y != 0.0 => {
                            Some(delta.y.signum())
                        }
                        _ => None,
                    })
                    .sum()
            });
            if passos != 0.0 {
                self.mudar_velocidade(passos * Self::passo_velocidade(ui.ctx()));
            }
        }
        // Botão direito segurado sobre o texto: passa rápido para a frente.
        if resp.hovered() && ui.input(|i| i.pointer.secondary_down()) {
            self.turbo_pedido = if invertido { -1.0 } else { 1.0 };
        }
        if resp.clicked_by(egui::PointerButton::Middle) {
            self.rodando = false;
        }
    }

    /// Liga/desliga a tela "só cronômetro" do apresentador. Ao ligar, se o
    /// bloco atual for um entrevistado com duração no espelho, já usa essa duração.
    pub(super) fn alternar_so_cronometro(&mut self) {
        self.so_cronometro = !self.so_cronometro;
        if self.so_cronometro
            && self.cfg.entrevista_com_tempo
            && self.meta_entrevista_txt.trim().is_empty()
            && let Some(seg) = self
                .medidas
                .bloco_atual(self.rolagem_em)
                .and_then(|i| self.roteiro.segmentos.get(i))
            && seg.tipo == Tipo::Entrevistado
            && !seg.duracao.trim().is_empty()
        {
            self.meta_entrevista_txt = seg.duracao.trim().to_string();
        }
    }

    pub(super) fn atalhos(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        // Mesmas teclas do Apotec, para a equipe não precisar reaprender.
        let (f6, f7) = ctx.input(|i| (i.key_pressed(Key::F6), i.key_pressed(Key::F7)));
        if f6 {
            self.crono_entrevista.alternar();
        }
        if f7 {
            self.alternar_so_cronometro();
        }
        let (espaco, cima, baixo, dir, esq, fim, pgup, pgdn, home, f5) = ctx.input(|i| {
            (
                i.key_pressed(Key::Space),
                i.key_down(Key::ArrowUp),
                i.key_down(Key::ArrowDown),
                i.key_pressed(Key::ArrowRight),
                i.key_pressed(Key::ArrowLeft),
                i.key_pressed(Key::End),
                i.key_pressed(Key::PageUp),
                i.key_pressed(Key::PageDown),
                i.key_pressed(Key::Home),
                i.key_pressed(Key::F5),
            )
        });
        if espaco {
            self.alternar_rolagem();
        }
        // Setas ↑/↓ seguradas movem o texto (≈ 3 linhas por segundo);
        // com Shift, passam rápido.
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        let shift = ctx.input(|i| i.modifiers.shift);
        if cima {
            if shift {
                self.turbo_pedido = -1.0;
            } else {
                self.rolagem_em -= 4.0 * dt;
            }
            ctx.request_repaint();
        }
        if baixo {
            if shift {
                self.turbo_pedido = 1.0;
            } else {
                self.rolagem_em += 4.0 * dt;
            }
            ctx.request_repaint();
        }
        let passo = Self::passo_velocidade(ctx);
        if dir {
            self.mudar_velocidade(passo);
        }
        if esq {
            self.mudar_velocidade(-passo);
        }
        if fim {
            self.rodando = false;
        }
        if pgup {
            self.bloco_relativo(-1);
        }
        if pgdn {
            self.bloco_relativo(1);
        }
        if home {
            self.ir_inicio();
        }
        if f5 {
            self.crono.alternar();
        }
    }

    pub(super) fn avancar(&mut self, ctx: &egui::Context) {
        // Tempo real desde o último quadro: a rolagem anda na mesma velocidade
        // mesmo quando o computador atrasa um quadro (PC mais simples).
        let agora = Instant::now();
        let dt = self
            .ultimo_avanco
            .map_or(0.0, |t| agora.duration_since(t).as_secs_f32())
            .min(0.1);
        self.ultimo_avanco = Some(agora);
        if self.turbo != 0.0 {
            self.rolagem_em += self.turbo * VELOCIDADE_TURBO_EM * dt;
            if self.medidas.total_em > 0.0 {
                self.rolagem_em = self.rolagem_em.min(self.medidas.total_em);
            }
            ctx.request_repaint();
        }
        if self.rodando {
            self.rolagem_em += self.em_por_segundo() * dt;
            if self.medidas.total_em > 0.0 && self.rolagem_em >= self.medidas.total_em {
                self.rolagem_em = self.medidas.total_em;
                self.rodando = false;
            }
            ctx.request_repaint();
        }
        self.rolagem_em = self.rolagem_em.max(0.0);
        if self.crono.rodando() || self.crono_entrevista.rodando() {
            ctx.request_repaint_after(Duration::from_millis(250));
        } else {
            // Relógio de parede na tela.
            ctx.request_repaint_after(Duration::from_secs(1));
        }
    }
}
