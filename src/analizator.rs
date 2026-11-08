/// Odpowiednik internal/analizator w języku Rust.
use crate::wers::{self, Wers};
#[cfg(feature = "tui")]
use crossterm::style::{Color, Stylize};
use std::io::{self, BufRead};

pub struct Analizator;

impl Analizator {
    pub fn nowy() -> Self {
        Analizator
    }
}

#[cfg(feature = "tui")]
const PALETA_KOLORÓW: &[u8] = &[160, 28, 166, 27, 93, 30, 124, 34, 208, 21, 54, 37, 220];

#[derive(Clone, Debug, PartialEq)]
pub struct ZapisWersu {
    pub linenr: usize,
    pub kolor: usize,
    pub koszt: i32,
    pub max: i32,
    pub wers: Wers,
    pub ma_rym: bool,
}

pub struct AnalizaWiersza {
    pub wersy: Vec<ZapisWersu>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RodzajSonetu {
    Szekspirowski,
    Spenserowski,
    Barnifieldowski,
    WłoskiPetrarkowski,
    FrancuskiMarotyczny,
    InnySonet,
}

pub fn usun_podpis(wersy: &mut Vec<Wers>) {
    while let Some(w) = wersy.last() {
        if w.oryginalne.is_empty() {
            wersy.pop();
        } else {
            break;
        }
    }

    if wersy.is_empty() {
        return;
    }

    let mut idx = wersy.len() - 1;
    let mut stanza_start = idx;
    let mut stanza_len = 0;

    while idx < wersy.len() {
        if wersy[idx].oryginalne.is_empty() {
            break;
        }
        stanza_start = idx;
        stanza_len += 1;
        if idx == 0 {
            break;
        }
        idx -= 1;
    }

    if stanza_len == 1 {
        let text = wersy[stanza_start].surowy_tekst.trim_start();
        if text.starts_with("--") {
            wersy.truncate(stanza_start);
            while let Some(w) = wersy.last() {
                if w.oryginalne.is_empty() {
                    wersy.pop();
                } else {
                    break;
                }
            }
        }
    }
}

impl AnalizaWiersza {
    pub fn nowy(mut wersy_bazowe: Vec<Wers>) -> Self {
        usun_podpis(&mut wersy_bazowe);
        let mut zapisane = Vec::new();
        for (idx, w) in wersy_bazowe.into_iter().enumerate() {
            zapisane.push(ZapisWersu {
                linenr: idx + 1,
                kolor: 0,
                koszt: 0,
                max: 0,
                wers: w,
                ma_rym: false,
            });
        }
        AnalizaWiersza { wersy: zapisane }
    }

    pub fn schemat_rymów(&self) -> (String, Vec<usize>) {
        let mut schemat = String::new();
        let mut mapowanie_kolorów: std::collections::HashMap<usize, char> = std::collections::HashMap::new();
        let mut następna_litera = b'a';
        let mut numery_grup = Vec::new();

        for z in &self.wersy {
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

    pub fn analizuj_sonet(&self) -> Option<RodzajSonetu> {
        let (_schemat, grupy) = self.schemat_rymów();
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

    pub fn analizuj_rymy(&mut self) {
        let mut dsu: Vec<usize> = (0..self.wersy.len()).collect();

        for i in 0..self.wersy.len() {
            if self.wersy[i].wers.oryginalne.is_empty() {
                continue;
            }
            let a_wers = &self.wersy[i].wers;
            if a_wers.rym_meski.is_empty() && a_wers.rym_zenski.is_empty() {
                continue;
            }

            let mut best_j = None;
            let mut min_koszt = i32::MAX;
            let mut best_max = 0;

            for j in 0..i {
                if self.wersy[j].wers.oryginalne.is_empty() {
                    continue;
                }
                let b_wers = &self.wersy[j].wers;
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

                self.wersy[i].koszt = min_koszt;
                self.wersy[i].max = best_max;
                self.wersy[i].ma_rym = true;

                self.wersy[j].koszt = min_koszt;
                self.wersy[j].max = best_max;
                self.wersy[j].ma_rym = true;
            }
        }

        for i in 0..self.wersy.len() {
            if self.wersy[i].ma_rym {
                let mut root = i;
                while dsu[root] != root {
                    root = dsu[root];
                }
                self.wersy[i].kolor = root + 1;
            } else {
                self.wersy[i].kolor = 0;
            }
        }
    }

    pub fn optymalizuj_średniówki(&mut self) {
        let mut non_empty_indices = Vec::new();
        for (i, z) in self.wersy.iter().enumerate() {
            if !z.wers.oryginalne.is_empty() {
                non_empty_indices.push(i);
            }
        }

        if non_empty_indices.is_empty() {
            return;
        }

        let mut poem_candidates: Vec<Vec<(String, i32)>> = Vec::new();
        for &i in &non_empty_indices {
            let w = &self.wersy[i].wers;
            let cands = wers::kandydaci_średniówki(&w.słowa, w.liczba_sylab);
            poem_candidates.push(cands);
        }

        let mut current_best: Vec<String> = poem_candidates.iter().map(|cands| {
            if let Some((best_label, _)) = cands.iter().max_by_key(|&(_, score)| score) {
                best_label.clone()
            } else {
                "-".to_string()
            }
        }).collect();

        for _iter in 0..2 {
            let mut next_best = current_best.clone();
            for (idx, cands) in poem_candidates.iter().enumerate() {
                if cands.is_empty() {
                    continue;
                }
                let mut best_label = current_best[idx].clone();
                let mut max_total_score = i32::MIN;

                for (label, base_score) in cands {
                    let mut total_score = *base_score;

                    for n_offset in -2..=2 {
                        if n_offset == 0 { continue; }
                        let n_idx = idx as i32 + n_offset;
                        if n_idx >= 0 && (n_idx as usize) < current_best.len() {
                            let neighbor_label = &current_best[n_idx as usize];
                            if neighbor_label != "-" {
                                let distance = n_offset.abs();
                                let weight = match distance {
                                    1 => 25,
                                    2 => 15,
                                    _ => 10,
                                };
                                if label == neighbor_label {
                                    total_score += weight;
                                } else {
                                    total_score -= weight / 2;
                                }
                            }
                        }
                    }

                    if total_score > max_total_score {
                        max_total_score = total_score;
                        best_label = label.clone();
                    }
                }
                next_best[idx] = best_label;
            }
            current_best = next_best;
        }

        for (idx, label) in current_best.into_iter().enumerate() {
            let real_i = non_empty_indices[idx];
            self.wersy[real_i].wers.średniówka = label;
        }
    }

    #[cfg(feature = "tui")]
    pub fn wyznacz_kolory(&self, k_dist: usize) -> (std::collections::HashMap<usize, (usize, usize)>, Vec<usize>) {
        let mut groups_in_order = Vec::new();
        let mut seen_groups = std::collections::HashSet::new();
        for z in &self.wersy {
            if z.kolor != 0 && seen_groups.insert(z.kolor) {
                groups_in_order.push(z.kolor);
            }
        }

        let mut adjacencies: std::collections::HashMap<usize, std::collections::HashSet<usize>> =
            std::collections::HashMap::new();

        for i in 0..self.wersy.len() {
            let g1 = self.wersy[i].kolor;
            if g1 == 0 {
                continue;
            }

            let start_w = i.saturating_sub(k_dist);
            let end_w = std::cmp::min(self.wersy.len() - 1, i + k_dist);

            for j in start_w..=end_w {
                if i == j {
                    continue;
                }
                let g2 = self.wersy[j].kolor;
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

        let mut unikalne_kolory = std::collections::HashMap::new();
        for (i, &g) in groups_in_order.iter().enumerate() {
            let num = i + 1;
            let pal_idx = group_to_paleta_idx[&g];
            unikalne_kolory.insert(g, (num, pal_idx));
        }

        (unikalne_kolory, groups_in_order)
    }

    #[cfg(feature = "tui")]
    pub fn renderuj(&self, szczegoly: bool) {
        if szczegoly {
            if let Some(sonet) = self.analizuj_sonet() {
                let nazwa = match sonet {
                    RodzajSonetu::Szekspirowski => "sonet szekspirowski (elżbietański)",
                    RodzajSonetu::Spenserowski => "sonet spenserowski",
                    RodzajSonetu::Barnifieldowski => "sonet barnifieldowski",
                    RodzajSonetu::WłoskiPetrarkowski => "sonet włoski (petrarkowski)",
                    RodzajSonetu::FrancuskiMarotyczny => "sonet francuski (marotyczny)",
                    RodzajSonetu::InnySonet => "inny rodzaj sonetu",
                };
                println!("Rozpoznano rodzaj wiersza: {}\n", nazwa);
            }
        }

        let (unikalne_kolory, _) = self.wyznacz_kolory(6);

        let mut total_niepustych = 0;
        let mut max_syl_len = 1;
        let mut max_cost_len = 1;
        let mut max_color_len = 1;
        let mut max_akcent_len = 1;
        let mut max_line_width = 1;

        for z in &self.wersy {
            if z.wers.oryginalne.is_empty() {
                continue;
            }
            total_niepustych += 1;
            let syl_content = if szczegoly && z.wers.średniówka != "-" {
                format!("{} ({})", z.wers.liczba_sylab, z.wers.średniówka)
            } else {
                z.wers.liczba_sylab.to_string()
            };
            if syl_content.chars().count() > max_syl_len {
                max_syl_len = syl_content.chars().count();
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
        for z in &self.wersy {
            if z.wers.oryginalne.is_empty() {
                println!();
                continue;
            }
            numer_niepustego += 1;

            let nr_str = format!("{:>width$}", numer_niepustego, width = max_nr_len);
            let syl_content = if szczegoly && z.wers.średniówka != "-" {
                format!("{} ({})", z.wers.liczba_sylab, z.wers.średniówka)
            } else {
                z.wers.liczba_sylab.to_string()
            };
            let syl_str = format!("{:>width$}", syl_content, width = max_syl_len);

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
    }
}

#[cfg(feature = "tui")]
pub fn analizuj_strumien(reader: impl io::Read, szczegoly: bool) -> io::Result<()> {
    let buf_reader = io::BufReader::new(reader);
    let mut wersy = Vec::new();

    for line in buf_reader.lines() {
        let line = line?;
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }

        let mut comment_idx = None;
        if let Some(idx) = line.find("//") {
            comment_idx = Some(idx);
        }
        if let Some(idx) = line.find('#') {
            comment_idx = match comment_idx {
                Some(existing) => Some(std::cmp::min(existing, idx)),
                None => Some(idx),
            };
        }

        let clean_line = if let Some(idx) = comment_idx {
            &line[..idx]
        } else {
            &line
        };

        wersy.push(wers::nowy_wers(clean_line));
    }

    let mut analiza = AnalizaWiersza::nowy(wersy);
    analiza.analizuj_rymy();
    analiza.optymalizuj_średniówki();
    analiza.renderuj(szczegoly);

    Ok(())
}

#[cfg(feature = "tui")]
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

#[cfg(feature = "tui")]
fn apply_color(s: &str, color: Option<Color>) -> String {
    if let Some(col) = color {
        format!("{}", s.with(col))
    } else {
        s.to_string()
    }
}

#[cfg(feature = "tui")]
fn wyróżnij_akcent(s: &str, color: Option<Color>) -> String {
    if let Some(col) = color {
        format!("{}", s.with(col).underlined())
    } else {
        format!("{}", s.underlined())
    }
}

#[cfg(all(test, feature = "tui"))]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_analizuj_strumien_komentarze() {
        let input = "# Komentarz python\n// Komentarz C\n   # Komentarz z wcięciem\nLitwo! Ojczyzno moja! # epopeja\nA wieża ciśnień // komentarz inline\n";
        let cursor = Cursor::new(input.as_bytes());
        let result = analizuj_strumien(cursor, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analizuj_strumien_cross_stanza_rym() {
        let input = "a ja na to\n\njak na lato\n";
        let cursor = Cursor::new(input.as_bytes());
        let result = analizuj_strumien(cursor, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analiza_wiersza_metody() {
        let w1 = wers::nowy_wers("a ja na to");
        let w2 = wers::nowy_wers("jak na lato");
        let mut analiza = AnalizaWiersza::nowy(vec![w1, w2]);
        analiza.analizuj_rymy();
        analiza.optymalizuj_średniówki();
        assert!(analiza.wersy[0].ma_rym);
        assert!(analiza.wersy[1].ma_rym);
        assert_eq!(analiza.wersy[0].kolor, analiza.wersy[1].kolor);
    }

    #[test]
    fn test_rozpoznawanie_sonetow() {
        let make_analiza = |kolory: Vec<usize>| {
            let wersy = kolory.into_iter().map(|k| {
                let w = wers::nowy_wers("test");
                ZapisWersu {
                    linenr: 1,
                    kolor: k,
                    koszt: 0,
                    max: 0,
                    wers: w,
                    ma_rym: true,
                }
            }).collect();
            AnalizaWiersza { wersy }
        };

        // 1. Szekspirowski: abab cdcd efef gg
        let a_szekspir = make_analiza(vec![1, 2, 1, 2, 3, 4, 3, 4, 5, 6, 5, 6, 7, 7]);
        assert_eq!(a_szekspir.analizuj_sonet(), Some(RodzajSonetu::Szekspirowski));

        // 2. Spenserowski: abab bcbc cdcd ee
        let a_spenser = make_analiza(vec![1, 2, 1, 2, 2, 3, 2, 3, 3, 4, 3, 4, 5, 5]);
        assert_eq!(a_spenser.analizuj_sonet(), Some(RodzajSonetu::Spenserowski));

        // 3. Barnifieldowski: abba cddc effe gg
        let a_barnifield = make_analiza(vec![1, 2, 2, 1, 3, 4, 4, 3, 5, 6, 6, 5, 7, 7]);
        assert_eq!(a_barnifield.analizuj_sonet(), Some(RodzajSonetu::Barnifieldowski));

        // 4. Włoski (Petrarkowski): abba abba cdc dcd
        let a_włoski = make_analiza(vec![1, 2, 2, 1, 1, 2, 2, 1, 3, 4, 3, 4, 3, 4]);
        assert_eq!(a_włoski.analizuj_sonet(), Some(RodzajSonetu::WłoskiPetrarkowski));

        // 5. Francuski (Marotyczny): abba abba cc dede
        let a_francuski = make_analiza(vec![1, 2, 2, 1, 1, 2, 2, 1, 3, 3, 4, 5, 4, 5]);
        assert_eq!(a_francuski.analizuj_sonet(), Some(RodzajSonetu::FrancuskiMarotyczny));
    }

    #[test]
    fn test_usun_podpis() {
        let mut w = vec![
            wers::nowy_wers("Pierwszy wers"),
            wers::nowy_wers("Drugi wers"),
            wers::nowy_wers(""),
            wers::nowy_wers(""),
            wers::nowy_wers("-- Jan Kowalski"),
        ];
        usun_podpis(&mut w);
        assert_eq!(w.len(), 2);
        assert_eq!(w[0].surowy_tekst, "Pierwszy wers");
        assert_eq!(w[1].surowy_tekst, "Drugi wers");
    }

    #[test]
    fn test_analizuj_strumien_z_podpisem() {
        let input = "Wers pierwszy\nWers drugi\n\n-- Autor";
        let cursor = Cursor::new(input.as_bytes());
        let result = analizuj_strumien(cursor, false);
        assert!(result.is_ok());
    }
}
