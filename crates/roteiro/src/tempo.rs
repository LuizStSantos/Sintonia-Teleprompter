//! Cronômetro do programa (com contagem regressiva opcional) e formatação de tempo.

use std::time::{Duration, Instant};

#[derive(Default)]
pub struct Cronometro {
    inicio: Option<Instant>,
    acumulado: Duration,
}

impl Cronometro {
    pub fn rodando(&self) -> bool {
        self.inicio.is_some()
    }

    pub fn iniciar(&mut self) {
        if self.inicio.is_none() {
            self.inicio = Some(Instant::now());
        }
    }

    pub fn pausar(&mut self) {
        if let Some(i) = self.inicio.take() {
            self.acumulado += i.elapsed();
        }
    }

    pub fn alternar(&mut self) {
        if self.rodando() {
            self.pausar()
        } else {
            self.iniciar()
        }
    }

    pub fn zerar(&mut self) {
        self.inicio = None;
        self.acumulado = Duration::ZERO;
    }

    pub fn decorrido(&self) -> Duration {
        self.acumulado + self.inicio.map(|i| i.elapsed()).unwrap_or_default()
    }
}

/// Formata segundos como `MM:SS` (ou `H:MM:SS`), com sinal de menos quando estourado.
pub fn fmt_tempo(segundos: i64) -> String {
    let sinal = if segundos < 0 { "-" } else { "" };
    let s = segundos.unsigned_abs();
    let (h, m, s) = (s / 3600, (s % 3600) / 60, s % 60);
    if h > 0 {
        format!("{sinal}{h}:{m:02}:{s:02}")
    } else {
        format!("{sinal}{m:02}:{s:02}")
    }
}

/// Converte uma duração digitada no Sintonia ("1:30", "01:02:03", "90", "1'30\"") em segundos.
pub fn parse_duracao(txt: &str) -> Option<i64> {
    let t = txt.trim();
    if t.is_empty() {
        return None;
    }
    let normalizado: String = t
        .chars()
        .map(|c| {
            if c == '\'' || c == '"' || c == 'm' || c == 'h' {
                ':'
            } else {
                c
            }
        })
        .filter(|c| c.is_ascii_digit() || *c == ':')
        .collect();
    let partes: Vec<&str> = normalizado.split(':').filter(|p| !p.is_empty()).collect();
    if partes.is_empty() {
        return None;
    }
    let mut total: i64 = 0;
    for p in &partes {
        total = total * 60 + p.parse::<i64>().ok()?;
    }
    Some(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formata() {
        assert_eq!(fmt_tempo(0), "00:00");
        assert_eq!(fmt_tempo(75), "01:15");
        assert_eq!(fmt_tempo(3725), "1:02:05");
        assert_eq!(fmt_tempo(-5), "-00:05");
    }

    #[test]
    fn interpreta() {
        assert_eq!(parse_duracao("1:30"), Some(90));
        assert_eq!(parse_duracao("01:02:03"), Some(3723));
        assert_eq!(parse_duracao("45"), Some(45));
        assert_eq!(parse_duracao("2'10\""), Some(130));
        assert_eq!(parse_duracao(""), None);
    }
}
