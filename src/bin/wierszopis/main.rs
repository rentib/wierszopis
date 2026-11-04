use clap::Parser;
use std::fs::File;
use std::io;
use wierszopis::{analizator, fonetyka, indeks, slownik};

mod args;
mod tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = args::Cli::parse();

    match cli.komenda {
        args::Komenda::Wyraz { slowo, .. } => {
            let analiza = fonetyka::analizuj_slowo(&slowo);
            let sylaby_str: Vec<String> = analiza.sylaby.iter().map(|s| s.concat()).collect();
            println!("Słowo: {}", analiza.slowo);
            println!("Liczba sylab: {}", analiza.liczba_sylab);
            println!("Samogłoski: {:?}", analiza.samogloski);
            println!("Głoski: {:?}", analiza.gloski);
            println!("Sylaby: {:?}", sylaby_str);
            println!("Akcent: {}", analiza.akcent);
            println!("Rym męski: {}", analiza.rym_meski.concat());
            println!("Rym żeński: {}", analiza.rym_zenski.concat());
        }
        args::Komenda::Analizuj {
            sciezka, szczegoly, ..
        } => {
            if let Some(p) = sciezka {
                let f = File::open(p)?;
                analizator::analizuj_strumien(f, szczegoly)?;
            } else {
                analizator::analizuj_strumien(io::stdin(), szczegoly)?;
            }
        }
        args::Komenda::Rymuj { .. } => {
            tui::uruchom_tui()?;
        }
        args::Komenda::BudujIndeks { plik, .. } => {
            let csv_path = plik
                .or_else(slownik::pobierz_sciezke_slownika)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "Nie podano pliku słownika CSV i nie znaleziono domyślnego w XDG.",
                    )
                })?;
            let bin_path = slownik::pobierz_sciezke_indeksu().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "Nie można ustalić ścieżki indeksu binarnego w XDG.",
                )
            })?;

            println!("Budowanie indeksu z {:?} do {:?}...", csv_path, bin_path);
            let start = std::time::Instant::now();
            indeks::zbuduj_indeks(&csv_path, &bin_path)?;
            let elapsed = start.elapsed();
            println!("Indeks został pomyślnie zbudowany w {:?}!", elapsed);
        }
    }

    Ok(())
}
