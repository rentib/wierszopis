use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use wierszopis::{
    analizator::{self, StanzaConfig},
    fonetyka,
    indeks::IndeksCache,
    slownik,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintTrigger {
    OnEdit,
    OnSave,
    OnOpen,
}

#[derive(Debug, Clone)]
struct ServerConfig {
    trigger: LintTrigger,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            trigger: LintTrigger::OnSave,
        }
    }
}

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, String>>>,
    dictionary: Option<IndeksCache>,
    config: Arc<Mutex<ServerConfig>>,
}

impl Backend {
    async fn lint_and_publish_diagnostics(&self, uri: &Url, text: &str) {
        #[cfg(feature = "lsp")]
        let diagnostics = analizator::compute_diagnostics(text);
        #[cfg(not(feature = "lsp"))]
        let diagnostics = vec![];

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(
        &self,
        params: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        if let Some(opts) = params.initialization_options {
            if let Some(val) = opts
                .get("lintTrigger")
                .or_else(|| opts.get("trigger"))
                .or_else(|| opts.get("lint_trigger"))
                .and_then(|v| v.as_str())
            {
                let trigger = match val.to_lowercase().as_str() {
                    "onedit" | "edit" | "change" => LintTrigger::OnEdit,
                    "onopen" | "open" => LintTrigger::OnOpen,
                    _ => LintTrigger::OnSave,
                };
                let mut cfg = self.config.lock().await;
                cfg.trigger = trigger;
            }
        }
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                color_provider: Some(ColorProviderCapability::Simple(true)),
                inlay_hint_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        ":".to_string(),
                        " ".to_string(),
                        "=".to_string(),
                        ";".to_string(),
                    ]),
                    ..Default::default()
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::COMMENT,  // 0: Comments & Signatures
                                    SemanticTokenType::REGEXP,   // 1: Rhyme Group 1
                                    SemanticTokenType::STRING,   // 2: Rhyme Group 2
                                    SemanticTokenType::KEYWORD,  // 3: Rhyme Group 3
                                    SemanticTokenType::NUMBER,   // 4: Rhyme Group 4
                                    SemanticTokenType::OPERATOR, // 5: Rhyme Group 5
                                    SemanticTokenType::MACRO,    // 6: Accented syllables
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
                "Wierszopis LSP Server initialized with configurable trigger modes (onOpen, onSave, onEdit)!",
            )
            .await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        if let Some(val) = params
            .settings
            .get("wierszopis")
            .or_else(|| params.settings.get("wierszopisLsp"))
            .and_then(|s| {
                s.get("lintTrigger")
                    .or_else(|| s.get("trigger"))
                    .or_else(|| s.get("lint_trigger"))
            })
            .and_then(|v| v.as_str())
        {
            let trigger = match val.to_lowercase().as_str() {
                "onedit" | "edit" | "change" => LintTrigger::OnEdit,
                "onopen" | "open" => LintTrigger::OnOpen,
                _ => LintTrigger::OnSave,
            };
            let mut cfg = self.config.lock().await;
            cfg.trigger = trigger;
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        {
            let mut docs = self.documents.lock().await;
            docs.insert(uri.clone(), text.clone());
        }
        let trigger = {
            let cfg = self.config.lock().await;
            cfg.trigger
        };
        if trigger != LintTrigger::OnOpen || true {
            // always lint on open if requested or default
            self.lint_and_publish_diagnostics(&uri, &text).await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text;
            {
                let mut docs = self.documents.lock().await;
                docs.insert(uri.clone(), text.clone());
            }
            let trigger = {
                let cfg = self.config.lock().await;
                cfg.trigger
            };
            if trigger == LintTrigger::OnEdit {
                self.lint_and_publish_diagnostics(&uri, &text).await;
            }
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

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let docs = self.documents.lock().await;
        let Some(text) = docs.get(&uri) else {
            return Ok(None);
        };

        let lines: Vec<&str> = text.lines().collect();
        if (position.line as usize) >= lines.len() {
            return Ok(None);
        }

        let line = lines[position.line as usize];
        let trimmed = line.trim_start();

        if !(trimmed.starts_with("//") || trimmed.starts_with('#')) {
            return Ok(None);
        }

        let content_start_idx = if let Some(idx) = line.find("//") {
            idx + 2
        } else if let Some(idx) = line.find('#') {
            idx + 1
        } else {
            return Ok(None);
        };

        let cursor_char = position.character as usize;
        if cursor_char < content_start_idx {
            return Ok(None);
        }

        let content_until_cursor = &line[content_start_idx..cursor_char];
        let content_lower = content_until_cursor.trim_start().to_lowercase();

        let mut items = vec![];

        if content_lower.is_empty()
            || "wierszopis:".starts_with(&content_lower)
            || "@wierszopis:".starts_with(&content_lower)
        {
            items.push(CompletionItem {
                label: "wierszopis: ".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Dyrektywa Wierszopisa".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: "@wierszopis: ".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Dyrektywa Wierszopisa (alternatywna)".to_string()),
                ..Default::default()
            });
        }

        if content_until_cursor.to_lowercase().contains("wierszopis:") {
            let colon_idx = content_until_cursor
                .to_lowercase()
                .find("wierszopis:")
                .unwrap()
                + "wierszopis:".len();
            let after_colon = &content_until_cursor[colon_idx..];

            let parts: Vec<&str> = after_colon.split(';').collect();
            let last_part = parts.last().unwrap_or(&"").trim_start();

            if !last_part.contains('=') {
                let keys = ["sylaby=", "rymy=", "akcenty="];
                for key in keys {
                    if key.starts_with(&last_part.to_lowercase()) {
                        items.push(CompletionItem {
                            label: key.to_string(),
                            kind: Some(CompletionItemKind::FIELD),
                            ..Default::default()
                        });
                    }
                }
            } else {
                let kv: Vec<&str> = last_part.splitn(2, '=').collect();
                let key = kv[0].trim().to_lowercase();
                let val = if kv.len() > 1 { kv[1].trim_start() } else { "" };

                if key == "akcenty" {
                    let vals = ["tak", "nie"];
                    for v in vals {
                        if v.starts_with(&val.to_lowercase()) {
                            items.push(CompletionItem {
                                label: v.to_string(),
                                kind: Some(CompletionItemKind::VALUE),
                                ..Default::default()
                            });
                        }
                    }
                } else if key == "rymy" {
                    let common = ["aabb", "abab", "abba"];
                    for c in common {
                        if c.starts_with(&val.to_lowercase()) {
                            items.push(CompletionItem {
                                label: c.to_string(),
                                kind: Some(CompletionItemKind::VALUE),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        }

        Ok(Some(CompletionResponse::Array(items)))
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
                    let przetlumaczona_czesc =
                        wierszopis::slownik::przetłumacz_część_mowy(&entry.czesc_mowy);
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

    async fn document_color(
        &self,
        params: DocumentColorParams,
    ) -> tower_lsp::jsonrpc::Result<Vec<ColorInformation>> {
        let uri = params.text_document.uri;
        let docs = self.documents.lock().await;
        let Some(text) = docs.get(&uri) else {
            return Ok(vec![]);
        };

        let (group_map, _group_sizes, _wersy, _comments, _signatures) =
            analizator::analyze_document(text);
        let lines: Vec<&str> = text.lines().collect();

        let palette = [
            Color {
                red: 1.0,
                green: 0.3,
                blue: 0.3,
                alpha: 1.0,
            },
            Color {
                red: 0.3,
                green: 0.5,
                blue: 1.0,
                alpha: 1.0,
            },
            Color {
                red: 0.2,
                green: 0.8,
                blue: 0.4,
                alpha: 1.0,
            },
            Color {
                red: 1.0,
                green: 0.6,
                blue: 0.2,
                alpha: 1.0,
            },
            Color {
                red: 0.7,
                green: 0.3,
                blue: 0.9,
                alpha: 1.0,
            },
            Color {
                red: 0.2,
                green: 0.8,
                blue: 0.8,
                alpha: 1.0,
            },
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

    async fn inlay_hint(
        &self,
        params: InlayHintParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<InlayHint>>> {
        let uri = params.text_document.uri;
        let docs = self.documents.lock().await;
        let Some(text) = docs.get(&uri) else {
            return Ok(None);
        };

        let (_group_map, _group_sizes, wersy, _comments, _signatures) =
            analizator::analyze_document(text);
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

        let (_group_map, _group_sizes, wersy, comment_lines, signature_lines) =
            analizator::analyze_document(text);
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
                if let Some(Ok(cfg)) = analizator::parse_directive(line) {
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
                    let char_spans = analizator::get_accented_spans_for_line(line, wers_obj);
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
        config: Arc::new(Mutex::new(ServerConfig::default())),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
