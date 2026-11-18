/// Odpowiednik internal/wers w języku Rust.
use crate::rym;
use crate::slowo::{self, Słowo};

#[derive(Clone, Debug, PartialEq)]
pub struct Wers {
    pub surowy_tekst: String,
    pub oryginalne: String,
    pub słowa: Vec<Słowo>,
    pub liczba_sylab: usize,
    pub akcent_wersu: String,
    pub rym_meski: Vec<String>,
    pub rym_zenski: Vec<String>,
    pub średniówka: String,
}

pub fn wyznacz_akcent_wersu(słowa: &[Słowo]) -> String {
    let mut word_accents: Vec<String> = słowa.iter().map(|s| s.akcent.clone()).collect();

    let proclitics_mono = [
        "do", "na", "w", "z", "o", "u", "nad", "pod", "przed", "przez", "po", "ku", "bez", "dla",
        "od", "ze", "we", "nas", "i", "oraz", "prócz", "tych", "tej",
    ];
    let proclitics_di = ["beze", "pode", "nade", "przede", "przeze", "poza"];
    let enclitics = ["ci", "go", "ją", "mu", "was", "się", "mi"];

    let mut multi_run_indices = vec![false; słowa.len()];
    let mut idx_run = 0;
    while idx_run < słowa.len() {
        if word_accents[idx_run].len() == 1 {
            let start = idx_run;
            while idx_run < słowa.len() && word_accents[idx_run].len() == 1 {
                idx_run += 1;
            }
            let end = idx_run;
            if end - start >= 2 {
                for idx in start..end {
                    multi_run_indices[idx] = true;
                }
            }
            for (offset, idx) in (start..end).enumerate() {
                word_accents[idx] = if offset % 2 == 0 { "S".to_string() } else { "s".to_string() };
            }
        } else {
            idx_run += 1;
        }
    }

    for i in 0..słowa.len() {
        if multi_run_indices[i] {
            continue;
        }
        let w_lower = słowa[i].oryginalne.to_lowercase();

        if i > 0 && proclitics_di.contains(&w_lower.as_str()) && i + 1 < słowa.len() {
            let len = word_accents[i].len();
            if len > 0 {
                let mut acc = vec!['s'; len];
                acc[len - 1] = 'S';
                word_accents[i] = acc.into_iter().collect();
            }
            word_accents[i + 1] = "s".repeat(word_accents[i + 1].len());
        } else if w_lower == "nie" && i + 1 < słowa.len() {
            let next_len = word_accents[i + 1].len();
            if next_len == 1 {
                word_accents[i] = "S".to_string();
                word_accents[i + 1] = "s".to_string();
            } else {
                word_accents[i] = "s".repeat(word_accents[i].len());
            }
        } else if i > 0 && proclitics_mono.contains(&w_lower.as_str()) && i + 1 < słowa.len() {
            let next_len = word_accents[i + 1].len();
            let len = word_accents[i].len();
            if next_len == 1 {
                word_accents[i] = "S".repeat(len);
                word_accents[i + 1] = "s".to_string();
            } else {
                word_accents[i] = "s".repeat(len);
            }
        }

        if enclitics.contains(&w_lower.as_str()) {
            let len = word_accents[i].len();
            word_accents[i] = "s".repeat(len);
        }
    }

    word_accents.concat()
}

pub fn kandydaci_średniówki(słowa: &[Słowo], liczba_sylab: usize) -> Vec<(String, i32)> {
    if liczba_sylab < 9 || słowa.len() < 2 {
        return vec![];
    }

    let mut syl_counts = Vec::with_capacity(słowa.len());
    let mut sum = 0;
    for sw in słowa {
        sum += sw.sylaby.len();
        syl_counts.push(sum);
    }

    let akcent = wyznacz_akcent_wersu(słowa);
    let mut candidates = Vec::new();

    for k in 0..(słowa.len() - 1) {
        let x = syl_counts[k];
        let y = liczba_sylab - x;
        if x < 2 || y < 2 {
            continue;
        }

        let asymmetry = (x as i32 - y as i32).abs();
        let mut score = 100 - (asymmetry * 5);

        match (liczba_sylab, x, y) {
            (13, 7, 6) => score += 30,
            (13, 6, 7) => score += 24,
            (13, 8, 5) => score += 15,
            (13, 5, 8) => score += 10,
            (11, 5, 6) => score += 30,
            (11, 6, 5) => score += 24,
            (11, 4, 7) => score += 15,
            (11, 7, 4) => score += 10,
            (10, 5, 5) => score += 15,
            (10, 4, 6) | (10, 6, 4) => score += 20,
            (9, 5, 4) => score += 30,
            (9, 4, 5) => score += 24,
            _ => {}
        }

        if x > 0 && x <= akcent.len() && akcent.chars().nth(x - 1) == Some('S') {
            score -= 15;
        }
        if x >= 2 && x <= akcent.len() && akcent.chars().nth(x - 2) == Some('S') {
            score += 15;
        }

        candidates.push((format!("{}+{}", x, y), score));
    }

    candidates
}

pub fn wyznacz_średniówkę(słowa: &[Słowo], liczba_sylab: usize) -> String {
    kandydaci_średniówki(słowa, liczba_sylab)
        .into_iter()
        .max_by_key(|&(_, score)| score)
        .map(|(label, _)| label)
        .unwrap_or_else(|| "-".to_string())
}

pub fn nowy_wers(tekst: &str) -> Wers {
    let surowy = tekst.to_string();
    let czyste = tekst.trim().to_string();
    if czyste.is_empty() {
        return Wers {
            surowy_tekst: surowy,
            oryginalne: "".to_string(),
            słowa: vec![],
            liczba_sylab: 0,
            akcent_wersu: "".to_string(),
            rym_meski: vec![],
            rym_zenski: vec![],
            średniówka: "-".to_string(),
        };
    }

    let mut słowa = Vec::new();
    let mut całkowita_liczba_sylab = 0;
    let mut wszystkie_głoski = Vec::new();

    for sw in czyste.split_whitespace() {
        let czyste_słowo: String = sw
            .chars()
            .filter(|c| c.is_alphabetic() || "ąęśćżźńłóĄĘŚĆŻŹŃŁÓ".contains(*c))
            .collect();
        if czyste_słowo.is_empty() {
            continue;
        }
        let s = slowo::nowe_słowo(&czyste_słowo);
        całkowita_liczba_sylab += s.sylaby.len();
        wszystkie_głoski.extend(s.głoski.clone());
        słowa.push(s);
    }

    let indeksy_samogłosek: Vec<usize> = wszystkie_głoski
        .iter()
        .enumerate()
        .filter(|(_, g)| czy_głoska_samogłoska(g))
        .map(|(i, _)| i)
        .collect();

    let rym_meski = indeksy_samogłosek.last().map(|&o| wszystkie_głoski[o..].to_vec()).unwrap_or_default();
    let rym_zenski = if indeksy_samogłosek.len() >= 2 {
        let p = indeksy_samogłosek[indeksy_samogłosek.len() - 2];
        wszystkie_głoski[p..].to_vec()
    } else {
        vec![]
    };

    let akcent_wersu = wyznacz_akcent_wersu(&słowa);
    let średniówka = wyznacz_średniówkę(&słowa, całkowita_liczba_sylab);

    Wers {
        surowy_tekst: surowy,
        oryginalne: czyste,
        słowa,
        liczba_sylab: całkowita_liczba_sylab,
        akcent_wersu,
        rym_meski,
        rym_zenski,
        średniówka,
    }
}

pub fn czy_rymują(w1: &Wers, w2: &Wers) -> (bool, i32, i32) {
    if w1.słowa.is_empty() || w2.słowa.is_empty() {
        return (false, 999, 0);
    }

    let n1 = w1.słowa.last().unwrap().głoski.iter().filter(|g| czy_głoska_samogłoska(g)).count();
    let n2 = w2.słowa.last().unwrap().głoski.iter().filter(|g| czy_głoska_samogłoska(g)).count();

    if n1 == 1 && n2 == 1 && !w1.rym_meski.is_empty() && !w2.rym_meski.is_empty() {
        return rym::porównaj_rymy(&w1.rym_meski, &w2.rym_meski);
    }
    if (n1 >= 2 || n2 >= 2) && !w1.rym_zenski.is_empty() && !w2.rym_zenski.is_empty() {
        return rym::porównaj_rymy(&w1.rym_zenski, &w2.rym_zenski);
    }
    if !w1.rym_zenski.is_empty() && !w2.rym_zenski.is_empty() {
        return rym::porównaj_rymy(&w1.rym_zenski, &w2.rym_zenski);
    }
    if !w1.rym_meski.is_empty() && !w2.rym_meski.is_empty() {
        return rym::porównaj_rymy(&w1.rym_meski, &w2.rym_meski);
    }

    (false, 999, 0)
}

pub fn wyznacz_podświetlone_części(wers: &Wers) -> Vec<(String, bool)> {
    let text = &wers.surowy_tekst;
    let mut accented_spans = Vec::new();
    let (mut search_start, mut global_syl_idx) = (0, 0);

    for sw in &wers.słowa {
        if let Some(word_rel_pos) = text[search_start..].find(&sw.oryginalne) {
            let word_start = search_start + word_rel_pos;
            for &(s_start, s_end) in &sw.sylaba_spans() {
                let akcent_char = wers.akcent_wersu.chars().nth(global_syl_idx).unwrap_or('s');
                global_syl_idx += 1;
                if akcent_char == 'S' {
                    let abs_start = word_start + s_start.min(sw.oryginalne.len());
                    let abs_end = word_start + s_end.min(sw.oryginalne.len());
                    if abs_start < abs_end && abs_end <= text.len() {
                        accented_spans.push((abs_start, abs_end));
                    }
                }
            }
            search_start = word_start + sw.oryginalne.len();
        }
    }

    accented_spans.sort_by_key(|&(start, _)| start);
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for span in accented_spans {
        if let Some(last) = merged.last_mut() {
            if span.0 <= last.1 {
                last.1 = last.1.max(span.1);
            } else {
                merged.push(span);
            }
        } else {
            merged.push(span);
        }
    }

    let mut result = Vec::new();
    let mut last_idx = 0;
    for &(start, end) in &merged {
        if start > last_idx {
            result.push((text[last_idx..start].to_string(), false));
        }
        result.push((text[start..end].to_string(), true));
        last_idx = end;
    }
    if last_idx < text.len() {
        result.push((text[last_idx..].to_string(), false));
    }
    result
}

pub fn usun_podpis(wersy: &mut Vec<Wers>) {
    while let Some(w) = wersy.last() {
        if w.oryginalne.is_empty() { wersy.pop(); } else { break; }
    }
    if wersy.is_empty() { return; }

    let mut idx = wersy.len() - 1;
    let mut stanza_start = idx;
    let mut stanza_len = 0;

    while idx < wersy.len() {
        if wersy[idx].oryginalne.is_empty() { break; }
        stanza_start = idx;
        stanza_len += 1;
        if idx == 0 { break; }
        idx -= 1;
    }

    if stanza_len == 1 && wersy[stanza_start].surowy_tekst.trim_start().starts_with("--") {
        wersy.truncate(stanza_start);
        while let Some(w) = wersy.last() {
            if w.oryginalne.is_empty() { wersy.pop(); } else { break; }
        }
    }
}

pub fn czy_lekka_modyfikacja_wersu(w1: &Wers, w2: &Wers) -> bool {
    if w1.liczba_sylab != w2.liczba_sylab || w1.akcent_wersu != w2.akcent_wersu {
        return false;
    }
    let w1_last = w1.słowa.last().map(|s| s.oryginalne.to_lowercase());
    let w2_last = w2.słowa.last().map(|s| s.oryginalne.to_lowercase());
    if w1_last != w2_last {
        return false;
    }

    let words1: Vec<String> = w1.słowa.iter().map(|s| s.oryginalne.to_lowercase()).collect();
    let words2: Vec<String> = w2.słowa.iter().map(|s| s.oryginalne.to_lowercase()).collect();
    let n = words1.len();
    if n == 0 {
        return w1.oryginalne.trim().to_lowercase() == w2.oryginalne.trim().to_lowercase();
    }

    let max_len = words1.len().max(words2.len());
    let min_len = words1.len().min(words2.len());
    let mut diff_count = max_len - min_len;
    for i in 0..min_len {
        if words1[i] != words2[i] { diff_count += 1; }
    }
    diff_count <= ((n as f64) / 5.0).ceil() as usize
}

pub fn is_lekka_modyfikacja_wersu(w1: &Wers, w2: &Wers) -> bool {
    czy_lekka_modyfikacja_wersu(w1, w2)
}

pub fn wyznacz_zaakcentowane_zakresy_dla_linii(linia: &str, wers_obj: &Wers) -> Vec<(u32, u32)> {
    let text = &wers_obj.surowy_tekst;
    if text.is_empty() { return vec![]; }

    let line_offset = linia.find(text).unwrap_or(0);
    let mut accented_chars = Vec::new();
    let (mut search_start, mut global_syl_idx) = (0, 0);

    for sw in &wers_obj.słowa {
        if let Some(word_rel_pos) = text[search_start..].find(&sw.oryginalne) {
            let word_start = search_start + word_rel_pos;
            for &(s_start, s_end) in &sw.sylaba_spans() {
                let akcent_char = wers_obj.akcent_wersu.chars().nth(global_syl_idx).unwrap_or('s');
                global_syl_idx += 1;
                if akcent_char == 'S' {
                    let abs_start = word_start + s_start.min(sw.oryginalne.len());
                    let abs_end = word_start + s_end.min(sw.oryginalne.len());
                    if abs_start < abs_end && abs_end <= text.len() {
                        let (mut b_start, mut b_end) = (line_offset + abs_start, line_offset + abs_end);
                        b_start = b_start.min(linia.len());
                        while b_start > 0 && !linia.is_char_boundary(b_start) { b_start -= 1; }
                        b_end = b_end.min(linia.len());
                        while b_end < linia.len() && !linia.is_char_boundary(b_end) { b_end += 1; }

                        if b_start < b_end {
                            let char_start = linia[..b_start].chars().count() as u32;
                            let char_end = linia[..b_end].chars().count() as u32;
                            if char_start < char_end {
                                accented_chars.push((char_start, char_end));
                            }
                        }
                    }
                }
            }
            search_start = word_start + sw.oryginalne.len();
        }
    }
    accented_chars
}

pub fn wyznacz_podswietlone_czesci(wers: &Wers) -> Vec<(String, bool)> {
    wyznacz_podświetlone_części(wers)
}

pub fn get_accented_spans_for_line(linia: &str, wers_obj: &Wers) -> Vec<(u32, u32)> {
    wyznacz_zaakcentowane_zakresy_dla_linii(linia, wers_obj)
}

fn czy_głoska_samogłoska(głoska: &str) -> bool {
    rym::czy_samogloska(głoska)
}

impl Wers {
    pub fn czy_lekka_modyfikacja(&self, other: &Wers) -> bool {
        czy_lekka_modyfikacja_wersu(self, other)
    }

    pub fn wyznacz_podświetlone_części(&self) -> Vec<(String, bool)> {
        wyznacz_podświetlone_części(self)
    }

    pub fn wyznacz_zaakcentowane_zakresy(&self, linia: &str) -> Vec<(u32, u32)> {
        wyznacz_zaakcentowane_zakresy_dla_linii(linia, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rym_miedzy_wersami() {
        assert!(czy_rymują(&nowy_wers("a ja na to"), &nowy_wers("jak na lato")).0);
    }

    #[test]
    fn akcent_zdaniowy() {
        assert_eq!(nowy_wers("w objęcia mocno chwytając").akcent_wersu, "sSsSssSs");
        assert_eq!(nowy_wers("Nie rusz, Andziu, tego kwiatka,").akcent_wersu, "SsSsSsSs");
        assert_eq!(nowy_wers("Róża kole – rzekła matka.").akcent_wersu, "SsSsSsSs");
        assert_eq!(nowy_wers("Między nami nic nie było!").akcent_wersu, "SsSsSsSs");
        assert_eq!(nowy_wers("Żadnych zwierzeń, wyznań żadnych,").akcent_wersu, "SsSsSsSs");
        assert_eq!(nowy_wers("Nic nas z sobą nie łączyło").akcent_wersu, "SsSsssSs");
        assert_eq!(nowy_wers("Prócz wiosennych marzeń zdradnych;").akcent_wersu, "SsSsSsSs");
        assert_eq!(nowy_wers("Prócz tych woni, barw i blasków").akcent_wersu, "SsSsSsSs");
        assert_eq!(nowy_wers("Prócz szumiących śpiewem lasków").akcent_wersu, "SsSsSsSs");
        assert_eq!(nowy_wers("I tej świeżej łąk zieleni;").akcent_wersu, "SsSsSsSs");
        assert_eq!(
            nowy_wers("Niedostępna ludzkim oczom, że nikt po niej się nie błąka,").akcent_wersu,
            "PsSsSsSsSsSsSsSs"
        );
        assert_eq!(
            nowy_wers("W swym bezpieczu szmaragdowym rozkwitała w bezmiar łąka,").akcent_wersu,
            "SsSsPsSsPsSsSsSs"
        );
    }

    #[test]
    fn akcent_zdaniowy_przypadki_szczegolne() {
        assert_eq!(nowy_wers("nie wiem").akcent_wersu, "Ss");
        assert_eq!(nowy_wers("nie chce").akcent_wersu, "Ss");
        assert_eq!(nowy_wers("bez nich").akcent_wersu, "Ss");
        assert_eq!(nowy_wers("do was").akcent_wersu, "Ss");
        assert_eq!(nowy_wers("nie pomaga").akcent_wersu, "ssSs");
    }

    #[test]
    fn test_średniówka() {
        let w = nowy_wers("Litwo! Ojczyzno moja! ty jesteś jak zdrowie");
        assert_eq!(w.liczba_sylab, 13);
        assert_eq!(w.średniówka, "7+6");

        let w11 = nowy_wers("Któryś jest w niebie, bądź pochwalon Panie");
        assert_eq!(w11.liczba_sylab, 11);
        assert!(w11.średniówka == "5+6" || w11.średniówka == "6+5");
    }

    #[test]
    fn test_nowe_zasady_średniówki() {
        let cands = kandydaci_średniówki(
            &nowy_wers("Litwo! Ojczyzno moja! ty jesteś jak zdrowie").słowa,
            13,
        );
        let score_7_6 = cands.iter().find(|(l, _)| l == "7+6").map(|&(_, s)| s).unwrap_or(0);
        let score_6_7 = cands.iter().find(|(l, _)| l == "6+7").map(|&(_, s)| s).unwrap_or(0);
        assert!(score_7_6 > score_6_7);
    }
}
