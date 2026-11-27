//! Roteiro do teleprompter: uma lista de blocos (segmentos) na ordem de leitura.
//!
//! Um roteiro pode vir de:
//! - um **Espelho** do Sintonia (cada item do espelho vira um bloco: matéria,
//!   entrevistado, item manual ou passagem de bloco/intervalo);
//! - um arquivo **TXT** (linhas começando com `#` viram títulos de bloco, e
//!   `//` separa laudas — o mesmo formato do teleprompter Apotec);
//! - ou ser digitado do zero na tela do operador.

use crate::timer::parse_duracao;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tipo {
    Materia,
    Entrevistado,
    Manual,
    Intervalo,
    Texto,
}

impl Tipo {
    pub fn rotulo(self) -> &'static str {
        match self {
            Tipo::Materia => "Matéria",
            Tipo::Entrevistado => "Entrevistado",
            Tipo::Manual => "Item manual",
            Tipo::Intervalo => "Intervalo",
            Tipo::Texto => "Texto",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Segmento {
    /// `id` do item no Espelho (quando veio do Sintonia) — usado para gravar o texto de volta.
    pub item_id: Option<String>,
    pub tipo: Tipo,
    pub titulo: String,
    pub texto: String,
    pub duracao: String,
    /// Texto como veio do Sintonia, para saber se o operador alterou.
    pub texto_original: String,
}

impl Segmento {
    pub fn novo_texto(titulo: &str, texto: &str) -> Self {
        Self {
            item_id: None,
            tipo: Tipo::Texto,
            titulo: titulo.into(),
            texto: texto.into(),
            duracao: String::new(),
            texto_original: texto.into(),
        }
    }

    pub fn alterado(&self) -> bool {
        self.texto != self.texto_original
    }

    pub fn palavras(&self) -> usize {
        self.texto
            .split_whitespace()
            .filter(|p| *p != "<<" && *p != ">>")
            .count()
    }
}

/// Divide o texto em trechos normais e trechos em **destaque** (entre `<<` e `>>`),
/// como no Apotec. Um marcador sozinho na linha some junto com a quebra de linha.
pub fn trechos_destaque(texto: &str) -> Vec<(String, bool)> {
    let mut out: Vec<(String, bool)> = Vec::new();
    let mut atual = String::new();
    let mut destaque = false;
    let mut resto = texto;
    while !resto.is_empty() {
        let prox = [resto.find("<<"), resto.find(">>")]
            .into_iter()
            .flatten()
            .min();
        let Some(i) = prox else {
            atual.push_str(resto);
            break;
        };
        atual.push_str(&resto[..i]);
        let abre = resto[i..].starts_with("<<");
        resto = &resto[i + 2..];
        // Marcador sozinho na linha: tira a quebra de linha que sobraria.
        if (atual.ends_with('\n') || atual.is_empty())
            && let Some(r) = resto
                .strip_prefix("\r\n")
                .or_else(|| resto.strip_prefix('\n'))
        {
            resto = r;
        }
        if abre != destaque {
            if !atual.is_empty() {
                out.push((std::mem::take(&mut atual), destaque));
            }
            destaque = abre;
        }
    }
    if !atual.is_empty() {
        out.push((atual, destaque));
    }
    out
}

#[derive(Clone, Debug, Default)]
pub enum Origem {
    #[default]
    Nenhuma,
    Arquivo(PathBuf),
    Espelho {
        id: String,
        titulo: String,
    },
}

#[derive(Clone, Debug, Default)]
pub struct Roteiro {
    pub origem: Origem,
    pub titulo: String,
    pub segmentos: Vec<Segmento>,
}

fn s(v: &Value, campo: &str) -> String {
    match v.get(campo) {
        Some(Value::String(x)) => x.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

/// Resumo de um espelho para a lista de escolha.
#[derive(Clone, Debug)]
pub struct EspelhoResumo {
    pub id: String,
    pub titulo: String,
    pub data: String,
    pub programa: String,
    pub horario: String,
    pub itens: usize,
}

impl EspelhoResumo {
    pub fn de(v: &Value) -> Self {
        let programa = {
            let sigla = s(v, "programaSigla");
            let nome = s(v, "programaNome");
            if !sigla.is_empty() && !nome.is_empty() {
                format!("{nome} ({sigla})")
            } else if nome.is_empty() {
                sigla
            } else {
                nome
            }
        };
        Self {
            id: s(v, "id"),
            titulo: s(v, "titulo"),
            data: s(v, "data"),
            programa,
            horario: s(v, "horarioPrograma"),
            itens: v.get("itens").and_then(Value::as_array).map_or(0, Vec::len),
        }
    }

    /// Data no formato brasileiro (dd/mm/aaaa).
    pub fn data_br(&self) -> String {
        data_br(&self.data)
    }
}

pub fn data_br(iso: &str) -> String {
    let p: Vec<&str> = iso.split('-').collect();
    if p.len() == 3 {
        format!("{}/{}/{}", p[2], p[1], p[0])
    } else {
        iso.to_string()
    }
}

impl Roteiro {
    /// Monta o roteiro a partir de um registro da coleção `espelho` do Sintonia.
    pub fn do_espelho(esp: &Value) -> Self {
        let resumo = EspelhoResumo::de(esp);
        let mut segmentos = Vec::new();
        let mut bloco = 1;
        for item in esp
            .get("itens")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let tipo = match s(item, "tipo").as_str() {
                "materia" => Tipo::Materia,
                "entrevistado" => Tipo::Entrevistado,
                "passagem" => Tipo::Intervalo,
                _ => Tipo::Manual,
            };
            let titulo = if tipo == Tipo::Intervalo {
                bloco += 1;
                format!("INTERVALO — volta no bloco {bloco}")
            } else {
                s(item, "retranca")
            };
            let texto = s(item, "texto");
            segmentos.push(Segmento {
                item_id: Some(s(item, "id")).filter(|x| !x.is_empty()),
                tipo,
                titulo,
                texto_original: texto.clone(),
                texto,
                duracao: s(item, "duracao"),
            });
        }
        let titulo = if resumo.titulo.is_empty() {
            format!("Espelho {}", resumo.data_br())
        } else {
            format!("{} — {}", resumo.titulo, resumo.data_br())
        };
        Self {
            origem: Origem::Espelho {
                id: resumo.id,
                titulo: titulo.clone(),
            },
            titulo,
            segmentos,
        }
    }

    /// Lê um TXT. Linhas iniciadas por `#` abrem um bloco novo com aquele título;
    /// sem nenhum `#`, o arquivo inteiro vira um bloco só.
    pub fn do_txt(conteudo: &str, caminho: Option<PathBuf>) -> Self {
        let conteudo = conteudo.replace("\r\n", "\n").replace('\r', "\n");
        // "//" separa laudas (formato do Apotec). Não confunde com "http://".
        let usa_laudas = tem_separador_lauda(&conteudo);
        let conteudo = if usa_laudas {
            trocar_separador_lauda(&conteudo)
        } else {
            conteudo
        };
        let mut segmentos: Vec<Segmento> = Vec::new();
        let mut atual: Option<Segmento> = None;
        for linha in conteudo.lines() {
            if let Some(titulo) = linha.trim_start().strip_prefix('#') {
                if let Some(seg) = atual.take() {
                    segmentos.push(seg);
                }
                atual = Some(Segmento::novo_texto(
                    titulo.trim_start_matches('#').trim(),
                    "",
                ));
            } else {
                let seg = atual.get_or_insert_with(|| Segmento::novo_texto("", ""));
                if !seg.texto.is_empty() {
                    seg.texto.push('\n');
                }
                seg.texto.push_str(linha);
            }
        }
        if let Some(seg) = atual {
            segmentos.push(seg);
        }
        for seg in &mut segmentos {
            seg.texto = seg
                .texto
                .trim_matches(|c: char| c == '\n' || c == ' ')
                .to_string();
            seg.texto_original = seg.texto.clone();
        }
        if usa_laudas {
            segmentos.retain(|s| !(s.titulo.is_empty() && s.texto.is_empty()));
            for (i, seg) in segmentos.iter_mut().enumerate() {
                if seg.titulo.is_empty() {
                    seg.titulo = format!("LAUDA {}", i + 1);
                }
            }
        }
        let titulo = caminho
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Roteiro".into());
        Self {
            origem: caminho.map(Origem::Arquivo).unwrap_or_default(),
            titulo,
            segmentos,
        }
    }

    /// Gera o TXT no mesmo formato aceito por [`Roteiro::do_txt`].
    pub fn para_txt(&self) -> String {
        let mut out = String::new();
        for seg in &self.segmentos {
            if !seg.titulo.is_empty() || self.segmentos.len() > 1 {
                out.push_str("# ");
                out.push_str(&seg.titulo);
                out.push('\n');
            }
            out.push_str(&seg.texto);
            out.push_str("\n\n");
        }
        out
    }

    pub fn palavras(&self) -> usize {
        self.segmentos.iter().map(Segmento::palavras).sum()
    }

    /// Soma das durações informadas no espelho (matérias + intervalos), em segundos.
    pub fn duracao_total(&self) -> i64 {
        self.segmentos
            .iter()
            .filter_map(|s| parse_duracao(&s.duracao))
            .sum()
    }

    pub fn alterados(&self) -> usize {
        self.segmentos
            .iter()
            .filter(|s| s.alterado() && s.item_id.is_some())
            .count()
    }

    pub fn marcar_como_salvo(&mut self) {
        for seg in &mut self.segmentos {
            seg.texto_original = seg.texto.clone();
        }
    }
}

fn eh_separador(c: &str, i: usize) -> bool {
    c[i..].starts_with("//") && !c[..i].ends_with(':')
}

fn tem_separador_lauda(c: &str) -> bool {
    c.match_indices("//").any(|(i, _)| eh_separador(c, i))
}

/// Troca cada "//" de lauda por uma linha "#" (bloco novo sem título).
fn trocar_separador_lauda(c: &str) -> String {
    let mut out = String::with_capacity(c.len() + 16);
    let mut i = 0;
    while i < c.len() {
        if eh_separador(c, i) {
            out.push_str("\n#\n");
            i += 2;
        } else {
            let ch = c[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

/// Decodifica um TXT: UTF-8 (com ou sem BOM) ou, se não for, Windows-1252/Latin-1
/// (comum em arquivos salvos pelo Bloco de Notas antigo).
pub fn decodificar_txt(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    match std::str::from_utf8(bytes) {
        Ok(t) => t.to_string(),
        Err(_) => bytes.iter().map(|&b| cp1252(b)).collect(),
    }
}

fn cp1252(b: u8) -> char {
    const ALTOS: [char; 32] = [
        '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8d}', 'Ž',
        '\u{8f}', '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9d}',
        'ž', 'Ÿ',
    ];
    if (0x80..0xA0).contains(&b) {
        ALTOS[(b - 0x80) as usize]
    } else {
        b as char
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn espelho_vira_roteiro() {
        let esp = json!({
            "id": "e1", "titulo": "ESPELHO DO PROGRAMA: SUCESSO NO CAMPO", "data": "2026-11-27",
            "itens": [
                {"id": "a", "tipo": "materia", "retranca": "SAFRA", "texto": "Boa noite.", "duracao": "1:30"},
                {"id": "b", "tipo": "passagem", "duracao": "2:00"},
                {"id": "c", "tipo": "custom", "retranca": "ENCERRAMENTO", "texto": "Até amanhã."}
            ]
        });
        let r = Roteiro::do_espelho(&esp);
        assert_eq!(r.segmentos.len(), 3);
        assert_eq!(r.segmentos[0].tipo, Tipo::Materia);
        assert_eq!(r.segmentos[1].tipo, Tipo::Intervalo);
        assert_eq!(r.segmentos[2].titulo, "ENCERRAMENTO");
        assert_eq!(r.duracao_total(), 210);
        assert!(r.titulo.contains("27/11/2026"));
    }

    #[test]
    fn txt_ida_e_volta() {
        let r = Roteiro::do_txt("# ABERTURA\nOlá\n\n# FIM\nTchau", None);
        assert_eq!(r.segmentos.len(), 2);
        assert_eq!(r.segmentos[0].titulo, "ABERTURA");
        assert_eq!(r.segmentos[1].texto, "Tchau");
        let r2 = Roteiro::do_txt(&r.para_txt(), None);
        assert_eq!(r2.segmentos[0].texto, "Olá");
    }

    #[test]
    fn txt_sem_titulos() {
        let r = Roteiro::do_txt("linha 1\nlinha 2", None);
        assert_eq!(r.segmentos.len(), 1);
        assert_eq!(r.segmentos[0].texto, "linha 1\nlinha 2");
    }

    #[test]
    fn laudas_do_apotec() {
        let r = Roteiro::do_txt(
            "Texto da lauda 1// Texto da lauda 2 http://site.com// Lauda 3",
            None,
        );
        assert_eq!(r.segmentos.len(), 3);
        assert_eq!(r.segmentos[0].titulo, "LAUDA 1");
        assert_eq!(r.segmentos[1].texto, "Texto da lauda 2 http://site.com");
        assert_eq!(r.segmentos[2].titulo, "LAUDA 3");
    }

    #[test]
    fn destaque() {
        let t = trechos_destaque("normal\n<<\nem destaque\n>>\nfim");
        assert_eq!(
            t,
            vec![
                ("normal\n".into(), false),
                ("em destaque\n".into(), true),
                ("fim".into(), false)
            ]
        );
        let t = trechos_destaque("a <<b>> c");
        assert_eq!(
            t,
            vec![
                ("a ".into(), false),
                ("b".into(), true),
                (" c".into(), false)
            ]
        );
        assert_eq!(
            trechos_destaque("sem nada"),
            vec![("sem nada".into(), false)]
        );
    }

    #[test]
    fn latin1() {
        assert_eq!(decodificar_txt(&[0x63, 0xE7, 0xE3, 0x6F]), "cção");
        assert_eq!(decodificar_txt("ação".as_bytes()), "ação");
    }
}
