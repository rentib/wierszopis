use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use wierszopis::{analizator, fonetyka, indeks::IndeksCache, slownik, wers};

#[derive(Debug, Clone, Default)]
struct StanzaConfig {
    sylaby_pattern: Option<Vec<usize>>,
    expected_sredniowka: Option<String>,
    rymy_schemat: Option<String>,
    akcenty_enabled: bool,
}

fn parse_directive(line: &str) -> Option<Result<StanzaConfig, String>> {
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
        let kv: Vec<&str> = part.split('=').map(|s| s.trim()).collect();
        if kv.len() != 2 {
            return Some(Err(format!("Niepoprawna para klucz=wartość: '{}'", part)));
        }
        let key = kv[0].to_lowercase();
        let val = kv[1].trim();

        if key == "sylaby" {
            let val_clean = if val.starts_with('[') && val.ends_with(']') {
                val[1..val.len()-1].trim()
            } else {
                val
            };

            if val_clean.contains('+') {
                let parts: Vec<&str> = val_clean.split('+').collect();
                if parts.len() == 2 {
                    match (parts[0].trim().parse::<usize>(), parts[1].trim().parse::<usize>()) {
                        (Ok(p1), Ok(p2)) => {
                            config.sylaby_pattern = Some(vec![p1 + p2]);
                            config.expected_sredniowka = Some(val_clean.to_string());
                        }
                        _ => return Some(Err(format!("Niepoprawne wartości średniówki: '{}'", val_clean))),
                    }
                } else {
                    return Some(Err(format!("Niepoprawny format średniówki: '{}'", val_clean)));
                }
            } else if val_clean.contains(',') {
                let mut numbers = vec![];
                for s in val_clean.split(',') {
                    match s.trim().parse::<usize>() {
                        Ok(n) => numbers.push(n),
                        Err(_) => return Some(Err(format!("Niepoprawna liczba w liście sylab: '{}'", s.trim()))),
                    }
                }
                if !numbers.is_empty() {
                    config.sylaby_pattern = Some(numbers);
                } else {
                    return Some(Err("Pusta lista sylab".to_string()));
                }
            } else {
                match val_clean.parse::<usize>() {
                    Ok(n) => config.sylaby_pattern = Some(vec![n]),
                    Err(_) => return Some(Err(format!("Niepoprawna liczba sylab: '{}'", val_clean))),
                }
            }
        } else if key == "rymy" {
            let val_clean = if val.starts_with('[') && val.ends_with(']') {
                val[1..val.len()-1].trim()
            } else {
                val
            };
            let val_lower = val_clean.to_string().to_lowercase();
            if !val_lower.is_empty() {
                config.rymy_schemat = Some(val_lower);
            } else {
                return Some(Err("Pusty schemat rymów".to_string()));
            }
        } else if key == "akcenty" {
            let val_lower = val.to_lowercase();
            match val_lower.as_str() {
                "true" | "tak" | "1" => config.akcenty_enabled = true,
                "false" | "nie" | "0" => config.akcenty_enabled = false,
                _ => return Some(Err(format!("Niepoprawna wartość dla opcji akcenty (oczekiwano true/false): '{}'", val))),
            }
        } else {
            return Some(Err(format!("Nieznany parametr dyrektywy: '{}'", key)));
        }
    }

    Some(Ok(config))
}

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, String>>>,
    dictionary: Option<IndeksCache>,
}

impl Backend {
    async fn analyze_document(
        &self,
        text: &str,
    ) -> (
        HashMap<usize, usize>,
        HashMap<usize, usize>,
        Vec<Option<wers::Wers>>,
        Vec<usize>, // comment lines
        Vec<usize>, // signature lines
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

        // Apply usun_podpis from analizator to remove signatures/dedications
        let mut just_wersy: Vec<wers::Wers> = raw_wersy_with_line.iter().map(|(_, w)| w.clone()).collect();
        analizator::usun_podpis(&mut just_wersy);

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

        // Any trailing lines in raw_wersy_with_line beyond valid_count are signatures
        for i in valid_count..raw_wersy_with_line.len() {
            let (line_idx, w) = &raw_wersy_with_line[i];
            if !w.oryginalne.is_empty() {
                signature_lines.push(*line_idx);
            }
        }

        let mut group_map: HashMap<usize, usize> = HashMap::new();
        let mut group_sizes: HashMap<usize, usize> = HashMap::new();
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

            // Only assign group if at least 2 verses rhyme (not a singleton)
            if members.len() >= 2 {
                for m in &members {
                    group_map.insert(*m, next_group);
                }
                group_sizes.insert(next_group, members.len());
                next_group += 1;
            }
        }

        (group_map, group_sizes, wersy, comment_lines, signature_lines)
    }

    async fn compute_diagnostics(&self, text: &str) -> Vec<Diagnostic> {
        let lines: Vec<&str> = text.lines().collect();
        let mut diagnostics = vec![];
        let (_group_map, _group_sizes, wersy, comment_lines, _signature_lines) = self.analyze_document(text).await;

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
                        let end = chars.iter().rposition(|c| !c.is_whitespace()).map(|i| i + 1).unwrap_or(chars.len());
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: line_idx as u32, character: start as u32 },
                                end: Position { line: line_idx as u32, character: end as u32 },
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
            let end = chars.iter().rposition(|c| !c.is_whitespace()).map(|i| i + 1).unwrap_or(chars.len());

            // Syllable check
            if let Some(ref pattern) = current_config.sylaby_pattern {
                let expected = pattern[verse_in_block_idx % pattern.len()];
                let actual = w.liczba_sylab;
                if actual != expected {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: line_idx as u32, character: start as u32 },
                            end: Position { line: line_idx as u32, character: end as u32 },
                        },
                        severity: Some(DiagnosticSeverity::WARNING),
                        code: Some(NumberOrString::String("sylaby-mismatch".to_string())),
                        source: Some("wierszopis-lsp".to_string()),
                        message: format!("Oczekiwano {} sylab, wykryto {} sylab.", expected, actual),
                        ..Default::default()
                    });
                }
            }

            // Caesura (średniówka) check
            if let Some(ref expected_sred) = current_config.expected_sredniowka {
                if w.średniówka != "-" && w.średniówka != *expected_sred {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: line_idx as u32, character: start as u32 },
                            end: Position { line: line_idx as u32, character: end as u32 },
                        },
                        severity: Some(DiagnosticSeverity::WARNING),
                        code: Some(NumberOrString::String("średniówka-mismatch".to_string())),
                        source: Some("wierszopis-lsp".to_string()),
                        message: format!("Oczekiwano średniówki '{}', wykryto '{}'.", expected_sred, w.średniówka),
                        ..Default::default()
                    });
                }
            }

            verse_in_block_idx += 1;
        }

        // Rhyme scheme checks
        let active_verses: Vec<(usize, wers::Wers, StanzaConfig)> = {
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
                            let end = chars.iter().rposition(|c| !c.is_whitespace()).map(|i| i + 1).unwrap_or(chars.len());

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

    async fn lint_and_publish_diagnostics(&self, uri: &Url, text: &str) {
        let diagnostics = self.compute_diagnostics(text).await;
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }

    fn get_accented_spans_for_line(line: &str, wers_obj: &wers::Wers) -> Vec<(u32, u32)> {
        let text = &wers_obj.surowy_tekst;
        let mut accented_chars = Vec::new();
        if text.is_empty() {
            return accented_chars;
        }

        let line_offset = if let Some(pos) = line.find(text) {
            pos
        } else {
            0
        };

        let mut search_start = 0;
        let mut global_syl_idx = 0;
        for sw in &wers_obj.słowa {
            if let Some(word_rel_pos) = text[search_start..].find(&sw.oryginalne) {
                let word_start = search_start + word_rel_pos;
                let spans = sw.sylaba_spans();
                for (_i, &(s_start, s_end)) in spans.iter().enumerate() {
                    let akcent_char = wers_obj.akcent_wersu.chars().nth(global_syl_idx).unwrap_or('s');
                    global_syl_idx += 1;

                    let is_accented = akcent_char == 'S';
                    if is_accented {
                        let abs_start = word_start + s_start.min(sw.oryginalne.len());
                        let abs_end = word_start + s_end.min(sw.oryginalne.len());
                        if abs_start < abs_end && abs_end <= text.len() {
                            let line_byte_start = line_offset + abs_start;
                            let line_byte_end = line_offset + abs_end;

                            let mut b_start = line_byte_start.min(line.len());
                            while b_start > 0 && !line.is_char_boundary(b_start) {
                                b_start -= 1;
                            }
                            let mut b_end = line_byte_end.min(line.len());
                            while b_end < line.len() && !line.is_char_boundary(b_end) {
                                b_end += 1;
                            }

                            if b_start < b_end {
                                let char_start = line[..b_start].chars().count() as u32;
                                let char_end = line[..b_end].chars().count() as u32;
                                if char_start < char_end {
                                    accented_chars.push((char_start, char_end));
                                }
                            }
                        }
                    }
                }
                search_start = word_start + sw.oryginalne.len();
            }
        }
        accented_chars
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                color_provider: Some(ColorProviderCapability::Simple(true)),
                inlay_hint_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::COMMENT, // 0: Comments & Signatures
                                    SemanticTokenType::REGEXP,  // 1: Rhyme Group 1
                                    SemanticTokenType::STRING,  // 2: Rhyme Group 2
                                    SemanticTokenType::KEYWORD, // 3: Rhyme Group 3
                                    SemanticTokenType::NUMBER,  // 4: Rhyme Group 4
                                    SemanticTokenType::OPERATOR,// 5: Rhyme Group 5
                                    SemanticTokenType::MACRO,   // 6: Accented syllables
                                ],
                                token_modifiers: vec![],
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: None,
                            ..Default::default()
                        },
                    ),
                ),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "wierszopis-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(
                MessageType::INFO,
                "Wierszopis LSP Server initialized with Polish directives, linting & inlay hints!",
            )
            .await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        {
            let mut docs = self.documents.lock().await;
            docs.insert(uri.clone(), text.clone());
        }
        self.lint_and_publish_diagnostics(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text;
            {
                let mut docs = self.documents.lock().await;
                docs.insert(uri.clone(), text.clone());
            }
            self.lint_and_publish_diagnostics(&uri, &text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        let text_opt = {
            let docs = self.documents.lock().await;
            docs.get(&uri).cloned()
        };
        if let Some(text) = text_opt {
            self.lint_and_publish_diagnostics(&uri, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        let mut docs = self.documents.lock().await;
        docs.remove(&uri);
    }

    async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.lock().await;
        let Some(text) = docs.get(&uri) else {
            return Ok(None);
        };

        let lines: Vec<&str> = text.lines().collect();
        if (position.line as usize) >= lines.len() {
            return Ok(None);
        }

        let line = lines[position.line as usize];
        let chars: Vec<char> = line.chars().collect();
        let char_idx = position.character as usize;

        let start = chars[..char_idx.min(chars.len())]
            .iter()
            .rposition(|c| !c.is_alphabetic())
            .map(|i| i + 1)
            .unwrap_or(0);
        let end = chars[char_idx.min(chars.len())..]
            .iter()
            .position(|c| !c.is_alphabetic())
            .map(|i| char_idx.min(chars.len()) + i)
            .unwrap_or(chars.len());

        if start >= end {
            return Ok(None);
        }

        let word: String = chars[start..end].iter().collect();
        if word.is_empty() {
            return Ok(None);
        }

        let analiza = fonetyka::analizuj_slowo(&word);
        let sylaby_str: Vec<String> = analiza.sylaby.iter().map(|s| s.concat()).collect();

        let mut hover_content = format!(
            "### Analiza słowa: `{}`\n- **Liczba sylab:** {}\n- **Sylaby:** `{:?}`\n- **Akcent:** {}\n- **Rym męski:** `{}`\n- **Rym żeński:** `{}`",
            analiza.slowo,
            analiza.liczba_sylab,
            sylaby_str,
            analiza.akcent,
            analiza.rym_meski.concat(),
            analiza.rym_zenski.concat()
        );

        if let Some(dict) = &self.dictionary {
            let matching_entries: Vec<&wierszopis::indeks::WpisSlownika> = dict
                .entries
                .iter()
                .filter(|e| e.slowo.eq_ignore_ascii_case(&word))
                .collect();

            if !matching_entries.is_empty() {
                hover_content.push_str("\n\n---\n### Informacje ze słownika:");
                for entry in matching_entries.iter().take(5) {
                    let przetlumaczona_czesc = wierszopis::slownik::przetłumacz_część_mowy(&entry.czesc_mowy);
                    hover_content.push_str(&format!(
                        "\n- **Część mowy:** `{}`\n- **Forma podstawowa:** `{}`\n- **Cechy gramatyczne:** `{}`",
                        przetlumaczona_czesc,
                        entry.forma_podstawowa,
                        entry.cechy_gramatyczne
                    ));
                }
            }
        }

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: hover_content,
            }),
            range: Some(Range {
                start: Position {
                    line: position.line,
                    character: start as u32,
                },
                end: Position {
                    line: position.line,
                    character: end as u32,
                },
            }),
        }))
    }

    async fn document_color(&self, params: DocumentColorParams) -> tower_lsp::jsonrpc::Result<Vec<ColorInformation>> {
        let uri = params.text_document.uri;
        let docs = self.documents.lock().await;
        let Some(text) = docs.get(&uri) else {
            return Ok(vec![]);
        };

        let (group_map, _group_sizes, _wersy, _comments, _signatures) = self.analyze_document(text).await;
        let lines: Vec<&str> = text.lines().collect();

        let palette = [
            Color { red: 1.0, green: 0.3, blue: 0.3, alpha: 1.0 },
            Color { red: 0.3, green: 0.5, blue: 1.0, alpha: 1.0 },
            Color { red: 0.2, green: 0.8, blue: 0.4, alpha: 1.0 },
            Color { red: 1.0, green: 0.6, blue: 0.2, alpha: 1.0 },
            Color { red: 0.7, green: 0.3, blue: 0.9, alpha: 1.0 },
            Color { red: 0.2, green: 0.8, blue: 0.8, alpha: 1.0 },
        ];

        let mut color_infos = vec![];
        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(&group_id) = group_map.get(&line_idx) {
                let chars: Vec<char> = line.chars().collect();
                let start = match chars.iter().position(|c| !c.is_whitespace()) {
                    Some(s) => s as u32,
                    None => continue,
                };
                let end = match chars.iter().rposition(|c| !c.is_whitespace()) {
                    Some(e) => (e + 1) as u32,
                    None => continue,
                };
                if start < end {
                    let color = palette[group_id % palette.len()];
                    let range = Range {
                        start: Position {
                            line: line_idx as u32,
                            character: start,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: end,
                        },
                    };
                    color_infos.push(ColorInformation { range, color });
                }
            }
        }

        Ok(color_infos)
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> tower_lsp::jsonrpc::Result<Option<Vec<InlayHint>>> {
        let uri = params.text_document.uri;
        let docs = self.documents.lock().await;
        let Some(text) = docs.get(&uri) else {
            return Ok(None);
        };

        let (_group_map, _group_sizes, wersy, _comments, _signatures) = self.analyze_document(text).await;
        let lines: Vec<&str> = text.lines().collect();

        let mut hints = vec![];
        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(wers_obj) = &wersy[line_idx] {
                if !wers_obj.akcent_wersu.is_empty() {
                    let chars: Vec<char> = line.chars().collect();
                    let end = match chars.iter().rposition(|c| !c.is_whitespace()) {
                        Some(e) => (e + 1) as u32,
                        None => line.chars().count() as u32,
                    };
                    let label = format!(" [{}]", wers_obj.akcent_wersu);
                    hints.push(InlayHint {
                        position: Position {
                            line: line_idx as u32,
                            character: end,
                        },
                        label: InlayHintLabel::String(label),
                        kind: Some(InlayHintKind::TYPE),
                        text_edits: None,
                        tooltip: Some(InlayHintTooltip::String(format!(
                            "Liczba sylab: {}, Schemat akcentów: {}",
                            wers_obj.liczba_sylab, wers_obj.akcent_wersu
                        ))),
                        padding_left: Some(true),
                        padding_right: Some(false),
                        data: None,
                    });
                }
            }
        }

        Ok(Some(hints))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> tower_lsp::jsonrpc::Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        let docs = self.documents.lock().await;
        let Some(text) = docs.get(&uri) else {
            return Ok(None);
        };

        let (_group_map, _group_sizes, wersy, comment_lines, signature_lines) = self.analyze_document(text).await;
        let lines: Vec<&str> = text.lines().collect();

        let mut raw_tokens = vec![];

        for &line_idx in &comment_lines {
            if let Some(line) = lines.get(line_idx) {
                let chars: Vec<char> = line.chars().collect();
                if let Some(start) = chars.iter().position(|c| !c.is_whitespace()) {
                    if let Some(end) = chars.iter().rposition(|c| !c.is_whitespace()) {
                        let len = (end + 1 - start) as u32;
                        raw_tokens.push((line_idx, start as u32, len, 0));
                    }
                }
            }
        }

        for &line_idx in &signature_lines {
            if let Some(line) = lines.get(line_idx) {
                let chars: Vec<char> = line.chars().collect();
                if let Some(start) = chars.iter().position(|c| !c.is_whitespace()) {
                    if let Some(end) = chars.iter().rposition(|c| !c.is_whitespace()) {
                        let len = (end + 1 - start) as u32;
                        raw_tokens.push((line_idx, start as u32, len, 0));
                    }
                }
            }
        }

        let mut curr_semantic_cfg = StanzaConfig::default();
        for (line_idx, line) in lines.iter().enumerate() {
            if comment_lines.contains(&line_idx) {
                if let Some(Ok(cfg)) = parse_directive(line) {
                    curr_semantic_cfg = cfg;
                }
                continue;
            }
            if signature_lines.contains(&line_idx) {
                continue;
            }
            let mut comment_start = None;
            if let Some(idx) = line.find("//") {
                comment_start = Some(idx);
            }
            if let Some(idx) = line.find('#') {
                comment_start = match comment_start {
                    Some(existing) => Some(std::cmp::min(existing, idx)),
                    None => Some(idx),
                };
            }
            if let Some(byte_idx) = comment_start {
                let char_idx = line[..byte_idx].chars().count() as u32;
                let comment_len = line[byte_idx..].chars().count() as u32;
                if comment_len > 0 {
                    raw_tokens.push((line_idx, char_idx, comment_len, 0));
                }
            }

            if let Some(wers_obj) = &wersy[line_idx] {
                if curr_semantic_cfg.akcenty_enabled {
                    let char_spans = Self::get_accented_spans_for_line(line, wers_obj);
                    for (char_start, char_end) in char_spans {
                        let len = char_end - char_start;
                        if len > 0 {
                            raw_tokens.push((line_idx, char_start, len, 6)); // token type 6 = MACRO (accent)
                        }
                    }
                }
            }
        }

        raw_tokens.sort_by(|a, b| {
            if a.0 != b.0 {
                a.0.cmp(&b.0)
            } else {
                a.1.cmp(&b.1)
            }
        });

        let mut tokens = vec![];
        let mut prev_line = 0;
        let mut prev_char = 0;

        for (line_idx, start_char, length, token_type) in raw_tokens {
            let delta_line = (line_idx as i32 - prev_line as i32) as u32;
            let delta_start = if delta_line == 0 {
                (start_char as i32 - prev_char as i32) as u32
            } else {
                start_char
            };

            tokens.push(SemanticToken {
                delta_line,
                delta_start,
                length,
                token_type,
                token_modifiers_bitset: 0,
            });

            prev_line = line_idx;
            prev_char = start_char;
        }

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens,
        })))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let dictionary = slownik::załaduj_słownik().ok();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: Arc::new(Mutex::new(HashMap::new())),
        dictionary,
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_directive_sylaby_comma_and_brackets() {
        let res1 = parse_directive("# wierszopis: sylaby=12,12,7,7,8");
        assert!(res1.is_some());
        let cfg1 = res1.unwrap().unwrap();
        assert_eq!(cfg1.sylaby_pattern, Some(vec![12, 12, 7, 7, 8]));

        let res2 = parse_directive("# wierszopis: sylaby=[12, 12, 7, 7, 8]; akcenty = true");
        assert!(res2.is_some());
        let cfg2 = res2.unwrap().unwrap();
        assert_eq!(cfg2.sylaby_pattern, Some(vec![12, 12, 7, 7, 8]));
        assert!(cfg2.akcenty_enabled);

        let res3 = parse_directive("# wierszopis: akcenty=false");
        assert!(res3.is_some());
        let cfg3 = res3.unwrap().unwrap();
        assert!(!cfg3.akcenty_enabled);
    }

    #[test]
    fn test_parse_directive_invalid() {
        let res = parse_directive("# wierszopis: sylaby=abc");
        assert!(res.is_some());
        assert!(res.unwrap().is_err());
    }
}
