use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use wierszopis::{slowo, wers};

#[test]
fn test_pary_rymujące_słów() {
    let path = Path::new("tests/rymy/tak.txt");
    if !path.exists() {
        return;
    }
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    for (idx, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let s1 = parts[0];
        let s2 = parts[1];

        let w1 = slowo::nowe_słowo(s1);
        let w2 = slowo::nowe_słowo(s2);
        let (ok, koszt, prog) = slowo::czy_rymują(&w1, &w2);
        assert!(
            ok,
            "Linia {}: Oczekiwano, że słowa '{}' i '{}' się rymują (koszt: {}, próg: {})",
            idx + 1,
            s1,
            s2,
            koszt,
            prog
        );
    }
}

#[test]
fn test_pary_nierymujące_słów() {
    let path = Path::new("tests/rymy/nie.txt");
    if !path.exists() {
        return;
    }
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    for (idx, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let s1 = parts[0];
        let s2 = parts[1];

        let w1 = slowo::nowe_słowo(s1);
        let w2 = slowo::nowe_słowo(s2);
        let (ok, koszt, prog) = slowo::czy_rymują(&w1, &w2);
        assert!(
            !ok,
            "Linia {}: Oczekiwano, że słowa '{}' i '{}' się NIE rymują (koszt: {}, próg: {})",
            idx + 1,
            s1,
            s2,
            koszt,
            prog
        );
    }
}

fn test_sylaby_plik(skan_path: &str, oczekiwane_sylaby: usize) {
    let path = Path::new(skan_path);
    if !path.exists() {
        return;
    }
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    for (idx, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let w = wers::nowy_wers(&line);
        assert_eq!(
            w.liczba_sylab, oczekiwane_sylaby,
            "Plik {} linia {} ('{}'): oczekiwano {} sylab, otrzymano {}",
            skan_path,
            idx + 1,
            trimmed,
            oczekiwane_sylaby,
            w.liczba_sylab
        );
    }
}

#[test]
fn test_sylaby_7() {
    test_sylaby_plik("tests/sylaby/7.txt", 7);
}

#[test]
fn test_sylaby_8() {
    test_sylaby_plik("tests/sylaby/8.txt", 8);
}

#[test]
fn test_sylaby_9() {
    test_sylaby_plik("tests/sylaby/9.txt", 9);
}

#[test]
fn test_sylaby_10() {
    test_sylaby_plik("tests/sylaby/10.txt", 10);
}

#[test]
fn test_sylaby_11() {
    test_sylaby_plik("tests/sylaby/11.txt", 11);
}

#[test]
fn test_sylaby_12() {
    test_sylaby_plik("tests/sylaby/12.txt", 12);
}

#[test]
fn test_sylaby_13() {
    test_sylaby_plik("tests/sylaby/13.txt", 13);
}

#[test]
fn test_sylaby_14() {
    test_sylaby_plik("tests/sylaby/14.txt", 14);
}
