/// Odpowiednik internal/wers w języku Rust.
use crate::rym;
use crate::slowo::{self, Słowo};

#[derive(Clone, Debug)]
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

    for i in 0..słowa.len() {
        let w_lower = słowa[i].oryginalne.to_lowercase();

        if i > 0 && proclitics_di.contains(&w_lower.as_str()) && i + 1 < słowa.len() {
            let len = word_accents[i].len();
            if len > 0 {
                let mut acc = vec!['s'; len];
                acc[len - 1] = 'S';
                word_accents[i] = acc.into_iter().collect();
            }
            let next_len = word_accents[i + 1].len();
            word_accents[i + 1] = "s".repeat(next_len);
        } else if w_lower == "nie" && i + 1 < słowa.len() {
            let next_len = word_accents[i + 1].len();
            if next_len == 1 {
                // "nie" + monosyllabic verb (e.g., "nie wiem", "nie chce") -> accent on "nie" (S), verb unstressed (s)
                word_accents[i] = "S".to_string();
                word_accents[i + 1] = "s".to_string();
            } else {
                // "nie" + disyllabic or longer verb -> proclitic, "nie" is unstressed (s)
                let len = word_accents[i].len();
                word_accents[i] = "s".repeat(len);
            }
        } else if i > 0 && proclitics_mono.contains(&w_lower.as_str()) && i + 1 < słowa.len() {
            let next_len = word_accents[i + 1].len();
            let len = word_accents[i].len();
            if next_len == 1 {
                // Monosyllabic preposition + monosyllabic word (e.g., "bez nich", "do was") -> accent on preposition (S), following word unstressed (s)
                word_accents[i] = "S".repeat(len);
                word_accents[i + 1] = "s".to_string();
            } else {
                // Monosyllabic preposition + disyllabic/longer word -> preposition is unstressed (s)
                word_accents[i] = "s".repeat(len);
            }
        } else if word_accents[i].len() == 1
            && i + 1 < słowa.len()
            && word_accents[i + 1].len() == 1
        {
            // General zbitka dwóch słów jednosylabowych (monosyllabic word pair): first is S, second is s
            word_accents[i] = "S".to_string();
            word_accents[i + 1] = "s".to_string();
        }

        if enclitics.contains(&w_lower.as_str()) {
            let len = word_accents[i].len();
            word_accents[i] = "s".repeat(len);
        }
    }

    word_accents.concat()
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

    let surowe_słowa: Vec<&str> = czyste.split_whitespace().collect();
    let mut słowa = Vec::new();
    let mut całkowita_liczba_sylab = 0;
    let mut wszystkie_głoski = Vec::new();

    for sw in surowe_słowa {
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

    let mut indeksy_samogłosek = Vec::new();
    for (i, g) in wszystkie_głoski.iter().enumerate() {
        if czy_głoska_samogłoska(g) {
            indeksy_samogłosek.push(i);
        }
    }

    let mut rym_meski = Vec::new();
    let mut rym_zenski = Vec::new();

    if !indeksy_samogłosek.is_empty() {
        let ostatnia = indeksy_samogłosek[indeksy_samogłosek.len() - 1];
        rym_meski = wszystkie_głoski[ostatnia..].to_vec();
    }

    if indeksy_samogłosek.len() >= 2 {
        let przedostatnia = indeksy_samogłosek[indeksy_samogłosek.len() - 2];
        rym_zenski = wszystkie_głoski[przedostatnia..].to_vec();
    }

    let akcent_wersu = wyznacz_akcent_wersu(&słowa);

    Wers {
        surowy_tekst: surowy,
        oryginalne: czyste,
        słowa,
        liczba_sylab: całkowita_liczba_sylab,
        akcent_wersu,
        rym_meski,
        rym_zenski,
        średniówka: "-".to_string(),
    }
}

pub fn czy_rymują(w1: &Wers, w2: &Wers) -> (bool, i32, i32) {
    if w1.słowa.is_empty() || w2.słowa.is_empty() {
        return (false, 999, 0);
    }

    let last1 = &w1.słowa[w1.słowa.len() - 1];
    let last2 = &w2.słowa[w2.słowa.len() - 1];

    let num_vowels =
        |s: &Słowo| -> usize { s.głoski.iter().filter(|g| czy_głoska_samogłoska(g)).count() };

    let n1 = num_vowels(last1);
    let n2 = num_vowels(last2);

    // 1) Oba kończą się jednosylabowym słowem -> rymy męskie
    if n1 == 1 && n2 == 1 {
        if !w1.rym_meski.is_empty() && !w2.rym_meski.is_empty() {
            return rym::porównaj_rymy(&w1.rym_meski, &w2.rym_meski);
        }
        return (false, 999, 0);
    }

    // 2) Co najmniej jeden kończy się słowem dwusylabowym (lub wielosylabowym) -> rymy żeńskie
    if n1 >= 2 || n2 >= 2 {
        if !w1.rym_zenski.is_empty() && !w2.rym_zenski.is_empty() {
            return rym::porównaj_rymy(&w1.rym_zenski, &w2.rym_zenski);
        }
    }

    if !w1.rym_zenski.is_empty() && !w2.rym_zenski.is_empty() {
        return rym::porównaj_rymy(&w1.rym_zenski, &w2.rym_zenski);
    }

    if !w1.rym_meski.is_empty() && !w2.rym_meski.is_empty() {
        return rym::porównaj_rymy(&w1.rym_meski, &w2.rym_meski);
    }

    (false, 999, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_za_to_lato_wers() {
        let w1 = nowy_wers("za to");
        let w2 = nowy_wers("lato");
        let (ok, _, _) = czy_rymują(&w1, &w2);
        assert!(ok);
    }

    #[test]
    fn test_w_objecia_mocno_chwytajac() {
        let w = nowy_wers("w objęcia mocno chwytając");
        assert_eq!(w.akcent_wersu, "sSsSssSs");
    }

    #[test]
    fn test_akcent_zdaniowy_przypadki() {
        assert_eq!(nowy_wers("nie wiem").akcent_wersu, "Ss");
        assert_eq!(nowy_wers("nie chce").akcent_wersu, "Ss");
        assert_eq!(nowy_wers("bez nich").akcent_wersu, "Ss");
        assert_eq!(nowy_wers("do was").akcent_wersu, "Ss");
        assert_eq!(nowy_wers("nie pomaga").akcent_wersu, "ssSs");
    }

    #[test]
    fn trochej() {
        assert_eq!(
            nowy_wers("Nie rusz, Andziu, tego kwiatka,").akcent_wersu,
            "SsSsSsSs"
        );
        assert_eq!(
            nowy_wers("Róża kole – rzekła matka.").akcent_wersu,
            "SsSsSsSs"
        );
        assert_eq!(
            nowy_wers("Między nami nic nie było!").akcent_wersu,
            "SsSsSsSs"
        );
        assert_eq!(
            nowy_wers("Żadnych zwierzeń, wyznań żadnych,").akcent_wersu,
            "SsSsSsSs"
        );
        assert_eq!(
            nowy_wers("Nic nas z sobą nie łączyło").akcent_wersu,
            "SsSsssSs"
        );
        assert_eq!(
            nowy_wers("Prócz wiosennych marzeń zdradnych;").akcent_wersu,
            "SsSsSsSs"
        );
        assert_eq!(
            nowy_wers("Prócz tych woni, barw i blasków").akcent_wersu,
            "SsSsSsSs"
        );
        assert_eq!(
            nowy_wers("Prócz szumiących śpiewem lasków").akcent_wersu,
            "SsSsSsSs"
        );
        assert_eq!(
            nowy_wers("I tej świeżej łąk zieleni;").akcent_wersu,
            "SsSsSsSs"
        );
        // assert_eq!(nowy_wers("").akcent_wersu, "SsSsSsSs");
    }
}

fn czy_głoska_samogłoska(głoska: &str) -> bool {
    rym::czy_samogloska(głoska)
}
