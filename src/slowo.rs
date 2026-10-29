/// Odpowiednik internal/slowo w języku Rust.
use crate::rym;
use crate::fonetyka::czy_głoska_samogłoska;

#[derive(Clone, Debug)]
pub struct Słowo {
    pub oryginalne: String,
    pub głoski: Vec<String>,
    pub sylaby: Vec<Vec<String>>,
    pub akcent: String,
    pub rym_meski: Vec<String>,
    pub rym_zenski: Vec<String>,
}

pub fn czy_samogłoska(r: char) -> bool {
    matches!(r.to_ascii_lowercase(), 'a' | 'ą' | 'e' | 'ę' | 'i' | 'o' | 'ó' | 'u' | 'y')
}

pub fn czy_spółgłoska(r: char) -> bool {
    !czy_samogłoska(r)
}

pub fn nowe_słowo(tekst: &str) -> Słowo {
    let analiza = crate::fonetyka::analizuj_slowo(tekst);
    Słowo {
        oryginalne: tekst.to_string(),
        głoski: analiza.gloski,
        sylaby: analiza.sylaby,
        akcent: analiza.akcent,
        rym_meski: analiza.rym_meski,
        rym_zenski: analiza.rym_zenski,
    }
}

impl Słowo {
    pub fn klucz_bucketa(&self) -> String {
        let samogłoski: Vec<&String> = self.głoski.iter().filter(|g| czy_głoska_samogłoska(g)).collect();
        let start = samogłoski.len().saturating_sub(2);
        let wybrane = &samogłoski[start..];

        let mut klucz = String::new();
        for g in wybrane {
            let bazowa = g.trim_end_matches(|c| c == 'ⁱ' || c == 'ʲ');
            let normalizowana = match bazowa {
                "ó" => "u",
                "ą" => "o",
                "ę" => "e",
                "y" => "i",
                other => other,
            };
            klucz.push_str(normalizowana);
        }
        klucz
    }

    pub fn klucz_bucketa_id(&self) -> usize {
        let samogłoski: Vec<&String> = self.głoski.iter().filter(|g| czy_głoska_samogłoska(g)).collect();
        let start = samogłoski.len().saturating_sub(2);
        let wybrane = &samogłoski[start..];

        let v_code = |c: &str| -> usize {
            let bazowa = c.trim_end_matches(|ch| ch == 'ⁱ' || ch == 'ʲ');
            match bazowa {
                "a" => 1,
                "e" | "ę" => 2,
                "i" | "y" => 3,
                "o" | "ą" => 4,
                "u" | "ó" => 5,
                _ => 6,
            }
        };

        if wybrane.is_empty() {
            0
        } else if wybrane.len() == 1 {
            v_code(wybrane[0])
        } else {
            v_code(wybrane[0]) * 10 + v_code(wybrane[1])
        }
    }

    pub fn sylaba_spans(&self) -> Vec<(usize, usize)> {
        let text = &self.oryginalne;
        let chars: Vec<(usize, char)> = text.char_indices().filter(|(_, c)| c.is_alphabetic() || "ąęśćżźńłóĄĘŚĆŻŹŃŁÓ".contains(*c)).collect();
        if chars.is_empty() {
            return vec![(0, text.len())];
        }

        let mut spans = Vec::new();
        let mut char_idx = 0;

        for syl in &self.sylaby {
            let syl_chars_len: usize = syl.iter().map(|p| {
                if p.ends_with('ⁱ') || p.ends_with('ʲ') {
                    let base = p.trim_end_matches(|c| c == 'ⁱ' || c == 'ʲ');
                    base.chars().count() + 1
                } else {
                    match p.as_str() {
                        "sz" | "cz" | "rz" | "ch" | "dz" | "dź" | "dż" => 2,
                        "ʲ" | "ⁱ" | "ᵘ" => 0,
                        other => other.chars().count(),
                    }
                }
            }).sum();

            let start_char = char_idx;
            let end_char = (char_idx + syl_chars_len).min(chars.len());
            
            if start_char < chars.len() {
                let start_byte = chars[start_char].0;
                let end_byte = if end_char < chars.len() {
                    chars[end_char].0
                } else {
                    text.len()
                };
                spans.push((start_byte, end_byte));
            } else {
                spans.push((text.len(), text.len()));
            }
            char_idx = end_char;
        }
        spans
    }
}

pub fn czy_rymują(s1: &Słowo, s2: &Słowo) -> (bool, i32, i32) {
    let num_vowels = |s: &Słowo| -> usize {
        s.głoski.iter().filter(|g| czy_głoska_samogłoska(g)).count()
    };

    let n1 = num_vowels(s1);
    let n2 = num_vowels(s2);

    if n1 == 1 && n2 == 1 {
        if !s1.rym_meski.is_empty() && !s2.rym_meski.is_empty() {
            return rym::porównaj_rymy(&s1.rym_meski, &s2.rym_meski);
        }
        return (false, 999, 0);
    }

    if n1 >= 2 && n2 >= 2 {
        if !s1.rym_zenski.is_empty() && !s2.rym_zenski.is_empty() {
            return rym::porównaj_rymy(&s1.rym_zenski, &s2.rym_zenski);
        }
        return (false, 999, 0);
    }

    (false, 999, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mama_mamama_bucket() {
        let s1 = nowe_słowo("mama");
        let s2 = nowe_słowo("mamama");
        println!("mama bucket: {}, mamama bucket: {}", s1.klucz_bucketa(), s2.klucz_bucketa());
        assert_eq!(s1.klucz_bucketa(), s2.klucz_bucketa());
        assert_eq!(s1.rym_zenski, s2.rym_zenski);
        let (ok1, _, _) = czy_rymują(&s1, &s2);
        assert!(ok1);
    }

    #[test]
    fn test_tata_a_nierymuja() {
        let s1 = nowe_słowo("tata");
        let s2 = nowe_słowo("a");
        let (ok, _, _) = czy_rymują(&s1, &s2);
        assert!(!ok);
    }
}
