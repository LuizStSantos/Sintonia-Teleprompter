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
    /// Velocidade de rolagem (0 a 10).
    pub velocidade: f32,
    /// Palavras por minuto usadas para estimar o tempo de leitura.
    pub palavras_por_minuto: u32,
    /// O cronômetro começa sozinho quando o texto rola e zera no Home.
    pub crono_automatico: bool,

    /// Tema claro da tela do operador (igual à opção do Sintonia).
    pub tema_claro: bool,

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
            velocidade: 3.0,
            palavras_por_minuto: 150,
            crono_automatico: true,
            tema_claro: false,
            mostrar_relogio_apresentador: true,
            monitor_apresentador: 1,
            tela_cheia_apresentador: false,
        }
    }
}

fn config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("br", "Sintonia", "SintoniaTeleprompter")
        .map(|d| d.config_dir().join("config.json"))
}

impl Settings {
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
