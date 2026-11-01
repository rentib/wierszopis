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

const PALETA_KOLORÓW: &[u8] = &[160, 28, 166, 27, 93, 30, 124, 34, 208, 21, 54, 37, 220];

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

    let mut dsu: Vec<usize> = (0..zapisane.len()).collect();

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
                let mut root_i = i;
                while dsu[root_i] != root_i {
                    root_i = dsu[root_i];
                }

                let mut root_j = j;
                while dsu[root_j] != root_j {
                    root_j = dsu[root_j];
                }

                if root_i != root_j {
                    dsu[root_i] = root_j;
                }

                zapisane[i].koszt = min_koszt;
                zapisane[i].max = best_max;
                zapisane[i].ma_rym = true;

                zapisane[j].koszt = min_koszt;
                zapisane[j].max = best_max;
                zapisane[j].ma_rym = true;
            }
        }

        start = end;
    }

    for i in 0..zapisane.len() {
        if zapisane[i].ma_rym {
            let mut root = i;
            while dsu[root] != root {
                root = dsu[root];
            }
            zapisane[i].kolor = root + 1;
        } else {
            zapisane[i].kolor = 0;
        }
    }

    let k_dist = 6;

    let mut groups_in_order = Vec::new();
    let mut seen_groups = std::collections::HashSet::new();
    for z in &zapisane {
        if z.kolor != 0 && seen_groups.insert(z.kolor) {
            groups_in_order.push(z.kolor);
        }
    }

    let mut adjacencies: std::collections::HashMap<usize, std::collections::HashSet<usize>> =
        std::collections::HashMap::new();

    for i in 0..zapisane.len() {
        let g1 = zapisane[i].kolor;
        if g1 == 0 {
            continue;
        }

        let start_w = i.saturating_sub(k_dist);
        let end_w = std::cmp::min(zapisane.len() - 1, i + k_dist);

        for j in start_w..=end_w {
            if i == j {
                continue;
            }
            let g2 = zapisane[j].kolor;
            if g2 != 0 && g1 != g2 {
                adjacencies.entry(g1).or_default().insert(g2);
                adjacencies.entry(g2).or_default().insert(g1);
            }
        }
    }

    let mut group_to_paleta_idx = std::collections::HashMap::new();
    for &g in &groups_in_order {
        let mut forbidden = std::collections::HashSet::new();
        if let Some(neighbors) = adjacencies.get(&g) {
            for &n in neighbors {
                if let Some(&pal_idx) = group_to_paleta_idx.get(&n) {
                    forbidden.insert(pal_idx);
                }
            }
        }
        let mut color_idx = 0;
        while forbidden.contains(&(color_idx % PALETA_KOLORÓW.len()))
            && color_idx < PALETA_KOLORÓW.len()
        {
            color_idx += 1;
        }
        group_to_paleta_idx.insert(g, color_idx % PALETA_KOLORÓW.len());
    }

    let mut unikalne_kolory: std::collections::HashMap<usize, (usize, usize)> =
        std::collections::HashMap::new();
    for (i, &g) in groups_in_order.iter().enumerate() {
        let num = i + 1;
        let pal_idx = group_to_paleta_idx[&g];
        unikalne_kolory.insert(g, (num, pal_idx));
    }

    let mut total_niepustych = 0;
    let mut max_syl_len = 1;
    let mut max_cost_len = 1;
    let mut max_color_len = 1;
    let mut max_akcent_len = 1;
    let mut max_line_width = 1;

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
        let akcent_len = z.wers.akcent_wersu.len();
        if akcent_len > max_akcent_len {
            max_akcent_len = akcent_len;
        }
        let line_len = z.wers.surowy_tekst.chars().count();
        if line_len > max_line_width {
            max_line_width = line_len;
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

        let akcent_suffix = if szczegoly {
            format!(" [{:<width$}]", z.wers.akcent_wersu, width = max_akcent_len)
        } else {
            "".to_string()
        };

        let color = if z.kolor != 0 {
            if let Some(&(_, paleta_i)) = unikalne_kolory.get(&z.kolor) {
                Some(Color::AnsiValue(PALETA_KOLORÓW[paleta_i]))
            } else {
                None
            }
        } else {
            None
        };

        let rendered_text = renderuj_wers_z_akcentami(&z.wers, color);
        let current_width = z.wers.surowy_tekst.chars().count();
        let padding = if current_width < max_line_width {
            " ".repeat(max_line_width - current_width)
        } else {
            "".to_string()
        };
        println!("{} {}{}{}", etykieta, rendered_text, padding, akcent_suffix);
    }

    Ok(())
}

fn renderuj_wers_z_akcentami(wers: &Wers, color: Option<Color>) -> String {
    let text = &wers.surowy_tekst;
    let mut accented_spans = Vec::new();

    let mut search_start = 0;
    let mut global_syl_idx = 0;
    for sw in &wers.słowa {
        if let Some(word_rel_pos) = text[search_start..].find(&sw.oryginalne) {
            let word_start = search_start + word_rel_pos;
            let spans = sw.sylaba_spans();
            for (_i, &(s_start, s_end)) in spans.iter().enumerate() {
                let akcent_char = wers.akcent_wersu.chars().nth(global_syl_idx).unwrap_or('s');
                global_syl_idx += 1;

                let is_accented = akcent_char == 'S';
                if is_accented {
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
    let mut merged_spans: Vec<(usize, usize)> = Vec::new();
    for span in accented_spans {
        if let Some(last) = merged_spans.last_mut() {
            if span.0 <= last.1 {
                last.1 = last.1.max(span.1);
            } else {
                merged_spans.push(span);
            }
        } else {
            merged_spans.push(span);
        }
    }

    let mut result = String::new();
    let mut last_idx = 0;

    for &(start, end) in &merged_spans {
        if start > last_idx {
            let chunk = &text[last_idx..start];
            result.push_str(&apply_color(chunk, color));
        }
        let accented_chunk = &text[start..end];
        result.push_str(&wyróżnij_akcent(accented_chunk, color));
        last_idx = end;
    }
    if last_idx < text.len() {
        let chunk = &text[last_idx..];
        result.push_str(&apply_color(chunk, color));
    }
    result
}

fn apply_color(s: &str, color: Option<Color>) -> String {
    if let Some(col) = color {
        format!("{}", s.with(col))
    } else {
        s.to_string()
    }
}

fn wyróżnij_akcent(s: &str, color: Option<Color>) -> String {
    if let Some(col) = color {
        format!("{}", s.with(col).underlined())
    } else {
        format!("{}", s.underlined())
    }
}
