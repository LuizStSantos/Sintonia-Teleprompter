//! Formato de arquivo TXT: separador de laudas `//` (Apotec) e decodificação
//! de arquivos UTF-8 ou Windows-1252/Latin-1.

fn eh_separador(c: &str, i: usize) -> bool {
    c[i..].starts_with("//") && !c[..i].ends_with(':')
}

pub(crate) fn tem_separador_lauda(c: &str) -> bool {
    c.match_indices("//").any(|(i, _)| eh_separador(c, i))
}

/// Troca cada "//" de lauda por uma linha "#" (bloco novo sem título).
pub(crate) fn trocar_separador_lauda(c: &str) -> String {
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
