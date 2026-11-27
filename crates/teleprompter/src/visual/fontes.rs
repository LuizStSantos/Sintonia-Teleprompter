//! Fontes: lista as fontes instaladas no sistema (Windows, macOS e Linux) e
//! carrega no egui só a que o operador escolher.

use crate::visual::tema;
use egui::{FontData, FontDefinitions, FontFamily};
use std::collections::HashMap;
use std::sync::Arc;

pub struct Fontes {
    db: fontdb::Database,
    pub familias: Vec<String>,
    /// nome da família (+ negrito) -> já carregada no egui
    carregadas: HashMap<String, Option<(FontFamily, u64)>>,
    /// Definições completas enviadas ao egui (padrão + fontes carregadas).
    definicoes: FontDefinitions,
    /// Contador de quadros da janela principal (avançado pelo app a cada quadro).
    pub quadro: u64,
}

impl Fontes {
    pub fn carregar_sistema() -> Self {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        let mut familias: Vec<String> = db
            .faces()
            .filter_map(|f| f.families.first().map(|(n, _)| n.clone()))
            .collect();
        familias.sort_by_key(|f| f.to_lowercase());
        familias.dedup();
        // As fontes do Sintonia (embutidas) aparecem primeiro na lista.
        for (i, f) in tema::FONTES_EMBUTIDAS.iter().enumerate() {
            familias.insert(i, f.to_string());
        }
        Self {
            db,
            familias,
            carregadas: HashMap::new(),
            definicoes: tema::fontes(),
            quadro: 0,
        }
    }

    /// Fonte usada quando o operador deixa "Automática": a do Sintonia.
    pub fn automatica(&self) -> Option<&str> {
        tema::FONTES_EMBUTIDAS.first().copied()
    }

    /// Definições iniciais (fontes da interface do Sintonia).
    pub fn definicoes(&self) -> FontDefinitions {
        self.definicoes.clone()
    }

    /// Garante que a família está carregada no egui e devolve o `FontFamily` para usar.
    /// Se não achar a fonte, cai na fonte padrão do egui.
    pub fn familia(&mut self, ctx: &egui::Context, nome: &str, negrito: bool) -> FontFamily {
        let nome = if nome.is_empty() {
            self.automatica().unwrap_or("").to_string()
        } else {
            nome.to_string()
        };
        if nome.is_empty() {
            return FontFamily::Proportional;
        }
        let chave = format!("{nome}{}", if negrito { " [negrito]" } else { "" });
        let passo = self.quadro;
        if !self.carregadas.contains_key(&chave) {
            let resultado = self
                .carregar(ctx, &nome, &chave, negrito)
                .map(|f| (f, passo));
            self.carregadas.insert(chave.clone(), resultado);
        }
        match &self.carregadas[&chave] {
            // A fonte nova só passa a existir no egui a partir do próximo quadro;
            // usar o nome antes disso faria o egui entrar em pânico.
            Some((f, carregada_em)) if passo > *carregada_em => f.clone(),
            Some(_) => {
                ctx.request_repaint();
                FontFamily::Proportional
            }
            None => FontFamily::Proportional,
        }
    }

    fn carregar(
        &mut self,
        ctx: &egui::Context,
        nome: &str,
        chave: &str,
        negrito: bool,
    ) -> Option<FontFamily> {
        if let Some(dados) = tema::fonte_embutida(nome, negrito) {
            return Some(self.registrar(ctx, chave, dados));
        }
        let familias = [fontdb::Family::Name(nome)];
        let consulta = fontdb::Query {
            families: &familias,
            weight: if negrito {
                fontdb::Weight::BOLD
            } else {
                fontdb::Weight::NORMAL
            },
            stretch: fontdb::Stretch::Normal,
            style: fontdb::Style::Normal,
        };
        let id = self.db.query(&consulta)?;
        let (bytes, indice) = self
            .db
            .with_face_data(id, |dados, indice| (dados.to_vec(), indice))?;
        Some(self.registrar(ctx, chave, FontData::from_blob(Arc::new(bytes), indice)))
    }

    fn registrar(&mut self, ctx: &egui::Context, chave: &str, dados: FontData) -> FontFamily {
        let familia = FontFamily::Name(chave.into());
        self.definicoes
            .font_data
            .insert(chave.to_string(), Arc::new(dados));
        // A fonte escolhida primeiro; as padrão do egui de reserva (símbolos, emoji...).
        let mut lista = vec![chave.to_string()];
        lista.extend(
            self.definicoes
                .families
                .get(&FontFamily::Proportional)
                .cloned()
                .unwrap_or_default(),
        );
        self.definicoes.families.insert(familia.clone(), lista);
        ctx.set_fonts(self.definicoes.clone());
        familia
    }
}
