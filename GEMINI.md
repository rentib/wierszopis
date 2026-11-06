# Wierszopis - Instrukcje i kontekst projektu

## Przegląd projektu

**Wierszopis** to narzędzie CLI oraz TUI napisanego w języku **Rust**, zaprojektowane, aby pomagać poetom i pisarzom w analizowaniu oraz tworzeniu polskojęzycznej poezji.

### Główne funkcje i architektura

* **Analiza słowa (`wyraz`):** Szczegółowa analiza fonemów, samogłosek i sylab dla pojedynczego słowa.
* **Analiza wersu i sylab (`analizuj`):** Liczenie sylab, wykrywanie rymów przy użyciu zaawansowanego silnika fonetycznego (`internal/rym`), obliczanie kosztu substytucji fonetycznej oraz elastyczne dopasowywanie rymów męskich i żeńskich.
* **Interaktywny wyszukiwarka rymów (`rymuj`):** Szybki i responsywny interfejs TUI do wyszukiwania rymów w czasie rzeczywistym.
* **Zarządzanie słownikiem:** Obsługa zewnętrznych słowników rymów bez powiększania rozmiaru pliku binarnego.
* **Budowanie indeksu (`buduj-indeks`):** Tworzenie zoptymalizowanych binarnych indeksów słownika zapewniających szybkie uruchamianie i wyszukiwanie.
* **Biblioteka rdzenna i architektura wielobinaryczna:** Główna logika analizy poezji, fonetyki i słownika została wydzielona do wielorazowej biblioteki w języku Rust (paczka typu library w `src/lib.rs`), podczas gdy interfejsy (takie jak CLI/TUI w `src/bin/wierszopis/` oraz przyszłe aplikacje GUI) są zaimplementowane jako osobne pliki binarne korzystające z rdzennej biblioteki.

## Budowanie, uruchamianie i testowanie

Ponieważ Wierszopis jest projektem w języku Rust, mają do niego zastosowanie standardowe polecenia `cargo`:

### Budowanie

```bash
cargo build --release

```

### Uruchamianie

```bash
cargo run -- <komenda> [opcje]
```

Przykłady:

```bash
cargo run -- wyraz miłość
cargo run -- analizuj wiersz.txt
cargo run -- rymuj
cargo run -- buduj-indeks --plik slownik.csv
```

### Testowanie

```bash
cargo test
```

## Konfiguracja i lokalizacja słownika

Wierszopis oczekuje plików słownika w katalogu danych XDG:

* Słownik CSV: `$XDG_DATA_HOME/wierszopis/slownik.csv` (lub `~/.local/share/wierszopis/slownik.csv`)
* Indeks binarny: `$XDG_DATA_HOME/wierszopis/slownik.bin`

## Konwencje programistyczne

* **Język:** Rust (edycja 2021+).
* **Styl kodu:** Zgodność ze standardowymi linitami `cargo fmt` oraz `cargo clippy`.
* **Obsługa błędów:** Idiomatyczna obsługa błędów w Rust (`Result`, własne typy błędów lub `anyhow`/`thiserror`).
* **Modułowość:** Wyraźny podział pomiędzy interfejsami CLI/TUI, silnikiem fonetycznym (`internal/rym`) oraz logiką słownika i indeksowania.
