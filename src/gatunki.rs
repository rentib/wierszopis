use crate::wers::{self, Wers};
use crate::analizator::ZapisWersu;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RodzajSonetu {
    Szekspirowski,
    Spenserowski,
    Barnifieldowski,
    WłoskiPetrarkowski,
    FrancuskiMarotyczny,
    InnySonet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RodzajRondeau {
    Klasyczne10,
    Klasyczne13,
    Klasyczne15,
    Redouble,
    InneRondeau,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatunekLiteracki {
    Sonet(RodzajSonetu),
    Vilanella,
    Pantum,
    Triolet,
    Rondeau(RodzajRondeau),
    Limerick,
    Inny,
}

impl GatunekLiteracki {
    pub fn nazwa(&self) -> &'static str {
        match self {
            GatunekLiteracki::Sonet(sonet) => match sonet {
                RodzajSonetu::Szekspirowski => "sonet szekspirowski (elżbietański)",
                RodzajSonetu::Spenserowski => "sonet spenserowski",
                RodzajSonetu::Barnifieldowski => "sonet barnifieldowski",
                RodzajSonetu::WłoskiPetrarkowski => "sonet włoski (petrarkowski)",
                RodzajSonetu::FrancuskiMarotyczny => "sonet francuski (marotyczny)",
                RodzajSonetu::InnySonet => "inny rodzaj sonetu",
            },
            GatunekLiteracki::Vilanella => "vilanella",
            GatunekLiteracki::Pantum => "pantum",
            GatunekLiteracki::Triolet => "triolet",
            GatunekLiteracki::Rondeau(rondeau) => match rondeau {
                RodzajRondeau::Klasyczne10 => "rondeau (10-wersowe)",
                RodzajRondeau::Klasyczne13 => "rondeau (13-wersowe)",
                RodzajRondeau::Klasyczne15 => "rondeau (15-wersowe)",
                RodzajRondeau::Redouble => "rondeau redoublé",
                RodzajRondeau::InneRondeau => "inny rodzaj rondeau",
            },
            GatunekLiteracki::Limerick => "limeryk",
            GatunekLiteracki::Inny => "inny gatunek",
        }
    }
}

pub fn is_lekka_modyfikacja_wersu(w1: &Wers, w2: &Wers) -> bool {
    if w1.liczba_sylab != w2.liczba_sylab {
        return false;
    }
    if w1.akcent_wersu != w2.akcent_wersu {
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

    let max_len = std::cmp::max(words1.len(), words2.len());
    let min_len = std::cmp::min(words1.len(), words2.len());
    let mut diff_count = max_len - min_len;
    for i in 0..min_len {
        if words1[i] != words2[i] {
            diff_count += 1;
        }
    }
    let allowed_diff = ((n as f64) / 5.0).ceil() as usize;
    diff_count <= allowed_diff
}

pub fn schemat_rymów(wersy: &[ZapisWersu]) -> (String, Vec<usize>) {
    let mut schemat = String::new();
    let mut mapowanie_kolorów: std::collections::HashMap<usize, char> = std::collections::HashMap::new();
    let mut następna_litera = b'a';
    let mut numery_grup = Vec::new();

    for z in wersy {
        if z.wers.oryginalne.is_empty() {
            continue;
        }
        numery_grup.push(z.kolor);
        if z.kolor == 0 {
            schemat.push('-');
        } else {
            let litera = *mapowanie_kolorów.entry(z.kolor).or_insert_with(|| {
                let l = następna_litera as char;
                if następna_litera < b'z' {
                    następna_litera += 1;
                }
                l
            });
            schemat.push(litera);
        }
    }
    (schemat, numery_grup)
}

pub fn get_stanzas(wersy: &[ZapisWersu]) -> Vec<Vec<&ZapisWersu>> {
    let mut stanzas = Vec::new();
    let mut current = Vec::new();
    for z in wersy {
        if z.wers.oryginalne.is_empty() {
            if !current.is_empty() {
                stanzas.push(current);
                current = Vec::new();
            }
        } else {
            current.push(z);
        }
    }
    if !current.is_empty() {
        stanzas.push(current);
    }
    stanzas
}

pub fn analizuj_sonet(wersy: &[ZapisWersu]) -> Option<RodzajSonetu> {
    let (_schemat, grupy) = schemat_rymów(wersy);
    if grupy.len() != 14 {
        return None;
    }

    if grupy.iter().any(|&g| g == 0) {
        return None;
    }

    let mut normalized = [0usize; 14];
    let mut seen = std::collections::HashMap::new();
    let mut next_id = 0;
    for (i, &g) in grupy.iter().enumerate() {
        let id = *seen.entry(g).or_insert_with(|| {
            let n = next_id;
            next_id += 1;
            n
        });
        normalized[i] = id;
    }

    // 1. Szekspirowski: abab cdcd efef gg
    if normalized[0] != normalized[1]
        && normalized[2] == normalized[0] && normalized[3] == normalized[1]
        && normalized[4] != normalized[5]
        && normalized[6] == normalized[4] && normalized[7] == normalized[5]
        && normalized[4] != normalized[0] && normalized[4] != normalized[1]
        && normalized[5] != normalized[0] && normalized[5] != normalized[1]
        && normalized[8] != normalized[9]
        && normalized[10] == normalized[8] && normalized[11] == normalized[9]
        && normalized[8] != normalized[0] && normalized[8] != normalized[1]
        && normalized[8] != normalized[4] && normalized[8] != normalized[5]
        && normalized[9] != normalized[0] && normalized[9] != normalized[1]
        && normalized[9] != normalized[4] && normalized[9] != normalized[5]
        && normalized[12] == normalized[13]
        && normalized[12] != normalized[0] && normalized[12] != normalized[1]
        && normalized[12] != normalized[4] && normalized[12] != normalized[5]
        && normalized[12] != normalized[8] && normalized[12] != normalized[9]
    {
        return Some(RodzajSonetu::Szekspirowski);
    }

    // 2. Spenserowski: abab bcbc cdcd ee
    if normalized[0] != normalized[1]
        && normalized[2] == normalized[0] && normalized[3] == normalized[1]
        && normalized[4] == normalized[1]
        && normalized[5] != normalized[4]
        && normalized[6] == normalized[4]
        && normalized[7] == normalized[5]
        && normalized[8] == normalized[5]
        && normalized[9] != normalized[8]
        && normalized[10] == normalized[8]
        && normalized[11] == normalized[9]
        && normalized[12] == normalized[13]
        && normalized[12] != normalized[0] && normalized[12] != normalized[1]
        && normalized[12] != normalized[5] && normalized[12] != normalized[9]
    {
        return Some(RodzajSonetu::Spenserowski);
    }

    // 3. Barnifieldowski: abba cddc effe gg
    if normalized[0] == normalized[3] && normalized[1] == normalized[2] && normalized[0] != normalized[1]
        && normalized[4] == normalized[7] && normalized[5] == normalized[6] && normalized[4] != normalized[5]
        && normalized[4] != normalized[0] && normalized[4] != normalized[1]
        && normalized[5] != normalized[0] && normalized[5] != normalized[1]
        && normalized[8] == normalized[11] && normalized[9] == normalized[10] && normalized[8] != normalized[9]
        && normalized[8] != normalized[0] && normalized[8] != normalized[1] && normalized[8] != normalized[4] && normalized[8] != normalized[5]
        && normalized[9] != normalized[0] && normalized[9] != normalized[1] && normalized[9] != normalized[4] && normalized[9] != normalized[5]
        && normalized[12] == normalized[13]
        && normalized[12] != normalized[0] && normalized[12] != normalized[1]
        && normalized[12] != normalized[4] && normalized[12] != normalized[5]
        && normalized[12] != normalized[8] && normalized[12] != normalized[9]
    {
        return Some(RodzajSonetu::Barnifieldowski);
    }

    let q1_abba = normalized[0] == normalized[3] && normalized[1] == normalized[2] && normalized[0] != normalized[1];
    let q1_abab = normalized[0] == normalized[2] && normalized[1] == normalized[3] && normalized[0] != normalized[1];
    let q2_abba = normalized[4] == normalized[7] && normalized[5] == normalized[6] && normalized[4] != normalized[5];
    let q2_abab = normalized[4] == normalized[6] && normalized[5] == normalized[7] && normalized[4] != normalized[5];

    let ma_czterowiersze = (q1_abba || q1_abab) && (q2_abba || q2_abab);

    if ma_czterowiersze {
        let mut ma_kuplet_w_tercetach = false;
        for i in 8..13 {
            if normalized[i] == normalized[i + 1] {
                ma_kuplet_w_tercetach = true;
                break;
            }
        }
        if ma_kuplet_w_tercetach {
            return Some(RodzajSonetu::FrancuskiMarotyczny);
        } else {
            return Some(RodzajSonetu::WłoskiPetrarkowski);
        }
    }

    Some(RodzajSonetu::InnySonet)
}

pub fn analizuj_vilanelle(wersy: &[ZapisWersu]) -> Option<()> {
    let non_empty: Vec<&ZapisWersu> = wersy.iter()
        .filter(|z| !z.wers.oryginalne.is_empty())
        .collect();

    if non_empty.len() != 19 {
        return None;
    }

    let stanzas = get_stanzas(wersy);
    let stanza_lens: Vec<usize> = stanzas.iter().map(|s| s.len()).collect();

    if stanza_lens != vec![3, 3, 3, 3, 3, 4] {
        return None;
    }

    let check_refrain = |indices: &[usize]| -> Option<(usize, String, String)> {
        if indices.is_empty() {
            return None;
        }
        let first_w = &non_empty[indices[0]].wers;
        let syl = first_w.liczba_sylab;
        let średniówka = first_w.średniówka.clone();
        let last_word = first_w.słowa.last()?.oryginalne.to_lowercase();

        for &idx in &indices[1..] {
            let w = &non_empty[idx].wers;
            if !is_lekka_modyfikacja_wersu(first_w, w) {
                return None;
            }
            if w.średniówka != średniówka {
                return None;
            }
        }
        Some((syl, średniówka, last_word))
    };

    let a1_indices = [0, 5, 11, 17];
    let a2_indices = [2, 8, 14, 18];

    let a1_props = check_refrain(&a1_indices)?;
    let a2_props = check_refrain(&a2_indices)?;

    if a1_props.2 == a2_props.2 {
        return None;
    }

    Some(())
}

pub fn analizuj_pantum(wersy: &[ZapisWersu]) -> Option<()> {
    let stanzas = get_stanzas(wersy);
    if stanzas.len() < 2 {
        return None;
    }
    for i in 0..stanzas.len() - 1 {
        let s1 = &stanzas[i];
        let s2 = &stanzas[i + 1];
        if s1.len() != 4 || s2.len() != 4 {
            return None;
        }
        let line2_s1 = s1[1].wers.oryginalne.trim().to_lowercase();
        let line1_s2 = s2[0].wers.oryginalne.trim().to_lowercase();
        let line4_s1 = s1[3].wers.oryginalne.trim().to_lowercase();
        let line3_s2 = s2[2].wers.oryginalne.trim().to_lowercase();

        if line2_s1 != line1_s2 || line4_s1 != line3_s2 {
            return None;
        }
    }
    Some(())
}

pub fn analizuj_triolet(wersy: &[ZapisWersu]) -> Option<()> {
    let non_empty: Vec<&ZapisWersu> = wersy.iter()
        .filter(|z| !z.wers.oryginalne.is_empty())
        .collect();

    if non_empty.len() != 8 {
        return None;
    }

    let stanzas = get_stanzas(wersy);
    let has_stanza_8 = stanzas.iter().any(|s| s.len() == 8);
    if !has_stanza_8 && stanzas.len() != 1 {
        return None;
    }

    let w1 = &non_empty[0].wers;
    let w2 = &non_empty[1].wers;
    let w3 = &non_empty[2].wers;
    let w4 = &non_empty[3].wers;
    let w5 = &non_empty[4].wers;
    let w6 = &non_empty[5].wers;
    let w7 = &non_empty[6].wers;
    let w8 = &non_empty[7].wers;

    // Rym A: w1, w3, w5
    let (r13, _, _) = wers::czy_rymują(w1, w3);
    let (r15, _, _) = wers::czy_rymują(w1, w5);
    if !r13 || !r15 {
        return None;
    }

    // Rym B: w2, w6
    let (r26, _, _) = wers::czy_rymują(w2, w6);
    if !r26 {
        return None;
    }

    // A and B must not rhyme together
    let (r12, _, _) = wers::czy_rymują(w1, w2);
    if r12 {
        return None;
    }

    // Refrains: w4 and w7 are slight modifications of w1; w8 is slight modification of w2
    if !is_lekka_modyfikacja_wersu(w1, w4) {
        return None;
    }
    if !is_lekka_modyfikacja_wersu(w1, w7) {
        return None;
    }
    if !is_lekka_modyfikacja_wersu(w2, w8) {
        return None;
    }

    Some(())
}

pub fn analizuj_rondeau(wersy: &[ZapisWersu]) -> Option<RodzajRondeau> {
    let non_empty: Vec<&ZapisWersu> = wersy.iter()
        .filter(|z| !z.wers.oryginalne.is_empty())
        .collect();

    let len = non_empty.len();
    if len != 10 && len != 13 && len != 15 && len != 24 {
        return None;
    }

    // Check that we have exactly 2 main rhyme groups (or at most 2-3 considering refrains)
    let (_schemat, grupy) = schemat_rymów(wersy);
    if grupy.iter().any(|&g| g == 0) {
        return None;
    }

    let mut unique_groups = std::collections::HashSet::new();
    for &g in &grupy {
        unique_groups.insert(g);
    }

    if unique_groups.len() > 3 {
        return None;
    }

    if len == 10 {
        // Typically 10 lines: ABBAABc ABBAc
        return Some(RodzajRondeau::Klasyczne10);
    } else if len == 13 {
        return Some(RodzajRondeau::Klasyczne13);
    } else if len == 15 {
        return Some(RodzajRondeau::Klasyczne15);
    } else if len == 24 {
        return Some(RodzajRondeau::Redouble);
    }

    Some(RodzajRondeau::InneRondeau)
}

pub fn analizuj_limeryk(wersy: &[ZapisWersu]) -> Option<()> {
    let non_empty: Vec<&ZapisWersu> = wersy.iter()
        .filter(|z| !z.wers.oryginalne.is_empty())
        .collect();

    if non_empty.len() != 5 {
        return None;
    }

    let stanzas = get_stanzas(wersy);
    // Limeryk to zazwyczaj jedna strofa 5-wersowa
    if stanzas.len() != 1 || stanzas[0].len() != 5 {
        return None;
    }

    let w1 = &non_empty[0].wers;
    let w2 = &non_empty[1].wers;
    let w3 = &non_empty[2].wers;
    let w4 = &non_empty[3].wers;
    let w5 = &non_empty[4].wers;

    // Schemat rymów AABBA: 1, 2, 5 rymują się; 3, 4 rymują się; 1 i 3 NIE rymują się.
    let (r12, _, _) = wers::czy_rymują(w1, w2);
    let (r15, _, _) = wers::czy_rymują(w1, w5);
    let (r34, _, _) = wers::czy_rymują(w3, w4);
    let (r13, _, _) = wers::czy_rymują(w1, w3);

    if (!r12 && non_empty[0].kolor != non_empty[1].kolor) || (!r15 && non_empty[0].kolor != non_empty[4].kolor) {
        // Fallback to kolor check if phonetic rym returns false due to dictionary limitations
        if non_empty[0].kolor != non_empty[1].kolor || non_empty[0].kolor != non_empty[4].kolor {
            return None;
        }
    }

    if !r34 && non_empty[2].kolor != non_empty[3].kolor {
        return None;
    }

    if r13 || non_empty[0].kolor == non_empty[2].kolor {
        return None;
    }

    // Sylabika: wersy 1, 2, 5 zazwyczaj dłuższe niż 3, 4
    // (lub akceptujemy dowolne AABBA jeśli długości wierszy spełniają warunek długich i krótkich)
    let s1 = w1.liczba_sylab;
    let s2 = w2.liczba_sylab;
    let s3 = w3.liczba_sylab;
    let s4 = w4.liczba_sylab;
    let s5 = w5.liczba_sylab;

    if s1 > s3 && s2 > s4 && s5 > s3 {
        Some(())
    } else {
        // Nawet jeśli sylaby są równe lub niezbyt zróżnicowane, jeśli rymy to AABBA, uznajemy za limeryk
        Some(())
    }
}

pub fn gatunek_literacki(wersy: &[ZapisWersu]) -> Option<GatunekLiteracki> {
    if let Some(sonet) = analizuj_sonet(wersy) {
        return Some(GatunekLiteracki::Sonet(sonet));
    }
    if analizuj_vilanelle(wersy).is_some() {
        return Some(GatunekLiteracki::Vilanella);
    }
    if analizuj_pantum(wersy).is_some() {
        return Some(GatunekLiteracki::Pantum);
    }
    if analizuj_triolet(wersy).is_some() {
        return Some(GatunekLiteracki::Triolet);
    }
    if let Some(rondeau) = analizuj_rondeau(wersy) {
        return Some(GatunekLiteracki::Rondeau(rondeau));
    }
    if analizuj_limeryk(wersy).is_some() {
        return Some(GatunekLiteracki::Limerick);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wers;

    fn make_zapis(text: &str, kolor: usize) -> ZapisWersu {
        ZapisWersu {
            linenr: 1,
            kolor,
            koszt: 0,
            max: 0,
            wers: wers::nowy_wers(text),
            ma_rym: true,
        }
    }

    #[test]
    fn test_rozpoznawanie_rondeau() {
        let mut wersy = Vec::new();
        for i in 0..10 {
            let kol = if i % 2 == 0 { 1 } else { 2 };
            wersy.push(make_zapis("Kot wlazł na płot", kol));
        }
        let gatunek = gatunek_literacki(&wersy);
        assert_eq!(gatunek, Some(GatunekLiteracki::Rondeau(RodzajRondeau::Klasyczne10)));
    }

    #[test]
    fn test_rozpoznawanie_limeryku() {
        let wersy = vec![
            make_zapis("Mieszkał w Krakowie stary pan", 1),
            make_zapis("Który miał w kieszeni duży dzban", 1),
            make_zapis("Lecz w tym dzbanie", 2),
            make_zapis("Puste dranie", 2),
            make_zapis("Schowały stary sagan", 1),
        ];
        let gatunek = gatunek_literacki(&wersy);
        assert_eq!(gatunek, Some(GatunekLiteracki::Limerick));
    }
}
