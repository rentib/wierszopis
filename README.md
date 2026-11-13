# Wierszopis

**Wierszopis** to narzędzie CLI oraz TUI napisane w Rust, jako pomoc w
analizowaniu i pisaniu wierszy.

## Funkcjonalności

- **Analiza słowa (`wyraz`):** Szczegółowa analiza głosek, samogłosek i sylab dla pojedynczego słowa.
- **Analiza wersów i sylab (`analizuj`):** Sprawdzanie liczby sylab w wersach oraz wykrywanie rymów na podstawie silnika fonetycznego z uwzględnieniem kosztów zamian fonetycznych oraz elastycznego dopasowywania rymów męskich i żeńskich.
- **Interaktywny rymownik (`rymuj`):** Szybki i responsywny terminalowy interfejs TUI pozwalający na wyszukiwanie rymów na żywo.
- **Serwer LSP (`wierszopis-lsp`):** Integracja z protokołem Language Server Protocol (LSP) zapewniająca diagnostykę sylab w czasie rzeczywistym, podpowiedzi po najechaniu kursorem (hover) oraz autouzupełnianie dla edytorów takich jak VS Code i Neovim.
- **Zarządzanie słownikiem:** Obsługa zewnętrznego słownika rymów bez obciążania rozmiaru pliku wykonywalnego.
- **Budowanie indeksu:** Budowanie indeksu z danego słownika w celu zoptymalizowania czasu ładowania.

## Użycie

```bash
./wierszopis <komenda> [opcje]

Komendy:
    wyraz <słowo>         Pokazuje szczegółową analizę głosek i sylab pojedynczego słowa.
    analizuj <ścieżka>    Analizuje sylaby, rymy i koloruje wersy ze standardowego wejścia lub pliku, jeśli został podany.
    rymuj [opcje]         Uruchamia interaktywny rymownik TUI
        -- workers <n>    Liczba workerów równoległych (domyślnie maksymalna liczba wątków).
    buduj-indeks [opcje]  Buduje i zapisuje szybki indeks binarny słownika.
        --workers <n>     Liczba workerów równoległych (domyślnie maksymalna liczba wątków)
        --plik <ścieżka>  Ścieżka do pliku słownika (opcjonalnie)
    pomoc                 Wyświetla tę pomoc.
```

## Konfiguracja i lokalizacja słownika

Wierszopis ładuje słownik z `$XDG_DATA_HOME/wierszopis/slownik.csv`, albo
z `$XDG_DATA_HOME/wierszopis/slownik.bin` jeśli wcześniej został zbudowany
indeks.

## Serwer LSP (`wierszopis-lsp`)

Aby zbudować serwer LSP:
```bash
cargo build --release --features lsp --bin wierszopis-lsp
```
Plik binarny znajdziesz w `target/release/wierszopis-lsp`.

### Konfiguracja i dyrektywy LSP w komentarzach

Serwer LSP Wierszopisa wspiera konfigurację i linting bezpośrednio w edytorze za pomocą komentarzy dyrektyw umieszczanych **nad** tekstem, którego dotyczą. Wszystkie parametry i klucze są wyłącznie w języku polskim.

* **Składnia dyrektywy:** `# wierszopis: <parametry>` lub `# @wierszopis: <parametry>`
* **Obsługiwane parametry (po polsku):**
  * `sylaby` – oczekiwana liczba sylab lub liczba sylab ze średniówką (np. `sylaby: 13`, `sylaby: 7+6` oczekująca średniówki po 7. sylabie, bądź lista cykliczna `sylaby: 11, 8, 11, 8`).
  * `rymy` – rymy dla strofy (o długości co najmniej 4 liter, np. `aabb` dla parzystych, `abab` dla krzyżowych/nieparzystych, `abba` dla okalających).
  * `akcenty` - czy podkreślać akcentowane sylaby (true/false)

* **Uwaga dotycząca podpisów:** Podpisy (dedykacje lub podpisy na końcu wiersza) są automatycznie wykrywane i wykluczane ze zliczania sylab oraz lintingu metrycznego.

**Przykład użycia w pliku tekstowym / poezji:**
```poet
# wierszopis: sylaby = 5+6,8; rymy = abab
"Pójdźcie, o dziatki, pójdźcie wszystkie razem
Za miasto, pod słup na wzgórek,
Tam przed cudownym klęknijcie obrazem,
Pobożnie zmówcie paciórek.
```

### Funkcje LSP:
1. **Real-time Linting (Diagnostyka):** Automatyczne ostrzeżenia o niezgodności liczby sylab z dyrektywą oraz o braku wymaganych rymów w schemacie.
2. **Inlay Hints:** Pokazywanie schematu akcentów na końcu wersu (np. `[ssSs]`).
3. **Semantic Tokens & Document Color:** Kolorowanie rymów i grup wersów, z zachowaniem czystości białych znaków (kolorowany jest wyłącznie tekst bez wcięć i spacji).
4. **Hover & Completion:** Szczegółowa analiza fonetyczna słów po najechaniu kursorem oraz inteligentne autouzupełnianie ze słownika rymów.

## Netlify

Jest instancja GUI hostowana na netlify.
[![Netlify Status](https://api.netlify.com/api/v1/badges/451105ba-ce29-4289-a1dd-8fadfbb7c119/deploy-status)](https://app.netlify.com/projects/wierszopis/deploys)
