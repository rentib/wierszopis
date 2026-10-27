/// Odpowiednik internal/wers w języku Rust.
use crate::rym;
use crate::slowo::{self, Słowo};

#[derive(Clone, Debug)]
pub struct Wers {
    pub surowy_tekst: String,
    pub oryginalne: String,
    pub słowa: Vec<Słowo>,
    pub liczba_sylab: usize,
    pub rym_meski: Vec<String>,
    pub rym_zenski: Vec<String>,
    pub średniówka: String,
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
        let czyste_słowo: String = sw.chars()
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

    Wers {
        surowy_tekst: surowy,
        oryginalne: czyste,
        słowa,
        liczba_sylab: całkowita_liczba_sylab,
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

    let num_vowels = |s: &Słowo| -> usize {
        s.głoski.iter().filter(|g| czy_głoska_samogłoska(g)).count()
    };

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
}

fn czy_głoska_samogłoska(głoska: &str) -> bool {
    rym::czy_samogloska(głoska)
}
