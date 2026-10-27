/// Moduł interaktywnego rymownika TUI z asynchronicznym wyszukiwaniem (jobs) i zerowym opóźnieniem inputu.
use crate::indeks::WpisSlownika;
use crate::slowo;
use crate::slownik;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc, Arc,
};
use std::thread;
use std::time::{Duration, Instant};

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

struct FilterDef {
    nazwa: &'static str,
    match_fn: fn(&str) -> bool,
}

const FILTRY: &[FilterDef] = &[
    FilterDef { nazwa: "Wszystkie", match_fn: |_| true },
    FilterDef {
        nazwa: "Rzeczownik",
        match_fn: |pos| {
            let p = pos.to_lowercase();
            p.starts_with("subst") || p == "depr"
        },
    },
    FilterDef {
        nazwa: "Czasownik",
        match_fn: |pos| {
            let p = pos.to_lowercase();
            p == "verb" || p == "fin" || p == "bedzie" || p == "inf" || p == "praet" || p == "impt" || p == "ger" || p == "pact" || p == "ppas" || p == "winien" || p == "pred" || p == "imps" || p == "pcon" || p == "pant" || p.starts_with("verb")
        },
    },
    FilterDef {
        nazwa: "Przymiotnik",
        match_fn: |pos| pos.to_lowercase().starts_with("adj"),
    },
    FilterDef {
        nazwa: "Przysłówek",
        match_fn: |pos| pos.to_lowercase() == "adv",
    },
    FilterDef {
        nazwa: "Liczebnik",
        match_fn: |pos| {
            let p = pos.to_lowercase();
            p == "num" || p == "romandig"
        },
    },
    FilterDef {
        nazwa: "Zaimek",
        match_fn: |pos| {
            let p = pos.to_lowercase();
            p.starts_with("pron") || p == "ppron12" || p == "ppron3" || p == "siebie"
        },
    },
    FilterDef {
        nazwa: "Przyimek",
        match_fn: |pos| pos.to_lowercase() == "prep",
    },
    FilterDef {
        nazwa: "Spójnik",
        match_fn: |pos| {
            let p = pos.to_lowercase();
            p == "conj" || p == "comp"
        },
    },
    FilterDef {
        nazwa: "Wykrzyknik",
        match_fn: |pos| pos.to_lowercase() == "interj",
    },
    FilterDef {
        nazwa: "Partykuła",
        match_fn: |pos| {
            let p = pos.to_lowercase();
            p == "part" || p == "frag"
        },
    },
    FilterDef {
        nazwa: "Nierozpoznane",
        match_fn: |pos| {
            let p = pos.to_lowercase();
            p == "ign" || p.is_empty()
        },
    },
];

#[derive(Clone)]
struct WynikRymu {
    slowo: String,
    forma_podstawowa: String,
    czesc_mowy: String,
    cechy_gramatyczne: String,
    pelny_tag: String,
    koszt: i32,
}

struct DictionaryIndex {
    entries: Vec<WpisSlownika>,
    buckets: HashMap<String, Vec<usize>>, // klucz_bucketa -> indices into entries
}

impl DictionaryIndex {
    fn new(cache: crate::indeks::IndeksCache) -> Self {
        DictionaryIndex {
            entries: cache.entries,
            buckets: cache.buckets,
        }
    }
}

struct WorkerResultMsg {
    query: String,
    worker_id: usize,
    total_workers: usize,
    cost_buckets: Vec<Vec<WynikRymu>>,
    duration: Duration,
}

fn polskie_porównanie(s1: &str, s2: &str) -> std::cmp::Ordering {
    let alfabet = "aąbcćdeęfghijklłmnńoóprsśtuwyzźż";
    let get_val = |c: char| -> usize {
        if let Some(pos) = alfabet.chars().position(|x| x == c) {
            pos + 1
        } else {
            999
        }
    };

    let r1: Vec<char> = s1.to_lowercase().chars().collect();
    let r2: Vec<char> = s2.to_lowercase().chars().collect();

    let min_len = r1.len().min(r2.len());
    for i in 0..min_len {
        let v1 = get_val(r1[i]);
        let v2 = get_val(r2[i]);
        if v1 != v2 {
            return v1.cmp(&v2);
        }
    }
    r1.len().cmp(&r2.len())
}

pub fn uruchom_tui() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Dictionary loading channel
    let (dict_tx, dict_rx) = mpsc::channel();
    let load_start = Instant::now();
    thread::spawn(move || {
        let res = slownik::załaduj_słownik().map(DictionaryIndex::new);
        let dur = load_start.elapsed();
        let _ = dict_tx.send((res, dur));
    });

    // Search job channel
    let (search_tx, search_rx) = mpsc::channel();

    let mut dict_index: Option<Result<(Arc<DictionaryIndex>, Duration), String>> = None;
    let mut input = String::new();
    let mut cursor_pos = 0; // character index
    let mut wyniki: Vec<WynikRymu> = Vec::new();
    let mut worker_results: HashMap<usize, Vec<Vec<WynikRymu>>> = HashMap::new();
    let mut total_workers_expected = 0;
    let mut wybrany_indeks = 0;
    let mut okno_start = 0;
    let mut filtr_index = 0;
    let mut status_message = String::new();
    let mut search_duration = Duration::ZERO;

    let mut spinner_frame = 0;
    let mut last_spinner_tick = Instant::now();

    let mut active_cancel_token: Option<Arc<AtomicBool>> = None;
    let mut last_queued_query = String::new();

    loop {
        // Check dictionary loading
        if dict_index.is_none() {
            if let Ok((res, dur)) = dict_rx.try_recv() {
                match res {
                    Ok(idx) => {
                        let arc_idx = Arc::new(idx);
                        dict_index = Some(Ok((arc_idx.clone(), dur)));
                        // If user already typed something before dict loaded, queue search now
                        if !input.trim().is_empty() {
                            worker_results.clear();
                            queue_search(
                                input.clone(),
                                arc_idx,
                                search_tx.clone(),
                                &mut active_cancel_token,
                                &mut last_queued_query,
                                &mut total_workers_expected,
                            );
                        }
                    }
                    Err(e) => dict_index = Some(Err(e.to_string())),
                }
            }
        }

        // Check completed search results
        while let Ok(msg) = search_rx.try_recv() {
            if msg.query == input.trim().to_lowercase() {
                worker_results.insert(msg.worker_id, msg.cost_buckets);
                total_workers_expected = msg.total_workers;
                search_duration = msg.duration;

                let mut rebuilt = Vec::new();
                let max_cost = worker_results.values()
                    .flat_map(|buckets| buckets.iter().enumerate())
                    .filter(|(_, b)| !b.is_empty())
                    .map(|(c, _)| c)
                    .max()
                    .unwrap_or(0);

                for cost in 0..=max_cost {
                    for w_id in 0..total_workers_expected {
                        if let Some(buckets) = worker_results.get(&w_id) {
                            if cost < buckets.len() {
                                rebuilt.extend(buckets[cost].iter().cloned());
                            }
                        }
                    }
                }
                wyniki = rebuilt;
            }
        }

        // Throttle spinner to 100ms per frame (1 rotation per second for 10 frames)
        if last_spinner_tick.elapsed() >= Duration::from_millis(100) {
            last_spinner_tick = Instant::now();
            spinner_frame = (spinner_frame + 1) % SPINNER_FRAMES.len();
        }

        let size = terminal.size()?;
        let height = size.height;

        terminal.draw(|f| {
            let area = f.size();
            if area.width < 60 || area.height < 15 {
                let warning_paragraph = Paragraph::new(format!(
                    "⚠️ TERMINAL JEST ZA MAŁY\n\nAktualny rozmiar: {}x{}\nWymagane minimum: szerokość 60, wysokość 15.\n\nZwiększ rozmiar okna terminala.",
                    area.width, area.height
                ))
                .style(Style::default().fg(Color::Red))
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Red)));
                f.render_widget(warning_paragraph, area);
                return;
            }

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5), // Header + input + filter
                    Constraint::Min(1),    // Main content
                ])
                .split(area);

            // Header area
            let header_chunk = chunks[0];
            let header_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(3), Constraint::Length(1)])
                .split(header_chunk);

            // Line 1: Title & Status
            let title_span = Span::styled("⚡ WIERSZOPIS", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD));
            let status_span = match &dict_index {
                Some(Ok((index, dur))) => Span::styled(
                    format!(" [{0} haseł w {1:?}]", index.entries.len(), dur),
                    Style::default().fg(Color::Green),
                ),
                Some(Err(e)) => Span::styled(format!(" ✗ Błąd: {}", e), Style::default().fg(Color::Red)),
                None => {
                    let spinner = SPINNER_FRAMES[spinner_frame];
                    Span::styled(format!(" {} Ładowanie słownika...", spinner), Style::default().fg(Color::Yellow))
                }
            };
            let header_line = Line::from(vec![title_span, status_span]);
            f.render_widget(Paragraph::new(header_line), header_layout[0]);

            // Line 2: Input box with 1-space margin
            let input_widget = Paragraph::new(format!(" {}", input))
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Magenta)));
            f.render_widget(input_widget, header_layout[1]);

            // Position terminal cursor inside input box
            let cursor_x = header_layout[1].x + 2 + (unicode_width::UnicodeWidthStr::width(&input[..input.char_indices().nth(cursor_pos).map_or(input.len(), |(idx, _)| idx)]) as u16);
            let cursor_y = header_layout[1].y + 1;
            f.set_cursor(cursor_x, cursor_y);

            // Line 3: Filter bar & status message
            let active_filter = FILTRY[filtr_index].nazwa;
            let przefiltrowane: Vec<&WynikRymu> = wyniki
                .iter()
                .filter(|r| (FILTRY[filtr_index].match_fn)(&r.czesc_mowy))
                .collect();
            let filter_str = format!(
                "[{0} ({1}/{2})] ({3}ms)",
                active_filter,
                przefiltrowane.len(),
                wyniki.len(),
                search_duration.as_millis()
            );
            let mut filter_spans = vec![Span::styled(filter_str, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))];
            if !status_message.is_empty() {
                filter_spans.push(Span::styled(format!("  {}", status_message), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
            }
            f.render_widget(Paragraph::new(Line::from(filter_spans)), header_layout[2]);

            // Main content area: Left (Results table) & Right (Details preview)
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            // Left Box: Results Table
            let max_widoczne = main_chunks[1].height.saturating_sub(4) as usize;
            let mut left_lines = vec![
                Line::from(vec![Span::styled(format!(" {:3} │ {:<24}", "Koszt", "Słowo"), Style::default().fg(Color::DarkGray))]),
                Line::from(vec![Span::styled(" ───────┼──────────────────────────", Style::default().fg(Color::DarkGray))]),
            ];

            let koniec = (okno_start + max_widoczne).min(przefiltrowane.len());
            for i in okno_start..koniec {
                let r = przefiltrowane[i];
                let row_str = format!(" {:5} │ {:<24}", r.koszt, r.slowo);
                if i == wybrany_indeks {
                    left_lines.push(Line::from(Span::styled(row_str, Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD))));
                } else {
                    left_lines.push(Line::from(row_str));
                }
            }

            if przefiltrowane.is_empty() {
                let msg = if input.trim().is_empty() {
                    "  (wpisz słowo w polu powyżej, aby szukać rymów...)"
                } else if dict_index.is_none() {
                    "  (trwa ładowanie słownika...)"
                } else {
                    "  (brak wyników lub brak rymów)"
                };
                left_lines.push(Line::from(Span::styled(msg, Style::default().fg(Color::DarkGray))));
            } else {
                let page_info = format!(" [Poz. {0}-{1} z {2}]", okno_start + 1, koniec, przefiltrowane.len());
                left_lines.push(Line::from(Span::styled(page_info, Style::default().fg(Color::DarkGray))));
            }

            let left_widget = Paragraph::new(left_lines)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Magenta)).title("Rymy"));
            f.render_widget(left_widget, main_chunks[0]);

            // Right Box: Details Preview Panel (with deduplicated details display)
            let mut right_lines = Vec::new();
            if !przefiltrowane.is_empty() && wybrany_indeks < przefiltrowane.len() {
                let r = przefiltrowane[wybrany_indeks];
                right_lines.push(Line::from(vec![Span::styled("Słowo:", Style::default().fg(Color::DarkGray))]));
                right_lines.push(Line::from(vec![Span::styled(format!("  {}", r.slowo), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))]));
                right_lines.push(Line::from(""));

                if let Some(Ok((index, _))) = &dict_index {
                    let mut seen_details = HashSet::new();
                    let mut details_list = Vec::new();
                    let s_word = slowo::nowe_słowo(&r.slowo);
                    if let Some(bucket) = index.buckets.get(&s_word.klucz_bucketa()) {
                        for &idx in bucket {
                            let entry = &index.entries[idx];
                            if entry.slowo == r.slowo {
                                let key = (
                                    entry.forma_podstawowa.clone(),
                                    entry.czesc_mowy.clone(),
                                    entry.cechy_gramatyczne.clone(),
                                    entry.pelny_tag.clone(),
                                );
                                if seen_details.insert(key) {
                                    details_list.push(entry);
                                }
                            }
                        }
                    }

                    let fallback;
                    if details_list.is_empty() {
                        fallback = WpisSlownika {
                            slowo: r.slowo.clone(),
                            forma_podstawowa: r.forma_podstawowa.clone(),
                            czesc_mowy: r.czesc_mowy.clone(),
                            cechy_gramatyczne: r.cechy_gramatyczne.clone(),
                            pelny_tag: r.pelny_tag.clone(),
                            liczba_sylab: 0,
                        };
                        details_list.push(&fallback);
                    }

                    for (idx_d, detail) in details_list.iter().enumerate() {
                        if idx_d > 0 {
                            right_lines.push(Line::from(Span::styled("────────────────────────", Style::default().fg(Color::DarkGray))));
                        }
                        let przetłumaczona_czesc = slownik::przetłumacz_część_mowy(&detail.czesc_mowy);
                        right_lines.push(Line::from(vec![Span::styled("Forma podstawowa (Lemat):", Style::default().fg(Color::DarkGray))]));
                        right_lines.push(Line::from(vec![Span::styled(format!("  {}", detail.forma_podstawowa), Style::default().add_modifier(Modifier::BOLD))]));
                        right_lines.push(Line::from(""));
                        right_lines.push(Line::from(vec![Span::styled("Część mowy:", Style::default().fg(Color::DarkGray))]));
                        right_lines.push(Line::from(vec![Span::styled(format!("  {}", przetłumaczona_czesc), Style::default())]));
                        right_lines.push(Line::from(""));
                        right_lines.push(Line::from(vec![Span::styled("Cechy gramatyczne:", Style::default().fg(Color::DarkGray))]));
                        right_lines.push(Line::from(vec![Span::styled(format!("  {}", detail.cechy_gramatyczne), Style::default())]));
                        right_lines.push(Line::from(""));
                        right_lines.push(Line::from(vec![Span::styled("Pełny tag morfologiczny:", Style::default().fg(Color::DarkGray))]));
                        right_lines.push(Line::from(vec![Span::styled(format!("  {}", detail.pelny_tag), Style::default().fg(Color::DarkGray))]));
                    }
                }
            } else {
                right_lines.push(Line::from(Span::styled("(Brak wybranego wyrazu)\n\nWskazówka:\nWpisz słowo, wybierz wynik\nz listy, aby zobaczyć szczegóły.", Style::default().fg(Color::DarkGray))));
            }

            let right_widget = Paragraph::new(right_lines)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Magenta)).title("Szczegóły wyrazu"))
                .wrap(Wrap { trim: true });
            f.render_widget(right_widget, main_chunks[1]);
        })?;

        if event::poll(Duration::from_millis(15))? {
            if let Event::Key(key) = event::read()? {
                let przefiltrowane: Vec<&WynikRymu> = wyniki
                    .iter()
                    .filter(|r| (FILTRY[filtr_index].match_fn)(&r.czesc_mowy))
                    .collect();
                let max_widoczne = (height.saturating_sub(9)) as usize;

                let chars_len = input.chars().count();

                let mut trigger_search = false;

                // Keybindings & Emacs editing
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('c') => break,
                        KeyCode::Char('d') => {
                            let half = max_widoczne / 2;
                            if !przefiltrowane.is_empty() {
                                wybrany_indeks = (wybrany_indeks + half).min(przefiltrowane.len() - 1);
                                if wybrany_indeks >= okno_start + max_widoczne {
                                    okno_start = wybrany_indeks - max_widoczne + 1;
                                }
                            }
                        }
                        KeyCode::Char('u') => {
                            let half = max_widoczne / 2;
                            wybrany_indeks = wybrany_indeks.saturating_sub(half);
                            if wybrany_indeks < okno_start {
                                okno_start = wybrany_indeks;
                            }
                        }
                        KeyCode::Char('f') => {
                            if !przefiltrowane.is_empty() {
                                wybrany_indeks = (wybrany_indeks + max_widoczne).min(przefiltrowane.len() - 1);
                                if wybrany_indeks >= okno_start + max_widoczne {
                                    okno_start = wybrany_indeks - max_widoczne + 1;
                                }
                            }
                        }
                        KeyCode::Char('b') => {
                            wybrany_indeks = wybrany_indeks.saturating_sub(max_widoczne);
                            if wybrany_indeks < okno_start {
                                okno_start = wybrany_indeks;
                            }
                        }
                        KeyCode::Char('n') => {
                            if !przefiltrowane.is_empty() && wybrany_indeks < przefiltrowane.len() - 1 {
                                wybrany_indeks += 1;
                                if wybrany_indeks >= okno_start + max_widoczne {
                                    okno_start = wybrany_indeks - max_widoczne + 1;
                                }
                            }
                        }
                        KeyCode::Char('p') => {
                            if wybrany_indeks > 0 {
                                wybrany_indeks -= 1;
                                if wybrany_indeks < okno_start {
                                    okno_start = wybrany_indeks;
                                }
                            }
                        }
                        KeyCode::Char('a') => cursor_pos = 0,
                        KeyCode::Char('e') => cursor_pos = chars_len,
                        KeyCode::Char('k') => {
                            let mut chars: Vec<char> = input.chars().collect();
                            chars.truncate(cursor_pos);
                            input = chars.into_iter().collect();
                            trigger_search = true;
                        }
                        KeyCode::Char('w') => {
                            if cursor_pos > 0 {
                                let mut chars: Vec<char> = input.chars().collect();
                                let mut new_pos = cursor_pos;
                                while new_pos > 0 && chars[new_pos - 1].is_whitespace() {
                                    new_pos -= 1;
                                }
                                while new_pos > 0 && !chars[new_pos - 1].is_whitespace() {
                                    new_pos -= 1;
                                }
                                chars.drain(new_pos..cursor_pos);
                                input = chars.into_iter().collect();
                                cursor_pos = new_pos;
                                trigger_search = true;
                            }
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Esc => break,
                        KeyCode::Enter => {
                            if !przefiltrowane.is_empty() && wybrany_indeks < przefiltrowane.len() {
                                let r = przefiltrowane[wybrany_indeks];
                                status_message = format!("✓ Wybrano wyraz: {}", r.slowo);
                            }
                        }
                        KeyCode::Tab => {
                            let dir = if key.modifiers.contains(KeyModifiers::SHIFT) { -1 } else { 1 };
                            filtr_index = ((filtr_index as isize + dir + FILTRY.len() as isize) % FILTRY.len() as isize) as usize;
                            wybrany_indeks = 0;
                            okno_start = 0;
                            status_message.clear();
                        }
                        KeyCode::BackTab => {
                            filtr_index = (filtr_index + FILTRY.len() - 1) % FILTRY.len();
                            wybrany_indeks = 0;
                            okno_start = 0;
                            status_message.clear();
                        }
                        KeyCode::Left => {
                            cursor_pos = cursor_pos.saturating_sub(1);
                        }
                        KeyCode::Right => {
                            cursor_pos = (cursor_pos + 1).min(chars_len);
                        }
                        KeyCode::Home => {
                            cursor_pos = 0;
                        }
                        KeyCode::End => {
                            cursor_pos = chars_len;
                        }
                        KeyCode::Up => {
                            if wybrany_indeks > 0 {
                                wybrany_indeks -= 1;
                                if wybrany_indeks < okno_start {
                                    okno_start = wybrany_indeks;
                                }
                            }
                        }
                        KeyCode::Down => {
                            if !przefiltrowane.is_empty() && wybrany_indeks < przefiltrowane.len() - 1 {
                                wybrany_indeks += 1;
                                if wybrany_indeks >= okno_start + max_widoczne {
                                    okno_start = wybrany_indeks - max_widoczne + 1;
                                }
                            }
                        }
                        KeyCode::PageUp => {
                            wybrany_indeks = wybrany_indeks.saturating_sub(max_widoczne);
                            if wybrany_indeks < okno_start {
                                okno_start = wybrany_indeks;
                            }
                        }
                        KeyCode::PageDown => {
                            if !przefiltrowane.is_empty() {
                                wybrany_indeks = (wybrany_indeks + max_widoczne).min(przefiltrowane.len() - 1);
                                if wybrany_indeks >= okno_start + max_widoczne {
                                    okno_start = wybrany_indeks - max_widoczne + 1;
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            let mut chars: Vec<char> = input.chars().collect();
                            chars.insert(cursor_pos, c);
                            input = chars.into_iter().collect();
                            cursor_pos += 1;
                            trigger_search = true;
                        }
                        KeyCode::Backspace => {
                            if cursor_pos > 0 {
                                let mut chars: Vec<char> = input.chars().collect();
                                chars.remove(cursor_pos - 1);
                                input = chars.into_iter().collect();
                                cursor_pos -= 1;
                                trigger_search = true;
                            }
                        }
                        KeyCode::Delete => {
                            if cursor_pos < chars_len {
                                let mut chars: Vec<char> = input.chars().collect();
                                chars.remove(cursor_pos);
                                input = chars.into_iter().collect();
                                trigger_search = true;
                            }
                        }
                        _ => {}
                    }
                }

                if trigger_search {
                    // Instantly clear results and reset selection on input change
                    wyniki.clear();
                    worker_results.clear();
                    total_workers_expected = 0;
                    wybrany_indeks = 0;
                    okno_start = 0;
                    search_duration = Duration::ZERO;
                    status_message.clear();
                    if let Some(Ok((index_arc, _))) = &dict_index {
                        queue_search(
                            input.clone(),
                            index_arc.clone(),
                            search_tx.clone(),
                            &mut active_cancel_token,
                            &mut last_queued_query,
                            &mut total_workers_expected,
                        );
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn queue_search(
    query: String,
    index: Arc<DictionaryIndex>,
    search_tx: mpsc::Sender<WorkerResultMsg>,
    active_cancel_token: &mut Option<Arc<AtomicBool>>,
    last_queued_query: &mut String,
    total_workers_out: &mut usize,
) {
    let trimmed = query.trim().to_lowercase();
    *last_queued_query = trimmed.clone();

    // Cancel previous search job immediately
    if let Some(token) = active_cancel_token.take() {
        token.store(true, Ordering::Relaxed);
    }

    let cancel_token = Arc::new(AtomicBool::new(false));
    *active_cancel_token = Some(cancel_token.clone());

    if trimmed.is_empty() {
        *total_workers_out = 0;
        return;
    }

    let num_workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(8);
    *total_workers_out = 1;

    let index_arc = index.clone();
    let cancel_token_clone = cancel_token.clone();

    thread::spawn(move || {
        // Debounce sleep
        thread::sleep(Duration::from_millis(150));
        if cancel_token_clone.load(Ordering::Relaxed) {
            return;
        }

        let search_start = Instant::now();
        let target = slowo::nowe_słowo(&trimmed);
        let target_bucket = target.klucz_bucketa();

        let mut seen_idx = HashSet::new();
        let mut candidate_indices = Vec::new();

        for (k, b) in &index_arc.buckets {
            if k == &target_bucket || (k.chars().last() == target_bucket.chars().last()) {
                for &idx in b {
                    if seen_idx.insert(idx) {
                        candidate_indices.push(idx);
                    }
                }
            }
        }

        if candidate_indices.is_empty() {
            for i in 0..index_arc.entries.len() {
                candidate_indices.push(i);
            }
        }

        let candidates_arc = Arc::new(candidate_indices);
        let counter = Arc::new(AtomicUsize::new(0));
        let (results_tx, results_rx) = mpsc::channel();

        let mut handles = vec![];
        for _ in 0..num_workers {
            let candidates = candidates_arc.clone();
            let counter = counter.clone();
            let index_arc = index_arc.clone();
            let results_tx = results_tx.clone();
            let cancel_token = cancel_token_clone.clone();
            let trimmed = trimmed.clone();
            let target = target.clone();

            let handle = thread::spawn(move || {
                let mut seen_wyniki = HashSet::new();
                loop {
                    let pos = counter.fetch_add(1, Ordering::Relaxed);
                    if pos >= candidates.len() {
                        break;
                    }
                    if pos % 200 == 0 && cancel_token.load(Ordering::Relaxed) {
                        return;
                    }

                    let idx = candidates[pos];
                    let entry = &index_arc.entries[idx];
                    if entry.slowo == trimmed {
                        continue;
                    }
                    if !seen_wyniki.insert(entry.slowo.clone()) {
                        continue;
                    }

                    let candidate = slowo::nowe_słowo(&entry.slowo);
                    let (ok, koszt, _prog) = slowo::czy_rymują(&target, &candidate);
                    if ok {
                        let _ = results_tx.send(WynikRymu {
                            slowo: entry.slowo.clone(),
                            forma_podstawowa: entry.forma_podstawowa.clone(),
                            czesc_mowy: entry.czesc_mowy.clone(),
                            cechy_gramatyczne: entry.cechy_gramatyczne.clone(),
                            pelny_tag: entry.pelny_tag.clone(),
                            koszt,
                        });
                    }
                }
            });
            handles.push(handle);
        }

        drop(results_tx);

        let search_tx_stream = search_tx;
        let trimmed_clone = trimmed.clone();
        let cancel_token_stream = cancel_token_clone.clone();

        let aggregator_handle = thread::spawn(move || {
            let mut accumulated_buckets: Vec<Vec<WynikRymu>> = Vec::new();
            let mut batch = Vec::new();
            let mut seen_global = HashSet::new();
            let mut last_flush = Instant::now();
            let flush_interval = Duration::from_millis(50);

            let flush = |batch: &mut Vec<WynikRymu>, buckets: &mut Vec<Vec<WynikRymu>>, seen: &mut HashSet<String>| {
                if batch.is_empty() {
                    return;
                }
                for r in batch.drain(..) {
                    if !seen.insert(r.slowo.clone()) {
                        continue;
                    }
                    let k_idx = r.koszt as usize;
                    if buckets.len() <= k_idx {
                        buckets.resize_with(k_idx + 1, Vec::new);
                    }
                    buckets[k_idx].push(r);
                }
                for bucket in buckets.iter_mut() {
                    bucket.sort_by(|a, b| {
                        let ord = polskie_porównanie(&a.slowo, &b.slowo);
                        if ord != std::cmp::Ordering::Equal {
                            ord
                        } else {
                            polskie_porównanie(&a.forma_podstawowa, &b.forma_podstawowa)
                        }
                    });
                }
            };

            for res in results_rx {
                if cancel_token_stream.load(Ordering::Relaxed) {
                    return;
                }
                batch.push(res);
                if last_flush.elapsed() >= flush_interval {
                    flush(&mut batch, &mut accumulated_buckets, &mut seen_global);
                    let _ = search_tx_stream.send(WorkerResultMsg {
                        query: trimmed_clone.clone(),
                        worker_id: 0,
                        total_workers: 1,
                        cost_buckets: accumulated_buckets.clone(),
                        duration: search_start.elapsed(),
                    });
                    last_flush = Instant::now();
                }
            }

            flush(&mut batch, &mut accumulated_buckets, &mut seen_global);
            let _ = search_tx_stream.send(WorkerResultMsg {
                query: trimmed_clone.clone(),
                worker_id: 0,
                total_workers: 1,
                cost_buckets: accumulated_buckets,
                duration: search_start.elapsed(),
            });
        });

        for h in handles {
            let _ = h.join();
        }
        let _ = aggregator_handle.join();
    });
}
