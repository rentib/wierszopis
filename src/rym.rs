/// Zaawansowany silnik rymów (inspirowany internal/rym).
use std::cmp::{max, min};
use std::collections::HashMap;

pub fn czy_samogloska(głoska: &str) -> bool {
    if głoska == "ᵘ" {
        return false;
    }
    let bazowa = głoska.trim_end_matches(|c| c == 'ⁱ' || c == 'ʲ');
    matches!(bazowa, "a" | "ą" | "e" | "ę" | "i" | "o" | "ó" | "u" | "y")
}

pub fn znajdz_ostatnia_samogloske(gloski: &[String]) -> usize {
    for i in (0..gloski.len()).rev() {
        if czy_samogloska(&gloski[i]) {
            return i;
        }
    }
    if gloski.is_empty() { 0 } else { gloski.len() - 1 }
}

fn koszt_dodania(g: &str, wygłos: bool) -> i32 {
    if czy_samogloska(g) {
        return 10;
    }

    match g {
        "b" => 3,
        "c" => if wygłos { 2 } else { 3 },
        "ć" => if wygłos { 2 } else { 3 },
        "ch" => if wygłos { 1 } else { 3 },
        "cz" | "d" | "dz" | "dź" | "dż" | "f" => 3,
        "g" => if wygłos { 2 } else { 3 },
        "h" => if wygłos { 2 } else { 3 },
        "j" => if wygłos { 2 } else { 3 },
        "k" => if wygłos { 2 } else { 3 },
        "l" => 3,
        "ł" => if wygłos { 2 } else { 3 },
        "m" => if wygłos { 1 } else { 3 },
        "n" | "ń" | "p" | "r" | "rz" => 3,
        "s" => if wygłos { 2 } else { 3 },
        "sz" => if wygłos { 2 } else { 3 },
        "ś" => if wygłos { 2 } else { 3 },
        "t" => 3,
        "w" => if wygłos { 2 } else { 3 },
        "z" => if wygłos { 2 } else { 3 },
        "ź" | "ż" => 3,
        _ => 10,
    }
}

static KOSZTY_ZAMIAN_WYGŁOS: std::sync::LazyLock<HashMap<(&'static str, &'static str), i32>> = std::sync::LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(("h", "ch"), 0); m.insert(("ch", "h"), 0);
    m.insert(("ż", "rz"), 0); m.insert(("rz", "ż"), 0);
    m.insert(("u", "ó"), 0); m.insert(("ó", "u"), 0);
    m.insert(("ł", "ᵘ"), 0); m.insert(("ᵘ", "ł"), 0);

    m.insert(("t", "d"), 0); m.insert(("d", "t"), 0);
    m.insert(("f", "w"), 0); m.insert(("w", "f"), 0);
    m.insert(("sz", "ż"), 0); m.insert(("ż", "sz"), 0);
    m.insert(("sz", "rz"), 0); m.insert(("rz", "sz"), 0);
    m.insert(("c", "dz"), 0); m.insert(("dz", "c"), 0);
    m.insert(("ś", "ź"), 0); m.insert(("ź", "ś"), 0);
    m.insert(("p", "b"), 0); m.insert(("b", "p"), 0);
    m.insert(("k", "g"), 0); m.insert(("g", "k"), 0);
    m.insert(("ć", "dź"), 0); m.insert(("dź", "ć"), 0);
    m.insert(("s", "z"), 0); m.insert(("z", "s"), 0);
    m.insert(("cz", "dż"), 0); m.insert(("dż", "cz"), 0);

    m.insert(("e", "ę"), 0); m.insert(("ę", "e"), 0);
    m.insert(("i", "y"), 1); m.insert(("y", "i"), 1);
    m.insert(("o", "ą"), 1); m.insert(("ą", "o"), 1);
    m.insert(("l", "ł"), 1); m.insert(("ł", "l"), 1);
    m.insert(("c", "cz"), 1); m.insert(("cz", "c"), 1);
    m.insert(("z", "ż"), 1); m.insert(("ż", "z"), 1);
    m.insert(("nⁱ", "mʲ"), 1); m.insert(("mʲ", "nⁱ"), 1);
    m.insert(("nⁱ", "n"), 1); m.insert(("n", "nⁱ"), 1);

    m.insert(("m", "n"), 2); m.insert(("n", "m"), 2);
    m.insert(("n", "ń"), 2); m.insert(("ń", "n"), 2);
    m.insert(("w", "ł"), 2); m.insert(("ł", "w"), 2);
    m.insert(("t", "c"), 2); m.insert(("c", "t"), 2);
    m.insert(("t", "ch"), 2); m.insert(("ch", "t"), 2);
    m
});

static KOSZTY_ZAMIAN_ŚRODEK: std::sync::LazyLock<HashMap<(&'static str, &'static str), i32>> = std::sync::LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(("h", "ch"), 0); m.insert(("ch", "h"), 0);
    m.insert(("ż", "rz"), 0); m.insert(("rz", "ż"), 0);
    m.insert(("u", "ó"), 0); m.insert(("ó", "u"), 0);
    m.insert(("ł", "ᵘ"), 0); m.insert(("ᵘ", "ł"), 0);

    m.insert(("t", "d"), 1); m.insert(("d", "t"), 1);
    m.insert(("f", "w"), 10); m.insert(("w", "f"), 10);
    m.insert(("sz", "ż"), 1); m.insert(("ż", "sz"), 1);
    m.insert(("sz", "rz"), 1); m.insert(("rz", "sz"), 1);
    m.insert(("c", "dz"), 1); m.insert(("dz", "c"), 1);
    m.insert(("ś", "ź"), 1); m.insert(("ź", "ś"), 1);
    m.insert(("p", "b"), 1); m.insert(("b", "p"), 1);
    m.insert(("k", "g"), 1); m.insert(("g", "k"), 1);
    m.insert(("ć", "dź"), 1); m.insert(("dź", "ć"), 1);
    m.insert(("s", "z"), 2); m.insert(("z", "s"), 2);
    m.insert(("cz", "dż"), 10); m.insert(("dż", "cz"), 10);

    m.insert(("e", "ę"), 2); m.insert(("ę", "e"), 2);
    m.insert(("i", "y"), 0); m.insert(("y", "i"), 0);
    m.insert(("o", "ą"), 1); m.insert(("ą", "o"), 1);
    m.insert(("l", "ł"), 1); m.insert(("ł", "l"), 1);
    m.insert(("c", "cz"), 1); m.insert(("cz", "c"), 1);
    m.insert(("z", "ż"), 1); m.insert(("ż", "z"), 1);
    m.insert(("nⁱ", "mʲ"), 1); m.insert(("mʲ", "nⁱ"), 1);
    m.insert(("nⁱ", "n"), 1); m.insert(("n", "nⁱ"), 1);

    m.insert(("m", "n"), 2); m.insert(("n", "m"), 2);
    m.insert(("n", "ń"), 2); m.insert(("ń", "n"), 2);
    m.insert(("w", "ł"), 2); m.insert(("ł", "w"), 2);
    m.insert(("t", "c"), 2); m.insert(("c", "t"), 2);
    m.insert(("t", "ch"), 2); m.insert(("ch", "t"), 2);
    m
});

fn koszt_zamiany(g1: &str, g2: &str, wygłos: bool) -> i32 {
    if g1 == g2 {
        return 0;
    }
    let key = (g1, g2);
    if wygłos {
        if let Some(&v) = KOSZTY_ZAMIAN_WYGŁOS.get(&key) {
            return v;
        }
    } else {
        if let Some(&v) = KOSZTY_ZAMIAN_ŚRODEK.get(&key) {
            return v;
        }
    }
    10
}

fn is_nasal_consonant(g: &str) -> bool {
    let base = g.trim_end_matches(|c| c == 'ⁱ' || c == 'ʲ');
    base == "n" || base == "m" || base == "ń"
}

/// Porównuje rymy między dwiema listami głosek. Zwraca (czy_się_rymują, koszt, próg).
pub fn porównaj_rymy(g1: &[String], g2: &[String]) -> (bool, i32, i32) {
    let n = g1.len();
    let m = g2.len();
    if n == 0 || m == 0 {
        return (false, 999, 0);
    }

    let last_vowel_1 = znajdz_ostatnia_samogloske(g1) as i32;
    let last_vowel_2 = znajdz_ostatnia_samogloske(g2) as i32;

    let req_len = m + 1;
    let mut r0 = vec![0i32; req_len];
    let mut r1 = vec![0i32; req_len];
    let mut r2 = vec![0i32; req_len];
    let mut r3 = vec![0i32; req_len];

    r2[0] = 0;
    for j in 0..m {
        let wygłos_2 = (j as i32 > last_vowel_2) || (last_vowel_2 == (m as i32 - 1) && j as i32 == last_vowel_2);
        r2[j + 1] = r2[j] + koszt_dodania(&g2[j], wygłos_2);
    }

    for i in 0..n {
        let wygłos_1 = (i as i32 > last_vowel_1) || (last_vowel_1 == (n as i32 - 1) && i as i32 == last_vowel_1);
        r3[0] = r2[0] + koszt_dodania(&g1[i], wygłos_1);

        for j in 0..m {
            let wygłos_1_cell = (i as i32 > last_vowel_1) || (last_vowel_1 == (n as i32 - 1) && i as i32 == last_vowel_1);
            let wygłos_2_cell = (j as i32 > last_vowel_2) || (last_vowel_2 == (m as i32 - 1) && j as i32 == last_vowel_2);
            let wygłos = wygłos_1_cell || wygłos_2_cell;

            let mut val = min(
                r2[j] + koszt_zamiany(&g1[i], &g2[j], wygłos),
                min(
                    r3[j] + koszt_dodania(&g2[j], wygłos_2_cell),
                    r2[j+1] + koszt_dodania(&g1[i], wygłos_1_cell),
                ),
            );

            if i >= 1 && j >= 2 {
                let gv = &g1[i - 1];
                let ov = &g2[j - 2];
                let nc = &g2[j - 1];
                if (gv == "ę" && ov == "e" && is_nasal_consonant(nc)) ||
                    (gv == "ą" && ov == "o" && is_nasal_consonant(nc)) {
                    val = min(val, r1[j - 2] + koszt_zamiany(&g1[i], &g2[j], wygłos));
                }
            }
            if i >= 2 && j >= 1 {
                let gv = &g2[j - 1];
                let ov = &g1[i - 2];
                let nc = &g1[i - 1];
                if (gv == "ę" && ov == "e" && is_nasal_consonant(nc)) ||
                    (gv == "ą" && ov == "o" && is_nasal_consonant(nc)) {
                    val = min(val, r0[j - 1] + koszt_zamiany(&g1[i], &g2[j], wygłos));
                }
            }

            if (g1[i] == "ę" && g2[j] == "e") ||
                (g1[i] == "e" && g2[j] == "ę") ||
                (g1[i] == "ą" && g2[j] == "ą") ||
                (g1[i] == "o" && g2[j] == "ą") {
                if (i + 1 < n && (g1[i + 1] == "l" || g1[i + 1] == "ł")) ||
                    (j + 1 < m && (g2[j + 1] == "l" || g2[j + 1] == "ł")) {
                    val = min(val, r2[j]);
                }
            }

            r3[j + 1] = val;
        }

        r0.clone_from(&r1);
        r1.clone_from(&r2);
        r2.clone_from(&r3);
    }

    let koszt = r2[m];
    let próg = (max(n, m) as i32 + 1) / 2;
    (koszt <= próg, koszt, próg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_porownaj_rymy() {
        let g1 = vec!["o".to_string(), "t".to_string()];
        let g2 = vec!["o".to_string(), "t".to_string()];
        let (rym, _, _) = porównaj_rymy(&g1, &g2);
        assert!(rym);
    }
}
