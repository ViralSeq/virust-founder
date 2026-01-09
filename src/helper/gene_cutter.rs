use std::fs::{ self, File };
use std::io::{ self, Cursor };
use std::path::PathBuf;

use bio::io::fasta;
use reqwest::blocking::{ multipart, Client };
use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GeneCutterError {
    #[error("Failed to open locator output file: {path}")] OpenLocator {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("GeneCutter request failed")] Request {
        #[source]
        source: reqwest::Error,
    },

    #[error("Failed to read GeneCutter response body")] ResponseRead {
        #[source]
        source: reqwest::Error,
    },

    #[error("GeneCutter request failed: {status}")] RequestFailed {
        status: StatusCode,
    },

    #[error("Failed to create output directory: {path}")] CreateOutputDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to create output file: {path}")] CreateOutputFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("No FASTA blocks found in GeneCutter response")]
    NoFastaBlocks,

    #[error("Found NA FASTA but no AA FASTA in response")]
    MissingAa,

    #[error("Found AA FASTA but no NA FASTA in response")]
    MissingNa,

    #[error("Invalid FASTA record in GeneCutter response")] FastaParse {
        #[source]
        source: io::Error,
    },

    #[error("Failed to write output FASTA")] WriteOutput {
        #[source]
        source: io::Error,
    },
}

pub fn process_gene_cutter(
    locator_output_pathbuf: PathBuf,
    gc_aa_output_pathbuf: PathBuf,
    gc_na_output_pathbuf: PathBuf
) -> Result<(), GeneCutterError> {
    let client = Client::new();

    let file = File::open(&locator_output_pathbuf).map_err(|e| GeneCutterError::OpenLocator {
        path: locator_output_pathbuf.clone(),
        source: e,
    })?;

    let form = multipart::Form
        ::new()
        .part("seq_upload", multipart::Part::reader(file).file_name("sample.fasta"))
        .text("region", "env")
        .text("return_format", "fasta");

    let response = client
        .post("https://www.hiv.lanl.gov/cgi-bin/GENE_CUTTER/simpleGC")
        .multipart(form)
        .send()
        .map_err(|e| GeneCutterError::Request { source: e })?;

    let status = response.status();
    let body = response.text().map_err(|e| GeneCutterError::ResponseRead { source: e })?;

    // println!("Status: {}", status);
    // println!("Body: {}", body);

    if !status.is_success() {
        return Err(GeneCutterError::RequestFailed { status });
    }

    // Ensure output dirs exist
    if let Some(p) = gc_aa_output_pathbuf.parent() {
        fs::create_dir_all(p).map_err(|e| GeneCutterError::CreateOutputDir {
            path: p.to_path_buf(),
            source: e,
        })?;
    }
    if let Some(p) = gc_na_output_pathbuf.parent() {
        fs::create_dir_all(p).map_err(|e| GeneCutterError::CreateOutputDir {
            path: p.to_path_buf(),
            source: e,
        })?;
    }

    let aa_file = File::create(&gc_aa_output_pathbuf).map_err(|e| {
        GeneCutterError::CreateOutputFile {
            path: gc_aa_output_pathbuf.clone(),
            source: e,
        }
    })?;
    let na_file = File::create(&gc_na_output_pathbuf).map_err(|e| {
        GeneCutterError::CreateOutputFile {
            path: gc_na_output_pathbuf.clone(),
            source: e,
        }
    })?;
    let mut aa_writer = fasta::Writer::new(aa_file);
    let mut na_writer = fasta::Writer::new(na_file);

    let (aa_count, na_count) = split_gene_cutter_fasta(&body, &mut aa_writer, &mut na_writer)?;
    if aa_count == 0 {
        return Err(GeneCutterError::MissingAa);
    }
    if na_count == 0 {
        return Err(GeneCutterError::MissingNa);
    }

    println!("GeneCutter results:");
    println!("NA Count: {}", aa_count);
    println!("AA Count: {}", na_count);

    Ok(())
}

/// Extract FASTA blocks from the response and write AA/NA records via bio::io::fasta.
/// Classification is by sequence alphabet: if all sequence chars are IUPAC nucleotides,
/// it is NA; otherwise it is AA.
fn split_gene_cutter_fasta(
    body: &str,
    aa_writer: &mut fasta::Writer<File>,
    na_writer: &mut fasta::Writer<File>
) -> Result<(usize, usize), GeneCutterError> {
    let blocks = extract_fasta_blocks(body);

    if blocks.is_empty() {
        return Err(GeneCutterError::NoFastaBlocks);
    }

    let mut aa_count = 0usize;
    let mut na_count = 0usize;
    for block in blocks {
        let cursor = Cursor::new(block);
        let reader = fasta::Reader::new(cursor);
        for rec in reader.records() {
            let record = rec.map_err(|e| GeneCutterError::FastaParse { source: e })?;
            if is_nucleotide_seq(record.seq()) {
                na_writer
                    .write_record(&record)
                    .map_err(|e| GeneCutterError::WriteOutput { source: e })?;
                na_count += 1;
            } else {
                aa_writer
                    .write_record(&record)
                    .map_err(|e| GeneCutterError::WriteOutput { source: e })?;
                aa_count += 1;
            }
        }
    }

    Ok((aa_count, na_count))
}

/// Extracts all FASTA blocks from arbitrary text (HTML or plain).
/// A FASTA block starts at a line beginning with '>' and continues until the next '>' line
/// or until we hit a long stretch of non-sequence / blank lines.
fn extract_fasta_blocks(text: &str) -> Vec<String> {
    let mut blocks: Vec<String> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    let mut in_block = false;

    for line in text.lines() {
        let trimmed = line.trim_end();

        if trimmed.starts_with('>') {
            if in_block && !current.is_empty() {
                blocks.push(current.join("\n") + "\n");
                current.clear();
            }
            in_block = true;
            current.push(trimmed.to_string());
            continue;
        }

        if in_block {
            // Allow empty lines inside block, but if we see a "definitely not FASTA" line,
            // we can choose to end the block. We keep it simple: accept sequence-ish lines.
            // If it's empty, keep it (some FASTA have blank lines).
            if trimmed.is_empty() {
                current.push(String::new());
            } else {
                // keep lines that look like sequence or FASTA-ish wrapping
                // (we'll validate later)
                current.push(trimmed.to_string());
            }
        }
    }

    if in_block && !current.is_empty() {
        blocks.push(current.join("\n") + "\n");
    }

    // Filter out blocks that are just headers with no sequence
    blocks
        .into_iter()
        .filter(|b| {
            let mut lines = b.lines();
            let first = lines.next().unwrap_or("");
            first.starts_with('>') && lines.any(|l| l.chars().any(|c| c.is_ascii_alphabetic()))
        })
        .collect()
}

/// Return true if the FASTA block looks like nucleotide sequence.
/// Accepts IUPAC nucleotide codes + '-' '.' and '*' (some tools include '*' for stops in NA too).
fn is_nucleotide_seq(seq: &[u8]) -> bool {
    if seq.is_empty() {
        return false;
    }

    seq.iter().all(|b| {
        match b.to_ascii_uppercase() {
            // Standard + IUPAC ambiguity codes
            | b'A'
            | b'C'
            | b'G'
            | b'T'
            | b'U'
            | b'R'
            | b'Y'
            | b'K'
            | b'M'
            | b'S'
            | b'W'
            | b'B'
            | b'D'
            | b'H'
            | b'V'
            | b'N'
            // Common gap/format chars
            | b'-'
            | b'.' => true,
            _ => false,
        }
    })
}
