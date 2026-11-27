//! Preferências do operador (aparência do texto, velocidade, servidor...).
//! Ficam num `config.json` na pasta de configuração do sistema operacional:
//! - Windows: `%APPDATA%\Sintonia\SintoniaTeleprompter\config\config.json`
//! - macOS:   `~/Library/Application Support/br.Sintonia.SintoniaTeleprompter/config.json`
//! - Linux:   `~/.config/sintoniateleprompter/config.json`
//!
//! A senha NUNCA é salva aqui — só o endereço do servidor e o usuário.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Alinhamento {
    Esquerda,
    Centro,
}

/// Como a tela é desenhada. O OpenGL é mais leve e roda bem em computadores
/// simples e placas de vídeo integradas; o "moderno" (wgpu: DirectX 12/Vulkan/
/// Metal) pode ser melhor em máquinas novas.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Renderizador {
    OpenGl,
    Moderno,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct Settings {
    // Conexão com o Sintonia
    pub server_url: String,
    pub username: String,

    // Formatação do texto
    /// Família de fonte do sistema. Vazio = escolha automática (Arial/Helvetica/DejaVu Sans...).
    pub font_family: String,
    pub negrito: bool,
    /// Tamanho da letra, em pixels, na tela do apresentador.
    pub font_size: f32,
    /// Espaçamento entre linhas (multiplicador do tamanho da letra).
    pub entrelinha: f32,
    /// Margem lateral, em % da largura da tela (cada lado).
    pub margem_pct: f32,
    pub alinhamento: Alinhamento,
    pub caixa_alta: bool,
    pub cor_texto: [u8; 3],
    pub cor_fundo: [u8; 3],
    pub cor_titulo: [u8; 3],
    /// Cor do texto entre `<<` e `>>` (destaque, como no Apotec).
    pub cor_destaque: [u8; 3],
    pub mostrar_titulos: bool,

    // Leitura
    pub espelhar_horizontal: bool,
    pub espelhar_vertical: bool,
    pub mostrar_marcador: bool,
    /// Altura da linha de leitura, em % da altura da tela.
    pub marcador_pct: f32,
    /// Velocidade de rolagem, de 0 a 100 %. (Campo novo: a escala antiga, de 0 a
    /// 10, ficava em `velocidade` e é ignorada.)
    pub velocidade_pct: f32,
    /// Palavras por minuto usadas para estimar o tempo de leitura.
    pub palavras_por_minuto: u32,
    /// O cronômetro começa sozinho quando o texto rola e zera no Home.
    pub crono_automatico: bool,
    /// Cronômetro da entrevista: `false` = só rodando (conta para cima);
    /// `true` = com tempo (regressivo a partir da duração).
    pub entrevista_com_tempo: bool,

    /// Barra de baixo recolhida (só o essencial numa linha).
    pub barra_recolhida: bool,
    /// Painel do roteiro (esquerda) recolhido.
    pub lateral_recolhida: bool,
    /// Renderizador gráfico (vale ao abrir o programa).
    pub renderizador: Renderizador,

    /// Tema claro da tela do operador (igual à opção do Sintonia).
    pub tema_claro: bool,

    /// Painel de Formatação aberto. Não é salvo: o programa sempre abre com ele
    /// escondido, e ele só aparece ao clicar em "Formatação" no menu.
    #[serde(skip)]
    pub mostrar_formatacao: bool,
    /// Guardar usuário e senha (a senha vai para o cofre do sistema, nunca para este arquivo).
    pub lembrar_login: bool,
    /// Com a senha lembrada, entra sozinho ao abrir o programa.
    pub entrar_automatico: bool,
    /// Predefinições salvas pelo operador (fonte, tamanho, cores, monitor, tela cheia...).
    pub predefinicoes: Vec<Predefinicao>,

    // Tela do apresentador
    pub mostrar_relogio_apresentador: bool,
    pub monitor_apresentador: usize,
    pub tela_cheia_apresentador: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:3001".into(),
            username: String::new(),
            font_family: String::new(),
            negrito: true,
            font_size: 72.0,
            entrelinha: 1.25,
            margem_pct: 8.0,
            alinhamento: Alinhamento::Esquerda,
            caixa_alta: true,
            cor_texto: [255, 255, 255],
            cor_fundo: [0, 0, 0],
            // Âmbar do Sintonia (--amber).
            cor_titulo: [232, 169, 58],
            cor_destaque: [255, 96, 80],
            mostrar_titulos: true,
            espelhar_horizontal: false,
            espelhar_vertical: false,
            mostrar_marcador: true,
            marcador_pct: 30.0,
            velocidade_pct: 15.0,
            palavras_por_minuto: 150,
            crono_automatico: true,
            entrevista_com_tempo: false,
            barra_recolhida: false,
            lateral_recolhida: false,
            renderizador: Renderizador::OpenGl,
            tema_claro: false,
            mostrar_formatacao: false,
            lembrar_login: true,
            entrar_automatico: false,
            predefinicoes: Vec::new(),
            mostrar_relogio_apresentador: false,
            monitor_apresentador: 1,
            tela_cheia_apresentador: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Predefinicao {
    pub nome: String,
    /// Os campos de [`Settings`] guardados nesta predefinição.
    pub valores: serde_json::Value,
}

/// Campos que NÃO entram numa predefinição (são da pessoa/conexão, não da aparência).
const FORA_DA_PREDEFINICAO: &[&str] = &[
    "server_url",
    "username",
    "predefinicoes",
    "lembrar_login",
    "entrar_automatico",
    "mostrar_formatacao",
    "tema_claro",
];

fn config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("br", "Sintonia", "SintoniaTeleprompter")
        .map(|d| d.config_dir().join("config.json"))
}

impl Settings {
    /// Fotografia da formatação atual, para guardar como predefinição.
    pub fn capturar(&self, nome: &str) -> Predefinicao {
        let mut v = serde_json::to_value(self).unwrap_or_default();
        if let Some(obj) = v.as_object_mut() {
            for k in FORA_DA_PREDEFINICAO {
                obj.remove(*k);
            }
        }
        Predefinicao {
            nome: nome.trim().to_string(),
            valores: v,
        }
    }

    /// Aplica uma predefinição por cima da configuração atual.
    pub fn aplicar(&mut self, pre: &Predefinicao) {
        let mut atual = serde_json::to_value(&*self).unwrap_or_default();
        if let (Some(dest), Some(orig)) = (atual.as_object_mut(), pre.valores.as_object()) {
            for (k, v) in orig {
                if !FORA_DA_PREDEFINICAO.contains(&k.as_str()) {
                    dest.insert(k.clone(), v.clone());
                }
            }
        }
        if let Ok(novo) = serde_json::from_value(atual) {
            *self = novo;
        }
    }

    /// Guarda (ou substitui, se o nome já existir) uma predefinição.
    pub fn guardar_predefinicao(&mut self, nome: &str) {
        let pre = self.capturar(nome);
        match self
            .predefinicoes
            .iter_mut()
            .find(|p| p.nome.eq_ignore_ascii_case(&pre.nome))
        {
            Some(existente) => *existente = pre,
            None => self.predefinicoes.push(pre),
        }
    }

    pub fn load() -> Self {
        config_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = config_path() else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predefinicao_ida_e_volta() {
        let mut cfg = Settings {
            font_size: 120.0,
            monitor_apresentador: 2,
            username: "ana".into(),
            ..Default::default()
        };
        cfg.guardar_predefinicao("Jornal");
        cfg.font_size = 50.0;
        cfg.monitor_apresentador = 0;
        cfg.username = "bia".into();
        let pre = cfg.predefinicoes[0].clone();
        cfg.aplicar(&pre);
        assert_eq!(cfg.font_size, 120.0);
        assert_eq!(cfg.monitor_apresentador, 2);
        assert_eq!(cfg.username, "bia", "usuário não faz parte da predefinição");
        cfg.guardar_predefinicao("jornal");
        assert_eq!(cfg.predefinicoes.len(), 1, "mesmo nome substitui");
    }
}
