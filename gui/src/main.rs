use dioxus::prelude::*;
use wierszopis::analizator;
use wierszopis::wers;

fn main() {
    dioxus::launch(App);
}

fn get_rhyme_bg(kolor: usize) -> String {
    if kolor == 0 {
        return "#f7fafc".to_string();
    }
    // Pastel equivalents matching PALETA_KOLORÓW order from CLI
    let pastel_palette = [
        "#fff5f5", // 160: Red/Crimson
        "#f0fff4", // 28: Forest Green
        "#feebc8", // 166: Orange/Brown
        "#ebf8ff", // 27: Blue
        "#f3e8ff", // 93: Purple
        "#e6fffa", // 30: Teal
        "#ffe3e3", // 124: Dark Red
        "#c6f6d5", // 34: Green
        "#fbd38d", // 208: Bright Orange
        "#bee3f8", // 21: Bright Blue
        "#e9d8fd", // 54: Deep Purple
        "#b2f5ea", // 37: Cyan
        "#fefcbf", // 220: Gold/Yellow
    ];
    let idx = (kolor - 1) % pastel_palette.len();
    pastel_palette[idx].to_string()
}

#[component]
fn App() -> Element {
    let mut tekst = use_signal(|| {
        "Litwo! Ojczyzno moja! ty jesteś jak zdrowie;\nIle cię trzeba cenić, ten tylko się dowie,\nKto cię stracił. Dziś piękność twą w całej ozdobie\nWidzę i opisuję, bo tęsknię po tobie."
            .to_string()
    });

    let analyzed_lines = use_memo(move || {
        let text_val = tekst.read();
        let wersy = text_val
            .lines()
            .map(|line| wers::nowy_wers(line))
            .collect::<Vec<_>>();

        let mut analiza = analizator::AnalizaWiersza::nowy(wersy);
        analiza.analizuj_rymy();
        analiza.optymalizuj_średniówki();
        analiza.wersy
    });

    let lines = analyzed_lines.read();

    rsx! {
        div {
            style: "max-width: 1000px; width: 95%; margin: 40px auto; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 20px; color: #333;",
            
            h1 { style: "font-size: 2.2rem; margin-bottom: 8px; color: #1a202c;", "Wierszopis - Analiza Wersu i Rymów" }
            p { style: "color: #666; margin-bottom: 24px; font-size: 1.05rem;", "Wpisz lub wklej wiersz poniżej, aby przeanalizować sylaby, akcenty i rymy." }

            div { style: "margin-bottom: 24px;",
                textarea {
                    style: "width: 100%; height: 200px; padding: 14px; font-size: 1.05rem; border: 1px solid #cbd5e0; border-radius: 8px; font-family: inherit; resize: vertical;",
                    value: "{tekst}",
                    oninput: move |evt| tekst.set(evt.value()),
                }
            }

            h2 { style: "font-size: 1.4rem; margin-bottom: 16px; border-bottom: 2px solid #e2e8f0; padding-bottom: 8px;", "Wyniki analizy:" }

            div { style: "display: flex; flex-direction: column; gap: 12px;",
                for (idx, z) in lines.iter().enumerate() {
                    if z.wers.oryginalne.is_empty() {
                        div { key: "{idx}", style: "height: 8px;" }
                    } else {
                        div {
                            key: "{idx}",
                            style: "background: {get_rhyme_bg(z.kolor)}; border: 1px solid #cbd5e0; border-radius: 8px; padding: 16px;",
                            div { style: "display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 8px;",
                                span { style: "font-weight: 600; font-size: 1.15rem;",
                                    for (seg, is_accented) in wers::wyznacz_podswietlone_czesci(&z.wers) {
                                        if is_accented {
                                            span { style: "text-decoration: underline; font-weight: bold; text-decoration-color: #2b6cb0; text-decoration-thickness: 2px;", "{seg}" }
                                        } else {
                                            span { "{seg}" }
                                        }
                                    }
                                }
                                div { style: "display: flex; gap: 12px; font-size: 0.95rem; align-items: center;",
                                    span { style: "background: rgba(255,255,255,0.7); color: #2b6cb0; padding: 3px 10px; border-radius: 6px; font-weight: 500; border: 1px solid #bee3f8;",
                                        "Sylaby: {z.wers.liczba_sylab}"
                                    }
                                    span { style: "background: rgba(255,255,255,0.7); color: #276749; padding: 3px 10px; border-radius: 6px; font-weight: 500; font-family: monospace; border: 1px solid #c6f6d5;",
                                        "Akcent: {z.wers.akcent_wersu}"
                                    }
                                }
                            }
                            if !z.wers.średniówka.is_empty() && z.wers.średniówka != "-" {
                                div { style: "font-size: 0.9rem; color: #4a5568; margin-top: 6px;",
                                    "Średniówka: {z.wers.średniówka}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
