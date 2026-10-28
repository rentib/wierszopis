/// Moduł indeksowania binarnego słownika.
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct WpisSlownika {
    pub slowo: String,
    pub forma_podstawowa: String,
    pub czesc_mowy: String,
    pub cechy_gramatyczne: String,
    pub pelny_tag: String,
    pub liczba_sylab: usize,
}

#[derive(Debug, Clone)]
pub struct IndeksCache {
    pub entries: Vec<WpisSlownika>,
    pub buckets: HashMap<String, Vec<usize>>,
}

fn write_string(w: &mut impl Write, s: &str) -> io::Result<()> {
    let bytes = s.as_bytes();
    let len = bytes.len() as u32;
    w.write_all(&len.to_le_bytes())?;
    w.write_all(bytes)?;
    Ok(())
}

fn read_string(r: &mut impl Read) -> io::Result<String> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut bytes = vec![0u8; len];
    r.read_exact(&mut bytes)?;
    String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn write_usize(w: &mut impl Write, val: usize) -> io::Result<()> {
    let val64 = val as u64;
    w.write_all(&val64.to_le_bytes())?;
    Ok(())
}

fn read_usize(r: &mut impl Read) -> io::Result<usize> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf) as usize)
}

impl WpisSlownika {
    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        write_string(w, &self.slowo)?;
        write_string(w, &self.forma_podstawowa)?;
        write_string(w, &self.czesc_mowy)?;
        write_string(w, &self.cechy_gramatyczne)?;
        write_string(w, &self.pelny_tag)?;
        write_usize(w, self.liczba_sylab)?;
        Ok(())
    }

    fn deserialize(r: &mut impl Read) -> io::Result<Self> {
        let slowo = read_string(r)?;
        let forma_podstawowa = read_string(r)?;
        let czesc_mowy = read_string(r)?;
        let cechy_gramatyczne = read_string(r)?;
        let pelny_tag = read_string(r)?;
        let liczba_sylab = read_usize(r)?;
        Ok(Self {
            slowo,
            forma_podstawowa,
            czesc_mowy,
            cechy_gramatyczne,
            pelny_tag,
            liczba_sylab,
        })
    }
}

impl IndeksCache {
    pub fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        write_usize(w, self.entries.len())?;
        for entry in &self.entries {
            entry.serialize(w)?;
        }
        write_usize(w, self.buckets.len())?;
        for (k, v) in &self.buckets {
            write_string(w, k)?;
            write_usize(w, v.len())?;
            for &idx in v {
                write_usize(w, idx)?;
            }
        }
        Ok(())
    }

    pub fn deserialize(r: &mut impl Read) -> io::Result<Self> {
        let entries_len = read_usize(r)?;
        let mut entries = Vec::with_capacity(entries_len);
        for _ in 0..entries_len {
            entries.push(WpisSlownika::deserialize(r)?);
        }

        let buckets_len = read_usize(r)?;
        let mut buckets = HashMap::with_capacity(buckets_len);
        for _ in 0..buckets_len {
            let k = read_string(r)?;
            let v_len = read_usize(r)?;
            let mut v = Vec::with_capacity(v_len);
            for _ in 0..v_len {
                v.push(read_usize(r)?);
            }
            buckets.insert(k, v);
        }

        Ok(Self { entries, buckets })
    }
}

pub fn zbuduj_indeks(sciezka_csv: &Path, sciezka_binarnie: &Path) -> io::Result<()> {
    let start_pre = std::time::Instant::now();

    let file = File::open(sciezka_csv)?;
    let reader = BufReader::new(file);
    let mut wpisy = Vec::new();

    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        if idx == 0 && line.starts_with("slowo,") {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 5 {
            let slowo = parts[0].trim().to_string();
            let forma_podstawowa = parts[1].trim().to_string();
            let czesc_mowy = parts[2].trim().to_string();
            let cechy_gramatyczne = parts[3].trim().to_string();
            let pelny_tag = parts[4].trim().to_string();
            if !slowo.is_empty() {
                let liczba_sylab = crate::fonetyka::zlicz_sylaby(&slowo);
                wpisy.push(WpisSlownika {
                    slowo,
                    forma_podstawowa,
                    czesc_mowy,
                    cechy_gramatyczne,
                    pelny_tag,
                    liczba_sylab,
                });
            }
        }
    }

    let mut buckets: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, entry) in wpisy.iter().enumerate() {
        let s = crate::slowo::nowe_słowo(&entry.slowo);
        let klucz = s.klucz_bucketa();
        buckets.entry(klucz).or_default().push(idx);
    }

    let czas_prekomputacji = start_pre.elapsed();

    let cache = IndeksCache {
        entries: wpisy,
        buckets,
    };

    let start_write = std::time::Instant::now();
    if let Some(parent) = sciezka_binarnie.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let out_file = File::create(sciezka_binarnie)?;
    let mut writer = BufWriter::new(out_file);
    cache.serialize(&mut writer)?;
    let czas_zapisywania = start_write.elapsed();

    println!("=== TELEMETRIA INDEKSU ===");
    println!("Czas prekomputacji: {:?}", czas_prekomputacji);
    println!("Czas zapisywania binarki: {:?}", czas_zapisywania);
    println!("Liczba bucketów: {}", cache.buckets.len());
    println!("Statystyki bucketów (klucz / samogłoski, hash, liczba elementów):");
    
    let mut sorted_buckets: Vec<(&String, &Vec<usize>)> = cache.buckets.iter().collect();
    sorted_buckets.sort_by_key(|b| b.0);

    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    for (klucz, indices) in sorted_buckets {
        hasher.write(klucz.as_bytes());
        let hash_val = hasher.finish();
        println!("  - Klucz/Samogłoski: {:<10} | Hash: {:<16} | Elementy: {}", klucz, hash_val, indices.len());
    }
    println!("==========================");

    Ok(())
}
