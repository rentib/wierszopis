/// Moduł zaawansowanej analizy fonetycznej i sylabizacji języka polskiego (inspirowany internal/slowo).

pub fn czy_samogloska(c: char) -> bool {
    matches!(c.to_ascii_lowercase(), 'a' | 'ą' | 'e' | 'ę' | 'i' | 'o' | 'ó' | 'u' | 'y')
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

pub fn rozwin_litere(c: char) -> &'static str {
    match c.to_ascii_lowercase() {
        'a' => "a",
        'ą' => "ą",
        'b' => "be",
        'c' => "ce",
        'ć' => "će",
        'd' => "de",
        'e' => "e",
        'ę' => "ę",
        'f' => "ef",
        'g' => "gie",
        'h' => "ha",
        'i' => "i",
        'j' => "jot",
        'k' => "ka",
        'l' => "el",
        'ł' => "eł",
        'm' => "em",
        'n' => "en",
        'ń' => "eń",
        'o' => "o",
        'ó' => "ó",
        'p' => "pe",
        'q' => "ku",
        'r' => "er",
        's' => "es",
        'ś' => "eś",
        't' => "te",
        'u' => "u",
        'v' => "fał",
        'w' => "wu",
        'x' => "iks",
        'y' => "y",
        'ź' => "źe",
        'z' => "zet",
        'ż' => "żet",
        _   => "",
    }
}

pub fn czy_skrotowiec(tekst: &str) -> bool {
    let czyste = tekst.trim();
    let char_count = czyste.chars().count();
    
    if char_count < 2 || char_count > 4 {
        return false;
    }

    let has_alpha = czyste.chars().any(|c| c.is_alphabetic());
    if !has_alpha {
        return false;
    }

    let all_upper = czyste.chars().all(|c| !c.is_alphabetic() || c.is_uppercase());
    let has_vowel = czyste.chars().any(czy_samogloska);

    all_upper || !has_vowel
}

pub fn rozwiniety_skrotowiec(tekst: &str) -> String {
    tekst.trim().chars()
        .map(rozwin_litere)
        .collect()
}

/// Przetwarza słowo na ciąg głosek uwzględniając zmiękczenia (i) oraz dwugłoski (u).
pub fn przetworz_na_gloski(tekst: &str) -> Vec<String> {
    let czyste = tekst.trim();
    let zpracowany = if czy_skrotowiec(czyste) {
        rozwiniety_skrotowiec(czyste)
    } else {
        czyste.to_lowercase()
    };
    let runes: Vec<char> = zpracowany.chars().collect();
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

        if r == 'u' && i > 0 && !czy_jest_u_wyjątkiem(&zpracowany, i) {
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

pub fn wyznacz_akcent(liczba_sylab: usize) -> String {
    wyznacz_akcent_dla_slowa("", liczba_sylab)
}

pub fn wyznacz_akcent_dla_slowa(slowo: &str, liczba_sylab: usize) -> String {
    if liczba_sylab == 0 {
        return String::new();
    }
    if liczba_sylab == 1 {
        return "S".to_string();
    }

    let s_lower = slowo.to_lowercase();
    let mut akcentowana_idx = liczba_sylab.saturating_sub(2); // default paroxytone

    // 1. Check Oxytone (last syllable)
    let is_abbr = czy_skrotowiec(slowo);
    let has_prefix_oxy = (s_lower.starts_with("arcy") || s_lower.starts_with("eks") || s_lower.starts_with("wice")) && liczba_sylab == 2;

    if is_abbr || has_prefix_oxy || s_lower == "pafawag" {
        akcentowana_idx = liczba_sylab - 1;
    } else {
        // 2. Check 4th syllable from end
        let ends_4th = [
            "alibyśmy", "elibyśmy", "ylibyśmy",
            "alibyście", "elibyście", "ylibyście",
        ];
        if liczba_sylab >= 4 && ends_4th.iter().any(|&s| s_lower.ends_with(s)) {
            akcentowana_idx = liczba_sylab - 4;
        } else {
            // 3. Check 3rd syllable from end (proparoxytone)
            let ends_3rd = [
                "liśmy", "liście", "łyśmy", "łyście",
                "łabym", "łabyś", "łaby", "liby", "łyby", "libyśmy", "łybyście",
                "kroć", "sta", "set",
                "abyśmy", "żebyście", "jeżeliby", "iśliby", "ponieważby",
            ];
            let proparoxytone_words = [
                "komitet", "uniwersytet", "maksimum", "minimum", "prezydent",
            ];

            let matches_3rd_suffix = ends_3rd.iter().any(|&s| s_lower.ends_with(s));
            let matches_special_word = proparoxytone_words.contains(&s_lower.as_str());
            let ends_ika_yka = (s_lower.ends_with("ika") || s_lower.ends_with("yka"))
                && !s_lower.ends_with("ikami") && !s_lower.ends_with("ykami") && !s_lower.ends_with("ikach") && !s_lower.ends_with("ykach");

            if liczba_sylab >= 3 && (matches_3rd_suffix || matches_special_word || ends_ika_yka) {
                akcentowana_idx = liczba_sylab - 3;
            }
        }
    }

    let mut akcent = vec!['s'; liczba_sylab];
    if akcentowana_idx < liczba_sylab {
        akcent[akcentowana_idx] = 'S';
    }

    // Add secondary accent (P) on the first syllable for words with 4 or more syllables
    if liczba_sylab >= 4 && akcentowana_idx > 0 {
        akcent[0] = 'P';
    }

    akcent.iter().collect()
}

pub struct AnalizaSlowa {
    pub slowo: String,
    pub liczba_sylab: usize,
    pub samogloski: Vec<char>,
    pub gloski: Vec<String>,
    pub sylaby: Vec<Vec<String>>,
    pub akcent: String,
    pub rym_meski: Vec<String>,
    pub rym_zenski: Vec<String>,
}

pub fn analizuj_slowo(slowo: &str) -> AnalizaSlowa {
    let gloski = przetworz_na_gloski(slowo);
    let sylaby = podziel_na_sylaby(&gloski);
    let samogloski: Vec<char> = slowo.to_lowercase().chars().filter(|&c| czy_samogloska(c)).collect();
    let liczba_sylab = sylaby.len();
    let akcent = wyznacz_akcent_dla_slowa(slowo, liczba_sylab);

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
        akcent,
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

    #[test]
    fn test_wyznacz_akcent() {
        assert_eq!(wyznacz_akcent(0), "");
        assert_eq!(wyznacz_akcent(1), "S");
        assert_eq!(wyznacz_akcent(2), "Ss");
        assert_eq!(wyznacz_akcent(3), "sSs");
        assert_eq!(wyznacz_akcent(4), "PsSs");
    }

    #[test]
    fn test_wyznacz_akcent_dla_slowa() {
        assert_eq!(analizuj_slowo("ONZ").akcent, "ssS");
        assert_eq!(analizuj_slowo("wicekról").akcent, "sSs");
        assert_eq!(analizuj_slowo("chodziliśmy").akcent, "PSss");
        assert_eq!(analizuj_slowo("matematyka").akcent, "PsSss");
    }

    #[test]
    fn test_skrotowce() {
        assert_eq!(zlicz_sylaby("bmw"), 3);
        assert_eq!(zlicz_sylaby("BMW"), 3);
        assert_eq!(analizuj_slowo("BMW").akcent, "ssS");
        assert_eq!(analizuj_slowo("bmw").akcent, "ssS");
    }
}
