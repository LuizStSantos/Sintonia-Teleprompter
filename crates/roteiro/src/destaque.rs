//! Destaque de trechos com `<<` e `>>` (formato do teleprompter Apotec).

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
