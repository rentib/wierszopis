use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "wierszopis",
    author,
    version,
    about = "Pomocnik do pisania i analizowania wierszy",
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub komenda: Komenda,

    #[arg(short = 'h', long = "pomoc", action = clap::ArgAction::Help, help = "Wyświetla pomoc")]
    pub pomoc: Option<bool>,

    #[arg(short = 'v', long = "wersja", action = clap::ArgAction::Version, help = "Wyświetla wersję programu")]
    pub wersja: Option<bool>,
}

#[derive(Subcommand)]
pub enum Komenda {
    /// Pokazuje szczegółową analizę głosek i sylab pojedynczego słowa.
    Wyraz {
        /// Słowo do analizy
        slowo: String,
        #[arg(short = 'h', long = "pomoc", action = clap::ArgAction::Help, help = "Wyświetla pomoc")]
        pomoc: Option<bool>,
    },
    /// Analizuje sylaby, rymy i koloruje wersy ze standardowego wejścia lub pliku.
    Analizuj {
        /// Pokaż szczegóły (koszty rymów i numery kolorów)
        #[arg(long)]
        szczegoly: bool,
        /// Ścieżka do pliku z wierszem
        #[arg(value_name = "PLIK")]
        sciezka: Option<PathBuf>,
        #[arg(short = 'h', long = "pomoc", action = clap::ArgAction::Help, help = "Wyświetla pomoc")]
        pomoc: Option<bool>,
    },
    /// Uruchamia interaktywny rymownik TUI
    Rymuj {
        /// Liczba workerów równoległych
        #[arg(long, default_value_t = 4)]
        workers: usize,
        #[arg(short = 'h', long = "pomoc", action = clap::ArgAction::Help, help = "Wyświetla pomoc")]
        pomoc: Option<bool>,
    },
    /// Buduje i zapisuje szybki indeks binarny słownika.
    BudujIndeks {
        /// Liczba workerów równoległych
        #[arg(long, default_value_t = 4)]
        workers: usize,
        /// Ścieżka do pliku słownika CSV
        #[arg(long, value_name = "PLIK")]
        plik: Option<PathBuf>,
        #[arg(short = 'h', long = "pomoc", action = clap::ArgAction::Help, help = "Wyświetla pomoc")]
        pomoc: Option<bool>,
    },
}
