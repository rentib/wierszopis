/// Moduł obsługi słownika i katalogów XDG.
use crate::indeks::WpisSlownika;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

pub fn pobierz_sciezke_slownika() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        Some(PathBuf::from(xdg).join("wierszopis").join("slownik.csv"))
    } else if let Some(home) = std::env::var_os("HOME") {
        Some(
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("wierszopis")
                .join("slownik.csv"),
        )
    } else {
        None
    }
}

pub fn pobierz_sciezke_indeksu() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        Some(PathBuf::from(xdg).join("wierszopis").join("slownik.bin"))
    } else if let Some(home) = std::env::var_os("HOME") {
        Some(
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("wierszopis")
                .join("slownik.bin"),
        )
    } else {
        None
    }
}

pub fn załaduj_słownik() -> io::Result<crate::indeks::IndeksCache> {
    if let Some(bin_path) = pobierz_sciezke_indeksu() {
        if bin_path.exists() {
            if let Ok(file) = File::open(&bin_path) {
                let mut reader = BufReader::new(file);
                if let Ok(cache) = crate::indeks::IndeksCache::deserialize(&mut reader) {
                    return Ok(cache);
                }
            }
        }
    }

    let csv_path = pobierz_sciezke_slownika()
        .filter(|p| p.exists())
        .or_else(|| {
            let p = PathBuf::from("slownik.csv");
            if p.exists() { Some(p) } else { None }
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Nie znaleziono pliku słownika (slownik.bin ani slownik.csv)",
            )
        })?;

    let file = File::open(&csv_path)?;
    let reader = BufReader::new(file);
    let mut wpisy = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        if idx == 0 && line.starts_with("slowo,") {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 5 {
            let slowo = parts[0].trim().to_string();
            let forma_podstawowa = parts[1].trim().to_string();
            let czesc_mowy = parts[2].trim().to_string();
            let cechy_gramatyczne = parts[3].trim().to_string();
            let pelny_tag = parts[4].trim().to_string();
            if !slowo.is_empty() {
                let liczba_sylab = crate::fonetyka::zlicz_sylaby(&slowo);
                wpisy.push(WpisSlownika {
                    slowo,
                    forma_podstawowa,
                    czesc_mowy,
                    cechy_gramatyczne,
                    pelny_tag,
                    liczba_sylab,
                });
            }
        }
    }

    let mut buckets: Vec<Vec<usize>> = Vec::new();
    for (idx, entry) in wpisy.iter().enumerate() {
        let s = crate::slowo::nowe_słowo(&entry.slowo);
        let id = s.klucz_bucketa_id();
        if buckets.len() <= id {
            buckets.resize_with(id + 1, Vec::new);
        }
        buckets[id].push(idx);
    }

    Ok(crate::indeks::IndeksCache {
        entries: wpisy,
        buckets,
    })
}

pub fn przetłumacz_część_mowy(pos: &str) -> &str {
    let p = pos.to_lowercase();
    if p.starts_with("subst") {
        "Rzeczownik"
    } else if p == "depr" {
        "Rzeczownik (deprecjatywny)"
    } else if p == "adj" || p == "adjp" || p == "adjc" || p.starts_with("adj") {
        "Przymiotnik"
    } else if p == "verb"
        || p == "fin"
        || p == "bedzie"
        || p == "inf"
        || p == "praet"
        || p == "impt"
        || p == "ger"
        || p == "pact"
        || p == "ppas"
        || p == "winien"
        || p == "pred"
        || p == "imps"
        || p == "pcon"
        || p == "pant"
        || p.starts_with("verb")
    {
        "Czasownik"
    } else if p == "adv" {
        "Przysłówek"
    } else if p == "num" || p == "romandig" {
        "Liczebnik"
    } else if p.starts_with("pron") || p == "ppron12" || p == "ppron3" || p == "siebie" {
        "Zaimek"
    } else if p == "prep" {
        "Przyimek"
    } else if p == "conj" || p == "comp" {
        "Spójnik"
    } else if p == "interj" {
        "Wykrzyknik"
    } else if p == "part" || p == "frag" {
        "Partykuła"
    } else if p == "ign" || p == "" {
        "Nierozpoznane"
    } else {
        pos
    }
}
