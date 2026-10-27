/// Moduł zaawansowanej analizy fonetycznej i sylabizacji języka polskiego (inspirowany internal/slowo).

pub fn czy_samogloska(c: char) -> bool {
    matches!(c, 'a' | 'ą' | 'e' | 'ę' | 'i' | 'o' | 'ó' | 'u' | 'y')
}

pub fn czy_spolgłoska(c: char) -> bool {
    !czy_samogloska(c)
}

fn czy_jest_u_wyjątkiem(czyste: &str, u_index: usize) -> bool {
    let wyjątki = [
        "auw", "deus", "leus", "feus", "teus", "zauf", "pouf",
        "nauk", "naucz", "nauce", "wyucz", "zausz", "tryumf", "nieu",
    ];
    for w in wyjątki {
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

/// Przetwarza słowo na ciąg głosek uwzględniając zmiękczenia (i) oraz dwugłoski (u).
pub fn przetworz_na_gloski(tekst: &str) -> Vec<String> {
    let czyste = tekst.trim().to_lowercase();
    let runes: Vec<char> = czyste.chars().collect();
    let n = runes.len();
    let mut przetworzone: Vec<String> = Vec::new();

    for i in 0..n {
        let r = runes[i];

        if i == 0 && r == 'i' && i + 1 < n && czy_samogloska(runes[i + 1]) {
            przetworzone.push("ʲ".to_string());
            continue;
        }

        if r == 'i' && i > 0 && i + 1 < n && czy_spolgłoska(runes[i - 1]) && czy_samogloska(runes[i + 1]) {
            if !przetworzone.is_empty() {
                let pop = przetworzone.pop().unwrap();
                if pop == "s" || pop == "c" || pop == "z" || pop == "n" ||
                    pop == "sz" || pop == "cz" || pop == "dz" || pop == "rz" {
                    przetworzone.push(format!("{}ⁱ", pop));
                } else {
                    przetworzone.push(format!("{}ʲ", pop));
                }
            }
            continue;
        }

        if r == 'u' && i > 0 && !czy_jest_u_wyjątkiem(&czyste, i) {
            let poprzedni = runes[i - 1];
            if czy_samogloska(poprzedni) && poprzedni != 'i' {
                przetworzone.push("ᵘ".to_string());
                continue;
            }
        }

        przetworzone.push(r.to_string());
    }

    rozbij_digrafy(&przetworzone)
}

fn rozbij_digrafy(elementy: &[String]) -> Vec<String> {
    let mut gloski = Vec::new();
    let n = elementy.len();
    let mut i = 0;

    while i < n {
        if i + 1 < n {
            let para = format!("{}{}", elementy[i], elementy[i + 1]);
            if para == "ch" || para == "cz" || para == "dz" || para == "dź" || para == "dż" || para == "rz" || para == "sz" {
                gloski.push(para);
                i += 2;
                continue;
            }
        }
        gloski.push(elementy[i].clone());
        i += 1;
    }

    gloski
}

pub fn czy_głoska_samogłoska(głoska: &str) -> bool {
    if głoska == "ᵘ" {
        return false;
    }
    let bazowa = głoska.trim_end_matches(|c| c == 'ⁱ' || c == 'ʲ');
    matches!(bazowa, "a" | "ą" | "e" | "ę" | "i" | "o" | "ó" | "u" | "y")
}

pub fn podziel_na_sylaby(gloski: &[String]) -> Vec<Vec<String>> {
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

pub fn zlicz_sylaby(slowo: &str) -> usize {
    let gloski = przetworz_na_gloski(slowo);
    podziel_na_sylaby(&gloski).len()
}

pub struct AnalizaSlowa {
    pub slowo: String,
    pub liczba_sylab: usize,
    pub samogloski: Vec<char>,
    pub gloski: Vec<String>,
    pub sylaby: Vec<Vec<String>>,
    pub rym_meski: Vec<String>,
    pub rym_zenski: Vec<String>,
}

pub fn analizuj_slowo(slowo: &str) -> AnalizaSlowa {
    let gloski = przetworz_na_gloski(slowo);
    let sylaby = podziel_na_sylaby(&gloski);
    let samogloski: Vec<char> = slowo.to_lowercase().chars().filter(|&c| czy_samogloska(c)).collect();
    let liczba_sylab = sylaby.len();

    let mut indeksy_samogłosek = Vec::new();
    for (i, g) in gloski.iter().enumerate() {
        if czy_głoska_samogłoska(g) {
            indeksy_samogłosek.push(i);
        }
    }

    let mut rym_meski = Vec::new();
    let mut rym_zenski = Vec::new();
    if !indeksy_samogłosek.is_empty() {
        rym_meski = gloski[indeksy_samogłosek[indeksy_samogłosek.len() - 1]..].to_vec();
    }
    if indeksy_samogłosek.len() >= 2 {
        rym_zenski = gloski[indeksy_samogłosek[indeksy_samogłosek.len() - 2]..].to_vec();
    }

    AnalizaSlowa {
        slowo: slowo.to_string(),
        liczba_sylab,
        samogloski,
        gloski,
        sylaby,
        rym_meski,
        rym_zenski,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zlicz_sylaby() {
        assert_eq!(zlicz_sylaby("kot"), 1);
        assert_eq!(zlicz_sylaby("dom"), 1);
        assert_eq!(zlicz_sylaby("wiersz"), 1);
        assert_eq!(zlicz_sylaby("mama"), 2);
        assert_eq!(zlicz_sylaby("poezja"), 3);
        assert_eq!(zlicz_sylaby("z"), 0);
    }
}
