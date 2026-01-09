use bio::io::fasta;
use std::fs;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CombineError {
    #[error("Failed to read input directory: {path}")] ReadDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to open FASTA file: {path}")] OpenFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Invalid FASTA record in file: {path}")] FastaParse {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Invalid sequence characters in file {path} record {id}: {bad}")] InvalidSeqChars {
        path: PathBuf,
        id: String,
        bad: String,
    },

    #[error("Failed to create output file: {path}")] CreateOutput {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to create output directory: {path}")] CreateOutputDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to write output FASTA")] WriteOutput(#[source] io::Error),
}

/// Example validator: DNA IUPAC + gap + N (customize as you like).
fn validate_dna(seq: &[u8]) -> Result<(), String> {
    // Accept uppercase/lowercase; allow '-' gap.
    // Expand if you want full IUPAC (R,Y,S,W,K,M,B,D,H,V,N).
    fn ok(b: u8) -> bool {
        matches!(
            b.to_ascii_uppercase(),
            b'A' |
                b'C' |
                b'G' |
                b'T' |
                b'N' |
                b'-' |
                b'R' |
                b'Y' |
                b'S' |
                b'W' |
                b'K' |
                b'M' |
                b'B' |
                b'D' |
                b'H' |
                b'V'
        )
    }

    let mut bad = Vec::new();
    for &b in seq {
        if !ok(b) {
            bad.push(b);
        }
    }

    if bad.is_empty() {
        Ok(())
    } else {
        // show unique bad chars
        bad.sort_unstable();
        bad.dedup();
        let bad_str = bad
            .into_iter()
            .map(|b| (b as char).to_string())
            .collect::<Vec<_>>()
            .join("");
        Err(bad_str)
    }
}

pub fn combine_fasta(
    input: &PathBuf,
    combined_fasta_pathbuf: &PathBuf
) -> Result<(), CombineError> {
    if let Some(parent) = combined_fasta_pathbuf.parent() {
        fs::create_dir_all(parent).map_err(|e| CombineError::CreateOutputDir {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    // Find fasta files
    let mut fasta_files: Vec<PathBuf> = fs
        ::read_dir(&input)
        .map_err(|e| CombineError::ReadDir {
            path: input.to_path_buf(),
            source: e,
        })?
        .filter_map(|ent| ent.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext|
                    matches!(ext.to_ascii_lowercase().as_str(), "fa" | "fasta" | "fna" | "faa")
                )
        })
        .collect();

    fasta_files.sort();

    // Create output writer
    let out_file = fs::File::create(combined_fasta_pathbuf).map_err(|e| CombineError::CreateOutput {
        path: combined_fasta_pathbuf.to_path_buf(),
        source: e,
    })?;
    let mut writer = fasta::Writer::new(io::BufWriter::new(out_file));
    // Optional: wrap lines at 60/70; leave None to keep as-is.
    writer.set_linewrap(Some(60)); // docs show set_linewrap()  [oai_citation:3‡Docs.rs](https://docs.rs/bio/latest/bio/io/fasta/struct.Writer.html?utm_source=chatgpt.com)

    // Stream records from each input into output
    for path in fasta_files {
        let file = fs::File::open(&path).map_err(|e| CombineError::OpenFile {
            path: path.clone(),
            source: e,
        })?;
        let reader = fasta::Reader::new(io::BufReader::new(file));

        for rec in reader.records() {
            let record = rec.map_err(|e| CombineError::FastaParse {
                path: path.clone(),
                source: e,
            })?;

            // Validate sequence characters (DNA example)
            if let Err(bad) = validate_dna(record.seq()) {
                return Err(CombineError::InvalidSeqChars {
                    path: path.clone(),
                    id: record.id().to_string(),
                    bad,
                });
            }

            // Write the record as-is
            writer.write_record(&record).map_err(CombineError::WriteOutput)?;
        }
    }

    Ok(())
}
