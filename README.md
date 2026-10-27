# Wierszopis

**Wierszopis** to narzędzie CLI oraz TUI (Terminal User Interface) napisane w
Rust, jako pomoc w analizowaniu i pisaniu wierszy.

## Funkcjonalności

- **Analiza słowa (`wyraz`):** Szczegółowa analiza głosek, samogłosek i sylab dla pojedynczego słowa.
- **Analiza wersów i sylab (`analizuj`):** Sprawdzanie liczby sylab w wersach oraz wykrywanie rymów na podstawie zaawansowanego silnika fonetycznego (`internal/rym`) z uwzględnieniem kosztów zamian fonetycznych oraz elastycznego dopasowywania rymów męskich i żeńskich.
- **Interaktywny rymownik (`rymuj`):** Szybki i responsywny terminalowy interfejs TUI pozwalający na wyszukiwanie rymów na żywo.
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
