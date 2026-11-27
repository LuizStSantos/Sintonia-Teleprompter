//! Abrir/salvar o roteiro: espelho, TXT, roteiro novo (com confirmação ao descartar).

use super::*;

impl App {
    pub(super) fn carregar_espelho(&mut self, esp: &Value) {
        self.roteiro = Roteiro::do_espelho(esp);
        self.depois_de_carregar();
        let total = self.roteiro.duracao_total();
        if total > 0 {
            self.meta_txt = fmt_tempo(total);
        }
    }

    pub(super) fn depois_de_carregar(&mut self) {
        self.editado = false;
        self.rolagem_em = 0.0;
        self.rodando = false;
        self.selecionado = if self.roteiro.segmentos.is_empty() {
            None
        } else {
            Some(0)
        };
        self.medidas = Medidas::default();
    }

    pub(super) fn executar(&mut self, acao: Pendente) {
        match acao {
            Pendente::AbrirEspelho(esp) => {
                self.carregar_espelho(&esp);
                self.seletor_aberto = false;
            }
            Pendente::AbrirTxt => self.abrir_txt(),
            Pendente::Novo => {
                self.roteiro = Roteiro {
                    titulo: "Roteiro novo".into(),
                    segmentos: vec![Segmento::novo_texto("", "")],
                    ..Default::default()
                };
                self.depois_de_carregar();
            }
        }
    }

    pub(super) fn pedir(&mut self, acao: Pendente) {
        if self.editado {
            self.pendente = Some(acao);
        } else {
            self.executar(acao);
        }
    }

    pub(super) fn abrir_txt(&mut self) {
        let Some(caminho) = rfd::FileDialog::new()
            .add_filter("Texto", &["txt", "TXT"])
            .pick_file()
        else {
            return;
        };
        match std::fs::read(&caminho) {
            Ok(bytes) => {
                self.roteiro = Roteiro::do_txt(&decodificar_txt(&bytes), Some(caminho));
                self.depois_de_carregar();
            }
            Err(e) => self.avisar(format!("Não foi possível abrir o arquivo: {e}"), true),
        }
    }

    pub(super) fn salvar_txt(&mut self) {
        let nome = format!(
            "{}.txt",
            self.roteiro
                .titulo
                .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-")
        );
        let Some(caminho) = rfd::FileDialog::new()
            .add_filter("Texto", &["txt"])
            .set_file_name(nome)
            .save_file()
        else {
            return;
        };
        match std::fs::write(&caminho, self.roteiro.para_txt()) {
            Ok(()) => {
                if matches!(self.roteiro.origem, Origem::Arquivo(_) | Origem::Nenhuma) {
                    self.editado = false;
                }
                self.avisar(format!("Salvo em {}", caminho.display()), false)
            }
            Err(e) => self.avisar(format!("Não foi possível salvar: {e}"), true),
        }
    }
}
