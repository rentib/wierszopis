# Wierszopis - Project Instructions & Context

## Project Overview

**Wierszopis** is a CLI and TUI (Terminal User Interface) tool written in **Rust**, designed to assist poets and writers in analyzing and writing Polish poetry.

### Key Features & Architecture
- **Word Analysis (`wyraz`):** Detailed analysis of phonemes, vowels, and syllables for a single word.
- **Line & Syllable Analysis (`analizuj`):** Syllable counting, rhyme detection using an advanced phonetic engine (`internal/rym`), phonetic substitution cost calculation, and flexible matching for masculine/feminine rhymes.
- **Interactive Rhyme Finder (`rymuj`):** Fast, responsive TUI for real-time rhyme searching.
- **Dictionary Management:** Support for external rhyme dictionaries without bloating binary size.
- **Index Building (`buduj-indeks`):** Builds optimized binary dictionary indices for fast startup and lookup.
- **Core Library & Multi-Binary Architecture:** The core poetry analysis, phonetics, and dictionary logic are extracted into a reusable Rust library crate (`src/lib.rs`), while interfaces (such as CLI/TUI in `src/bin/wierszopis/` and future GUI applications) are implemented as separate binaries using the core library.

## Building, Running, and Testing

Since Wierszopis is a Rust project, standard `cargo` commands apply:

### Build
```bash
cargo build --release
```

### Run
```bash
cargo run -- <komenda> [opcje]
```

Examples:
```bash
cargo run -- wyraz miłość
cargo run -- analizuj wiersz.txt
cargo run -- rymuj
cargo run -- buduj-indeks --plik slownik.csv
```

### Test
```bash
cargo test
```

## Configuration & Dictionary Location

Wierszopis expects dictionary files under the XDG data directory:
- CSV dictionary: `$XDG_DATA_HOME/wierszopis/slownik.csv` (or `~/.local/share/wierszopis/slownik.csv`)
- Binary index: `$XDG_DATA_HOME/wierszopis/slownik.bin`

## Development Conventions

- **Language:** Rust (edition 2021+).
- **Code Style:** Follow standard `cargo fmt` and `cargo clippy` lints.
- **Error Handling:** Idiomatic Rust error handling (`Result`, custom error types or `anyhow`/`thiserror`).
- **Modularity:** Clear separation between CLI/TUI interfaces, phonetic engine (`internal/rym`), and dictionary/indexing logic.
