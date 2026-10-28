/// Odpowiednik internal/slowo w języku Rust.
use crate::rym;

#[derive(Clone, Debug)]
pub struct Słowo {
    pub oryginalne: String,
    pub głoski: Vec<String>,
    pub sylaby: Vec<Vec<String>>,
    pub rym_meski: Vec<String>,
    pub rym_zenski: Vec<String>,
}

pub fn czy_samogłoska(r: char) -> bool {
    matches!(r, 'a' | 'ą' | 'e' | 'ę' | 'i' | 'o' | 'ó' | 'u' | 'y')
}

pub fn czy_spółgłoska(r: char) -> bool {
    !czy_samogłoska(r)
}

const WYJĄTKI_U: &[&str] = &[
    "auw", "deus", "leus", "feus", "teus", "zauf", "pouf",
    "nauk", "naucz", "nauce", "wyucz", "zausz", "tryumf", "nieu",
];

fn czy_jest_u_wyjątkiem(czyste: &str, u_index: usize) -> bool {
    for w in WYJĄTKI_U {
        let mut start = 0;
        while let Some(idx) = czyste[start..].find(w) {
            let abs_idx = start + idx;
            if let Some(u_offset) = w.find('u') {
                if abs_idx + u_offset == u_index {
                    return true;
                }
            }
            start = abs_idx + 1;
        }
    }
    false
}

pub fn nowe_słowo(tekst: &str) -> Słowo {
    let czyste = tekst.trim().to_lowercase();
    let runes: Vec<char> = czyste.chars().collect();
    let n = runes.len();
    let mut przetworzone: Vec<String> = Vec::new();

    for i in 0..n {
        let r = runes[i];

        if i == 0 && r == 'i' && i + 1 < n && czy_samogłoska(runes[i + 1]) {
            przetworzone.push("ʲ".to_string());
            continue;
        }

        if r == 'i' && i > 0 && i + 1 < n && czy_spółgłoska(runes[i - 1]) && czy_samogłoska(runes[i + 1]) {
            if !przetworzone.is_empty() {
                let poprzednia = przetworzone[przetworzone.len() - 1].clone();
                let idx = przetworzone.len() - 1;
                if poprzednia == "s" || poprzednia == "c" || poprzednia == "z" || poprzednia == "n" ||
                    poprzednia == "sz" || poprzednia == "cz" || poprzednia == "dz" || poprzednia == "rz" {
                    przetworzone[idx] = format!("{}ⁱ", poprzednia);
                } else {
                    przetworzone[idx] = format!("{}ʲ", poprzednia);
                }
            }
            continue;
        }

        if r == 'u' && i > 0 && !czy_jest_u_wyjątkiem(&czyste, i) {
            let poprzedni_znaku = runes[i - 1];
            if czy_samogłoska(poprzedni_znaku) && poprzedni_znaku != 'i' {
                przetworzone.push("ᵘ".to_string());
                continue;
            }
        }

        przetworzone.push(r.to_string());
    }

    let głoski = rozbij_na_głoski_z_przetworzonych(&przetworzone);
    let sylaby = podziel_na_sylaby(&głoski);

    let mut indeksy_samogłosek = Vec::new();
    for (i, g) in głoski.iter().enumerate() {
        if czy_głoska_samogłoska(g) {
            indeksy_samogłosek.push(i);
        }
    }

    let mut rym_meski = Vec::new();
    let mut rym_zenski = Vec::new();
    if !indeksy_samogłosek.is_empty() {
        rym_meski = głoski[indeksy_samogłosek[indeksy_samogłosek.len() - 1]..].to_vec();
    }
    if indeksy_samogłosek.len() >= 2 {
        rym_zenski = głoski[indeksy_samogłosek[indeksy_samogłosek.len() - 2]..].to_vec();
    }

    Słowo {
        oryginalne: tekst.to_string(),
        głoski,
        sylaby,
        rym_meski,
        rym_zenski,
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
}

fn rozbij_na_głoski_z_przetworzonych(elementy: &[String]) -> Vec<String> {
    let mut głoski = Vec::new();
    let n = elementy.len();
    let mut i = 0;

    while i < n {
        if i + 1 < n {
            let para = format!("{}{}", elementy[i], elementy[i + 1]);
            if para == "ch" || para == "cz" || para == "dz" || para == "dź" || para == "dż" || para == "rz" || para == "sz" {
                głoski.push(para);
                i += 2;
                continue;
            }
        }
        głoski.push(elementy[i].clone());
        i += 1;
    }

    głoski
}

fn czy_głoska_samogłoska(głoska: &str) -> bool {
    rym::czy_samogloska(głoska)
}

fn podziel_na_sylaby(gloski: &[String]) -> Vec<Vec<String>> {
    if gloski.is_empty() {
        return vec![];
    }

    let mut indeksy_samogłosek = Vec::new();
    for (i, g) in gloski.iter().enumerate() {
        if czy_głoska_samogłoska(g) {
            indeksy_samogłosek.push(i);
        }
    }

    if indeksy_samogłosek.is_empty() {
        return vec![];
    }

    if indeksy_samogłosek.len() == 1 {
        return vec![gloski.to_vec()];
    }

    let mut sylaby = Vec::new();
    let mut aktualny_start = 0;

    for s_idx in 0..indeksy_samogłosek.len() - 1 {
        let v1 = indeksy_samogłosek[s_idx];
        let v2 = indeksy_samogłosek[s_idx + 1];

        let liczba_spółgłosek = v2 - v1 - 1;
        let punkt_podziału = v1 + 1 + (liczba_spółgłosek / 2);

        sylaby.push(gloski[aktualny_start..punkt_podziału].to_vec());
        aktualny_start = punkt_podziału;
    }

    sylaby.push(gloski[aktualny_start..].to_vec());
    sylaby
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
