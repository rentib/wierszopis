/// Odpowiednik internal/analizator w języku Rust.
use crate::wers::{self, Wers};
use crossterm::style::{Color, Stylize};
use std::io::{self, BufRead};

pub struct Analizator;

impl Analizator {
    pub fn nowy() -> Self {
        Analizator
    }
}

const PALETA_KOLORÓW: &[u8] = &[160, 28, 166, 27, 93, 30, 124, 34, 208, 21, 54, 37];

#[allow(dead_code)]
struct ZapisWersu {
    linenr: usize,
    kolor: usize,
    koszt: i32,
    max: i32,
    wers: Wers,
    ma_rym: bool,
}

pub fn analizuj_strumien(reader: impl io::Read, szczegoly: bool) -> io::Result<()> {
    let buf_reader = io::BufReader::new(reader);
    let mut wersy = Vec::new();

    for line in buf_reader.lines() {
        let line = line?;
        wersy.push(wers::nowy_wers(&line));
    }

    #[allow(dead_code)]
    struct ZapisWersu {
        linenr: usize,
        kolor: usize,
        koszt: i32,
        max: i32,
        wers: Wers,
        ma_rym: bool,
    }

    let mut zapisane: Vec<ZapisWersu> = Vec::new();
    for (idx, w) in wersy.iter().enumerate() {
        zapisane.push(ZapisWersu {
            linenr: idx + 1,
            kolor: 0,
            koszt: 0,
            max: 0,
            wers: w.clone(),
            ma_rym: false,
        });
    }

    let mut start = 0;
    while start < zapisane.len() {
        if zapisane[start].wers.oryginalne.is_empty() {
            start += 1;
            continue;
        }
        let mut end = start;
        while end < zapisane.len() && !zapisane[end].wers.oryginalne.is_empty() {
            end += 1;
        }

        for i in start..end {
            let a_wers = &zapisane[i].wers;
            if a_wers.rym_meski.is_empty() && a_wers.rym_zenski.is_empty() {
                continue;
            }

            let mut best_j = None;
            let mut min_koszt = i32::MAX;
            let mut best_max = 0;

            for j in start..i {
                let b_wers = &zapisane[j].wers;
                if b_wers.rym_meski.is_empty() && b_wers.rym_zenski.is_empty() {
                    continue;
                }

                let (ok, koszt, max_k) = wers::czy_rymują(b_wers, a_wers);
                if ok && koszt < min_koszt {
                    min_koszt = koszt;
                    best_max = max_k;
                    best_j = Some(j);
                }
            }

            if let Some(j) = best_j {
                let group_id = std::cmp::min(zapisane[j].linenr, zapisane[i].linenr);
                zapisane[i].kolor = group_id;
                zapisane[i].koszt = min_koszt;
                zapisane[i].max = best_max;
                zapisane[i].ma_rym = true;

                zapisane[j].kolor = group_id;
                zapisane[j].koszt = min_koszt;
                zapisane[j].max = best_max;
                zapisane[j].ma_rym = true;
            }
        }

        start = end;
    }

    let mut unikalne_kolory_set: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
    for z in &zapisane {
        if z.kolor != 0 {
            unikalne_kolory_set.insert(z.kolor);
        }
    }

    let mut unikalne_kolory: std::collections::HashMap<usize, (usize, usize)> = std::collections::HashMap::new();
    let mut paleta_idx = 0;

    for col in unikalne_kolory_set {
        let num = paleta_idx + 1;
        let idx = paleta_idx % PALETA_KOLORÓW.len();
        paleta_idx += 1;
        unikalne_kolory.insert(col, (num, idx));
    }

    let mut total_niepustych = 0;
    let mut max_syl_len = 1;
    let mut max_cost_len = 1;
    let mut max_color_len = 1;

    for z in &zapisane {
        if z.wers.oryginalne.is_empty() {
            continue;
        }
        total_niepustych += 1;
        let syl_str = z.wers.liczba_sylab.to_string();
        if syl_str.len() > max_syl_len {
            max_syl_len = syl_str.len();
        }
        let cost_str = if !z.ma_rym {
            "-".to_string()
        } else {
            format!("{}/{}", z.koszt, z.max)
        };
        if cost_str.len() > max_cost_len {
            max_cost_len = cost_str.len();
        }
        let color_str = if z.kolor == 0 {
            "-".to_string()
        } else if let Some(&(num, _)) = unikalne_kolory.get(&z.kolor) {
            num.to_string()
        } else {
            "-".to_string()
        };
        if color_str.len() > max_color_len {
            max_color_len = color_str.len();
        }
    }

    let max_nr_len = total_niepustych.to_string().len();

    let mut numer_niepustego = 0;
    for z in &zapisane {
        if z.wers.oryginalne.is_empty() {
            println!();
            continue;
        }
        numer_niepustego += 1;

        let nr_str = format!("{:>width$}", numer_niepustego, width = max_nr_len);
        let syl_str = format!("{:>width$}", z.wers.liczba_sylab, width = max_syl_len);

        let etykieta = if szczegoly {
            let cost_str = if !z.ma_rym {
                "-".to_string()
            } else {
                format!("{}/{}", z.koszt, z.max)
            };
            let c_str = format!("{:>width$}", cost_str, width = max_cost_len);

            let color_str = if z.kolor == 0 {
                "-".to_string()
            } else if let Some(&(num, _)) = unikalne_kolory.get(&z.kolor) {
                num.to_string()
            } else {
                "-".to_string()
            };
            let col_str = format!("{:>width$}", color_str, width = max_color_len);
            format!("[{}] [{}] [{}] [{}]", nr_str, syl_str, c_str, col_str)
        } else {
            format!("[{}] [{}]", nr_str, syl_str)
        };
        let tekst = &z.wers.surowy_tekst;

        if z.kolor != 0 {
            if let Some(&(_, paleta_i)) = unikalne_kolory.get(&z.kolor) {
                let color_code = PALETA_KOLORÓW[paleta_i];
                let styled = tekst.as_str().with(Color::AnsiValue(color_code));
                println!("{} {}", etykieta, styled);
            } else {
                println!("{} {}", etykieta, tekst);
            }
        } else {
            println!("{} {}", etykieta, tekst);
        }
    }

    Ok(())
}
