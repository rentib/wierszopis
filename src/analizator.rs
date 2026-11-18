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

pub use crate::gatunki::{GatunekLiteracki, RodzajRondeau, RodzajSonetu};

pub use wers::{is_lekka_modyfikacja_wersu, usun_podpis};

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
        crate::gatunki::schemat_rymów(&self.wersy)
    }

    pub fn get_stanzas(&self) -> Vec<Vec<&ZapisWersu>> {
        crate::gatunki::get_stanzas(&self.wersy)
    }

    pub fn analizuj_sonet(&self) -> Option<RodzajSonetu> {
        crate::gatunki::analizuj_sonet(&self.wersy)
    }

    pub fn analizuj_vilanelle(&self) -> Option<()> {
        crate::gatunki::analizuj_vilanelle(&self.wersy)
    }

    pub fn analizuj_pantum(&self) -> Option<()> {
        crate::gatunki::analizuj_pantum(&self.wersy)
    }

    pub fn analizuj_triolet(&self) -> Option<()> {
        crate::gatunki::analizuj_triolet(&self.wersy)
    }

    pub fn analizuj_rondeau(&self) -> Option<RodzajRondeau> {
        crate::gatunki::analizuj_rondeau(&self.wersy)
    }

    pub fn analizuj_limeryk(&self) -> Option<()> {
        crate::gatunki::analizuj_limeryk(&self.wersy)
    }

    pub fn gatunek_literacki(&self) -> Option<GatunekLiteracki> {
        crate::gatunki::gatunek_literacki(&self.wersy)
    }

    pub fn rodzaj_wiersza(&self) -> Option<GatunekLiteracki> {
        self.gatunek_literacki()
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

            let mut best_j_w = None;
            let mut min_koszt_w = i32::MAX;
            let mut best_max_w = 0;

            let mut best_j_spoza = None;
            let mut min_koszt_spoza = i32::MAX;
            let mut best_max_spoza = 0;

            for j in 0..i {
                if self.wersy[j].wers.oryginalne.is_empty() {
                    continue;
                }
                let b_wers = &self.wersy[j].wers;
                if b_wers.rym_meski.is_empty() && b_wers.rym_zenski.is_empty() {
                    continue;
                }

                let (ok, koszt, max_k) = wers::czy_rymują(b_wers, a_wers);
                if !ok {
                    continue;
                }

                // Check if j is in the same stanza as i
                let mut same_stanza = true;
                for k in (j + 1)..i {
                    if self.wersy[k].wers.oryginalne.is_empty() {
                        same_stanza = false;
                        break;
                    }
                }

                if same_stanza {
                    if koszt < min_koszt_w {
                        min_koszt_w = koszt;
                        best_max_w = max_k;
                        best_j_w = Some(j);
                    }
                } else {
                    if koszt < min_koszt_spoza {
                        min_koszt_spoza = koszt;
                        best_max_spoza = max_k;
                        best_j_spoza = Some(j);
                    }
                }
            }

            let (best_j, min_koszt, best_max) = match (best_j_w, best_j_spoza) {
                (Some(jw), Some(js)) => {
                    // jedyna sytuacja, gdzie rym spoza strofy powinien być wybierany zamiast rymu ze strofy
                    // jest taka, że ten spoza jest dokładny (koszt == 0), a ten w jest niedokładny (koszt > 0)
                    if min_koszt_spoza == 0 && min_koszt_w > 0 {
                        (Some(js), min_koszt_spoza, best_max_spoza)
                    } else {
                        (Some(jw), min_koszt_w, best_max_w)
                    }
                }
                (Some(jw), None) => (Some(jw), min_koszt_w, best_max_w),
                (None, Some(js)) => (Some(js), min_koszt_spoza, best_max_spoza),
                (None, None) => (None, i32::MAX, 0),
            };

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

        let mut current_best: Vec<String> = poem_candidates
            .iter()
            .map(|cands| {
                if let Some((best_label, _)) = cands.iter().max_by_key(|&(_, score)| score) {
                    best_label.clone()
                } else {
                    "-".to_string()
                }
            })
            .collect();

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
                        if n_offset == 0 {
                            continue;
                        }
                        let n_idx = idx as i32 + n_offset;
                        if n_idx >= 0 && (n_idx as usize) < current_best.len() {
                            let n_real_idx = non_empty_indices[n_idx as usize];
                            let current_real_idx = non_empty_indices[idx];
                            if self.wersy[n_real_idx].wers.liczba_sylab
                                == self.wersy[current_real_idx].wers.liczba_sylab
                            {
                                let neighbor_label = &current_best[n_idx as usize];
                                if neighbor_label != "-" {
                                    let distance = n_offset.abs();
                                    let weight = match distance {
                                        1 => 3,
                                        2 => 2,
                                        _ => 1,
                                    };
                                    if label == neighbor_label {
                                        total_score += weight;
                                    } else {
                                        total_score -= weight / 2;
                                    }
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
    pub fn wyznacz_kolory(
        &self,
        k_dist: usize,
    ) -> (std::collections::HashMap<usize, (usize, usize)>, Vec<usize>) {
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
        if let Some(rodzaj) = self.rodzaj_wiersza() {
            println!("Gatunek literacki: {}\n", rodzaj.nazwa());
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

        if szczegoly {
            let (schemat_globalny, _) = crate::gatunki::schemat_rymow_filtrowany(&self.wersy);
            println!("\nSchemat rymów: {}", schemat_globalny);

            let sylaby: Vec<usize> = self.wersy.iter()
                .filter(|z| !z.wers.oryginalne.is_empty())
                .map(|z| z.wers.liczba_sylab)
                .collect();
            if let Some(okres) = crate::gatunki::wyznacz_okres_sylab(&sylaby) {
                let okres_str = okres.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ");
                println!("Wzorzec sylab (okres): [{}]", okres_str);
            }
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
    fn test_rym_priorytet_strofy() {
        let w0 = wers::nowy_wers("dom");
        let w_pusty = wers::nowy_wers("");
        let w1 = wers::nowy_wers("grom");
        let w2 = wers::nowy_wers("tom");
        let mut analiza = AnalizaWiersza::nowy(vec![w0, w_pusty, w1, w2]);
        analiza.analizuj_rymy();
        assert!(analiza.wersy[2].ma_rym);
        assert!(analiza.wersy[3].ma_rym);
        assert_eq!(analiza.wersy[2].kolor, analiza.wersy[3].kolor);
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
            let wersy = kolory
                .into_iter()
                .map(|k| {
                    let w = wers::nowy_wers("test");
                    ZapisWersu {
                        linenr: 1,
                        kolor: k,
                        koszt: 0,
                        max: 0,
                        wers: w,
                        ma_rym: true,
                    }
                })
                .collect();
            AnalizaWiersza { wersy }
        };

        // 1. Szekspirowski: abab cdcd efef gg
        let a_szekspir = make_analiza(vec![1, 2, 1, 2, 3, 4, 3, 4, 5, 6, 5, 6, 7, 7]);
        assert_eq!(
            a_szekspir.analizuj_sonet(),
            Some(RodzajSonetu::Szekspirowski)
        );

        // 2. Spenserowski: abab bcbc cdcd ee
        let a_spenser = make_analiza(vec![1, 2, 1, 2, 2, 3, 2, 3, 3, 4, 3, 4, 5, 5]);
        assert_eq!(a_spenser.analizuj_sonet(), Some(RodzajSonetu::Spenserowski));

        // 3. Barnifieldowski: abba cddc effe gg
        let a_barnifield = make_analiza(vec![1, 2, 2, 1, 3, 4, 4, 3, 5, 6, 6, 5, 7, 7]);
        assert_eq!(
            a_barnifield.analizuj_sonet(),
            Some(RodzajSonetu::Barnifieldowski)
        );

        // 4. Włoski (Petrarkowski): abba abba cdc dcd
        let a_włoski = make_analiza(vec![1, 2, 2, 1, 1, 2, 2, 1, 3, 4, 3, 4, 3, 4]);
        assert_eq!(
            a_włoski.analizuj_sonet(),
            Some(RodzajSonetu::WłoskiPetrarkowski)
        );

        // 5. Francuski (Marotyczny): abba abba cc dede
        let a_francuski = make_analiza(vec![1, 2, 2, 1, 1, 2, 2, 1, 3, 3, 4, 5, 4, 5]);
        assert_eq!(
            a_francuski.analizuj_sonet(),
            Some(RodzajSonetu::FrancuskiMarotyczny)
        );
    }

    #[test]
    fn test_rozpoznawanie_vilanelli() {
        let mut wersy = Vec::new();
        let r1 = "Refren pierwszy";
        let r2 = "Refren drugi";
        let other = "Inny wers";

        wersy.push(wers::nowy_wers(r1));
        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(r2));
        wersy.push(wers::nowy_wers(""));

        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(r1));
        wersy.push(wers::nowy_wers(""));

        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(r2));
        wersy.push(wers::nowy_wers(""));

        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(r1));
        wersy.push(wers::nowy_wers(""));

        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(r2));
        wersy.push(wers::nowy_wers(""));

        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(r1));
        wersy.push(wers::nowy_wers(r2));

        let analiza = AnalizaWiersza::nowy(wersy);
        assert_eq!(
            analiza.gatunek_literacki(),
            Some(GatunekLiteracki::Vilanella)
        );
    }

    #[test]
    fn test_rozpoznawanie_vilanelli_niezalezne_grupy() {
        let mut wersy = Vec::new();
        let r1 = "To jest długi refren pierwszy";
        let r2 = "Krótki drugi";
        let other = "Jakiś zupełnie inny wers w środku";

        wersy.push(wers::nowy_wers(r1));
        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(r2));
        wersy.push(wers::nowy_wers(""));

        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(r1));
        wersy.push(wers::nowy_wers(""));

        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(r2));
        wersy.push(wers::nowy_wers(""));

        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(r1));
        wersy.push(wers::nowy_wers(""));

        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(r2));
        wersy.push(wers::nowy_wers(""));

        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(other));
        wersy.push(wers::nowy_wers(r1));
        wersy.push(wers::nowy_wers(r2));

        let analiza = AnalizaWiersza::nowy(wersy);
        assert_eq!(
            analiza.gatunek_literacki(),
            Some(GatunekLiteracki::Vilanella)
        );
    }

    #[test]
    fn test_rozpoznawanie_nowych_gatunkow() {
        // Pantum: 2 stanzas of 4 lines with repeated refrains
        let mut wersy_pantum = Vec::new();
        wersy_pantum.push(wers::nowy_wers("A"));
        wersy_pantum.push(wers::nowy_wers("B"));
        wersy_pantum.push(wers::nowy_wers("C"));
        wersy_pantum.push(wers::nowy_wers("D"));
        wersy_pantum.push(wers::nowy_wers(""));
        wersy_pantum.push(wers::nowy_wers("B"));
        wersy_pantum.push(wers::nowy_wers("X"));
        wersy_pantum.push(wers::nowy_wers("D"));
        wersy_pantum.push(wers::nowy_wers("Y"));
        let analiza_pantum = AnalizaWiersza::nowy(wersy_pantum);
        assert_eq!(
            analiza_pantum.gatunek_literacki(),
            Some(GatunekLiteracki::Pantum)
        );
    }

    #[test]
    fn test_rozpoznawanie_trioletu() {
        let wersy = vec![
            wers::nowy_wers("To jest piękny dom"),
            wers::nowy_wers("Szumi wielki las"),
            wers::nowy_wers("Gdzie rośnie stary grom"),
            wers::nowy_wers("To jest piękny dom"),
            wers::nowy_wers("I leży zimny złom"),
            wers::nowy_wers("Mijają szybko czas"),
            wers::nowy_wers("To jest piękny dom"),
            wers::nowy_wers("Szumi wielki las"),
        ];
        let analiza = AnalizaWiersza::nowy(wersy);
        assert_eq!(analiza.gatunek_literacki(), Some(GatunekLiteracki::Triolet));
    }

    #[test]
    fn test_rozpoznawanie_trioletu_z_modyfikacja() {
        let wersy = vec![
            wers::nowy_wers("To jest piękny dom"),
            wers::nowy_wers("Szumi wielki las"),
            wers::nowy_wers("Gdzie rośnie stary grom"),
            wers::nowy_wers("To jest cudny dom"), // lekka modyfikacja refrenu 1
            wers::nowy_wers("I leży zimny złom"),
            wers::nowy_wers("Mijają szybko czas"),
            wers::nowy_wers("To jest piękny dom"),
            wers::nowy_wers("Szumi cichy las"), // lekka modyfikacja refrenu 2
        ];
        let analiza = AnalizaWiersza::nowy(wersy);
        assert_eq!(analiza.gatunek_literacki(), Some(GatunekLiteracki::Triolet));
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

    #[test]
    fn test_parse_directive_sylaby_comma_and_brackets() {
        let res1 = parse_directive("# wierszopis: sylaby=12,12,7,7,8");
        assert!(res1.is_some());
        let cfg1 = res1.unwrap().unwrap();
        assert_eq!(cfg1.sylaby_pattern, Some(vec![12, 12, 7, 7, 8]));

        let res2 = parse_directive("# wierszopis: sylaby=[12, 12, 7, 7, 8]; akcenty = tak");
        assert!(res2.is_some());
        let cfg2 = res2.unwrap().unwrap();
        assert_eq!(cfg2.sylaby_pattern, Some(vec![12, 12, 7, 7, 8]));
        assert!(cfg2.akcenty_enabled);

        let res3 = parse_directive("# wierszopis: akcenty=nie");
        assert!(res3.is_some());
        let cfg3 = res3.unwrap().unwrap();
        assert!(!cfg3.akcenty_enabled);
    }

    #[test]
    fn test_parse_directive_mixed_sylaby() {
        let res = parse_directive("# wierszopis: sylaby=[6+6,6+6,7,7,8]");
        assert!(res.is_some());
        let cfg = res.unwrap().unwrap();
        assert_eq!(cfg.sylaby_pattern, Some(vec![12, 12, 7, 7, 8]));
        assert_eq!(
            cfg.expected_sredniowka_pattern,
            Some(vec![
                "6+6".to_string(),
                "6+6".to_string(),
                "-".to_string(),
                "-".to_string(),
                "-".to_string()
            ])
        );
    }

    #[test]
    fn test_parse_directive_invalid() {
        let res = parse_directive("# wierszopis: sylaby=abc");
        assert!(res.is_some());
        assert!(res.unwrap().is_err());
    }
}

#[derive(Debug, Clone, Default)]
pub struct StanzaConfig {
    pub sylaby_pattern: Option<Vec<usize>>,
    pub expected_sredniowka_pattern: Option<Vec<String>>,
    pub rymy_schemat: Option<String>,
    pub akcenty_enabled: bool,
}

pub fn parse_directive(line: &str) -> Option<Result<StanzaConfig, String>> {
    let trimmed = line.trim_start();
    let content = if let Some(stripped) = trimmed.strip_prefix("//") {
        stripped.trim()
    } else if let Some(stripped) = trimmed.strip_prefix('#') {
        stripped.trim()
    } else {
        return None;
    };

    let content_lower = content.to_lowercase();
    if !content_lower.starts_with("wierszopis:") && !content_lower.starts_with("@wierszopis:") {
        return None;
    }

    let directive_part = if let Some(idx) = content.find(':') {
        &content[idx + 1..]
    } else {
        return Some(Err("Brak dwukropka po prefiksie dyrektywy".to_string()));
    };

    let mut config = StanzaConfig::default();

    for part in directive_part.split(|c| c == ';' || c == '\n' || c == '\r') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let kv: Vec<&str> = part.splitn(2, '=').map(|s| s.trim()).collect();
        if kv.len() != 2 {
            return Some(Err(format!("Niepoprawna para klucz=wartość: '{}'", part)));
        }
        let key = kv[0].to_lowercase();
        let val = kv[1];

        if key == "sylaby" {
            let val_clean = if val.starts_with('[') && val.ends_with(']') {
                val[1..val.len() - 1].trim()
            } else {
                val.trim()
            };

            let mut syl_pattern = vec![];
            let mut sred_pattern = vec![];

            for item in val_clean.split(',') {
                let item = item.trim();
                if item.is_empty() {
                    continue;
                }

                if item.contains('+') {
                    let parts: Vec<&str> = item.split('+').collect();
                    if parts.len() == 2 {
                        match (
                            parts[0].trim().parse::<usize>(),
                            parts[1].trim().parse::<usize>(),
                        ) {
                            (Ok(p1), Ok(p2)) => {
                                syl_pattern.push(p1 + p2);
                                sred_pattern.push(item.to_string());
                            }
                            _ => {
                                return Some(Err(format!(
                                    "Niepoprawne wartości średniówki: '{}'",
                                    item
                                )));
                            }
                        }
                    } else {
                        return Some(Err(format!("Niepoprawny format średniówki: '{}'", item)));
                    }
                } else {
                    match item.parse::<usize>() {
                        Ok(n) => {
                            syl_pattern.push(n);
                            sred_pattern.push("-".to_string());
                        }
                        Err(_) => {
                            return Some(Err(format!("Niepoprawna liczba sylab: '{}'", item)));
                        }
                    }
                }
            }

            if !syl_pattern.is_empty() {
                config.sylaby_pattern = Some(syl_pattern);
                config.expected_sredniowka_pattern = Some(sred_pattern);
            } else {
                return Some(Err("Pusta lista sylab".to_string()));
            }
        } else if key == "rymy" {
            let val_clean = if val.starts_with('[') && val.ends_with(']') {
                val[1..val.len() - 1].trim()
            } else {
                val.trim()
            };
            let val_lower = val_clean.to_string().to_lowercase();
            if !val_lower.is_empty() {
                config.rymy_schemat = Some(val_lower);
            } else {
                return Some(Err("Pusty schemat rymów".to_string()));
            }
        } else if key == "akcenty" {
            let val_lower = val.trim().to_lowercase();
            match val_lower.as_str() {
                "tak" => config.akcenty_enabled = true,
                "nie" => config.akcenty_enabled = false,
                _ => {
                    return Some(Err(format!(
                        "Niepoprawna wartość dla opcji akcenty (oczekiwano tak/nie): '{}'",
                        val
                    )));
                }
            }
        } else {
            return Some(Err(format!("Nieznany parametr dyrektywy: '{}'", key)));
        }
    }

    Some(Ok(config))
}

pub fn analyze_document(
    text: &str,
) -> (
    std::collections::HashMap<usize, usize>,
    std::collections::HashMap<usize, usize>,
    Vec<Option<Wers>>,
    Vec<usize>,
    Vec<usize>,
) {
    let lines: Vec<&str> = text.lines().collect();
    let mut raw_wersy_with_line = vec![];
    let mut comment_lines = vec![];
    let mut signature_lines = vec![];
    let mut _current_config = StanzaConfig::default();

    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with('#') {
            if let Some(Ok(cfg)) = parse_directive(line) {
                _current_config = cfg;
            }
            comment_lines.push(line_idx);
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

        let trimmed_clean = clean_line.trim();
        if trimmed_clean.is_empty() {
            raw_wersy_with_line.push((line_idx, wers::nowy_wers("")));
        } else {
            raw_wersy_with_line.push((line_idx, wers::nowy_wers(trimmed_clean)));
        }
    }

    let mut just_wersy: Vec<Wers> = raw_wersy_with_line.iter().map(|(_, w)| w.clone()).collect();
    usun_podpis(&mut just_wersy);

    let valid_count = just_wersy.len();
    let mut wersy = vec![None; lines.len()];
    let mut active_wersy_with_idx = vec![];

    for i in 0..valid_count {
        let (line_idx, _) = raw_wersy_with_line[i];
        let w = just_wersy[i].clone();
        if !w.oryginalne.is_empty() {
            wersy[line_idx] = Some(w.clone());
            active_wersy_with_idx.push((line_idx, w));
        }
    }

    for i in valid_count..raw_wersy_with_line.len() {
        let (line_idx, w) = &raw_wersy_with_line[i];
        if !w.oryginalne.is_empty() {
            signature_lines.push(*line_idx);
        }
    }

    let mut group_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut group_sizes: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut next_group = 0;

    for i in 0..active_wersy_with_idx.len() {
        let (line_i, w_i) = &active_wersy_with_idx[i];
        if group_map.contains_key(line_i) {
            continue;
        }
        let mut members = vec![*line_i];

        for j in (i + 1)..active_wersy_with_idx.len() {
            let (line_j, w_j) = &active_wersy_with_idx[j];
            if !group_map.contains_key(line_j) {
                let (rymuja, _, _) = wers::czy_rymują(w_i, w_j);
                if rymuja {
                    members.push(*line_j);
                }
            }
        }

        if members.len() >= 2 {
            for m in &members {
                group_map.insert(*m, next_group);
            }
            group_sizes.insert(next_group, members.len());
            next_group += 1;
        }
    }

    (
        group_map,
        group_sizes,
        wersy,
        comment_lines,
        signature_lines,
    )
}

pub use wers::get_accented_spans_for_line;

#[cfg(feature = "lsp")]
pub fn compute_diagnostics(text: &str) -> Vec<tower_lsp::lsp_types::Diagnostic> {
    use tower_lsp::lsp_types::*;
    let lines: Vec<&str> = text.lines().collect();
    let mut diagnostics = vec![];
    let (_group_map, _group_sizes, wersy, comment_lines, _signature_lines) = analyze_document(text);

    let mut current_config = StanzaConfig::default();
    let mut verse_in_block_idx = 0;

    for (line_idx, line) in lines.iter().enumerate() {
        if comment_lines.contains(&line_idx) {
            match parse_directive(line) {
                Some(Ok(cfg)) => {
                    current_config = cfg;
                    verse_in_block_idx = 0;
                }
                Some(Err(err_msg)) => {
                    let chars: Vec<char> = line.chars().collect();
                    let start = chars.iter().position(|c| !c.is_whitespace()).unwrap_or(0);
                    let end = chars
                        .iter()
                        .rposition(|c| !c.is_whitespace())
                        .map(|i| i + 1)
                        .unwrap_or(chars.len());
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: line_idx as u32,
                                character: start as u32,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: end as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::WARNING),
                        code: Some(NumberOrString::String("directive-error".to_string())),
                        source: Some("wierszopis-lsp".to_string()),
                        message: format!("Błąd parsowania dyrektywy: {}", err_msg),
                        ..Default::default()
                    });
                }
                None => {}
            }
            continue;
        }

        let Some(ref w) = wersy[line_idx] else {
            if line.trim().is_empty() {
                verse_in_block_idx = 0;
            }
            continue;
        };

        if w.oryginalne.is_empty() {
            verse_in_block_idx = 0;
            continue;
        }

        let chars: Vec<char> = line.chars().collect();
        let start = chars.iter().position(|c| !c.is_whitespace()).unwrap_or(0);
        let end = chars
            .iter()
            .rposition(|c| !c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(chars.len());

        if let Some(ref pattern) = current_config.sylaby_pattern {
            let expected = pattern[verse_in_block_idx % pattern.len()];
            let actual = w.liczba_sylab;
            if actual != expected {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: start as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: end as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String("sylaby-mismatch".to_string())),
                    source: Some("wierszopis-lsp".to_string()),
                    message: format!("Oczekiwano {} sylab, wykryto {} sylab.", expected, actual),
                    ..Default::default()
                });
            }
        }

        if let Some(ref expected_sred_pattern) = current_config.expected_sredniowka_pattern {
            let expected_sred =
                &expected_sred_pattern[verse_in_block_idx % expected_sred_pattern.len()];
            if expected_sred != "-" && w.średniówka != *expected_sred {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: start as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: end as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String("średniówka-mismatch".to_string())),
                    source: Some("wierszopis-lsp".to_string()),
                    message: format!(
                        "Oczekiwano średniówki '{}', wykryto '{}'.",
                        expected_sred, w.średniówka
                    ),
                    ..Default::default()
                });
            }
        }

        verse_in_block_idx += 1;
    }

    let active_verses: Vec<(usize, Wers, StanzaConfig)> = {
        let mut res = vec![];
        let mut curr_cfg = StanzaConfig::default();
        for (line_idx, line) in lines.iter().enumerate() {
            if comment_lines.contains(&line_idx) {
                if let Some(Ok(cfg)) = parse_directive(line) {
                    curr_cfg = cfg;
                }
            }
            if let Some(ref w) = wersy[line_idx] {
                if !w.oryginalne.is_empty() {
                    res.push((line_idx, w.clone(), curr_cfg.clone()));
                }
            }
        }
        res
    };

    let mut i = 0;
    while i < active_verses.len() {
        let (_, _, ref cfg) = active_verses[i];
        if let Some(ref scheme) = cfg.rymy_schemat {
            let scheme_chars: Vec<char> = scheme
                .chars()
                .filter(|c| c.is_ascii_alphabetic())
                .map(|c| c.to_ascii_lowercase())
                .collect();
            let block_len = scheme_chars.len();
            if block_len > 0 && i + block_len <= active_verses.len() {
                let mut block_verses = vec![];
                for offset in 0..block_len {
                    let (line_idx, ref w, _) = active_verses[i + offset];
                    let letter = scheme_chars[offset];
                    block_verses.push((line_idx, w.clone(), letter));
                }

                for a_idx in 0..block_verses.len() {
                    for b_idx in (a_idx + 1)..block_verses.len() {
                        let (line_a, ref w_a, char_a) = block_verses[a_idx];
                        let (line_b, ref w_b, char_b) = block_verses[b_idx];
                        let (rymuja, _, _) = wers::czy_rymują(w_a, w_b);

                        let line = lines[line_a];
                        let chars: Vec<char> = line.chars().collect();
                        let start = chars.iter().position(|c| !c.is_whitespace()).unwrap_or(0);
                        let end = chars
                            .iter()
                            .rposition(|c| !c.is_whitespace())
                            .map(|i| i + 1)
                            .unwrap_or(chars.len());

                        if char_a == char_b {
                            if !rymuja {
                                diagnostics.push(Diagnostic {
                                    range: Range {
                                        start: Position { line: line_a as u32, character: start as u32 },
                                        end: Position { line: line_a as u32, character: end as u32 },
                                    },
                                    severity: Some(DiagnosticSeverity::WARNING),
                                    code: Some(NumberOrString::String("rymy-mismatch".to_string())),
                                    source: Some("wierszopis-lsp".to_string()),
                                    message: format!("Wers {} miał rymować się z wersem {} (grupa '{}') według schematu '{}', ale rym nie występuje.", line_a + 1, line_b + 1, char_a, scheme),
                                    ..Default::default()
                                });
                            }
                        } else {
                            if rymuja {
                                diagnostics.push(Diagnostic {
                                    range: Range {
                                        start: Position { line: line_a as u32, character: start as u32 },
                                        end: Position { line: line_a as u32, character: end as u32 },
                                    },
                                    severity: Some(DiagnosticSeverity::WARNING),
                                    code: Some(NumberOrString::String("rymy-unexpected".to_string())),
                                    source: Some("wierszopis-lsp".to_string()),
                                    message: format!("Wers {} i wers {} nie powinny się rymować (różne grupy '{}' i '{}') według schematu '{}'.", line_a + 1, line_b + 1, char_a, char_b, scheme),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }

                i += block_len;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    diagnostics
}
