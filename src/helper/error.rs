use std::io;
use std::path::PathBuf;

use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CombineError {
    #[error("(Combination) Failed to read input directory: {path}")] ReadDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("(Combination) Failed to open FASTA file: {path}")] OpenFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("(Combination) Invalid FASTA record in file: {path}")] FastaParse {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error(
        "(Combination) Invalid sequence characters in file {path} record {id}: {bad}"
    )] InvalidSeqChars {
        path: PathBuf,
        id: String,
        bad: String,
    },

    #[error("(Combination) Failed to create output file: {path}")] CreateOutput {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("(Combination) Failed to create output directory: {path}")] CreateOutputDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("(Combination) Failed to write output FASTA")] WriteOutput(#[source] io::Error),
}

#[derive(Debug, Error)]
pub enum GeneCutterError {
    #[error("(GC) Failed to open locator output file: {path}")] OpenLocator {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("(GC) GeneCutter request failed")] Request {
        #[source]
        source: reqwest::Error,
    },

    #[error("(GC) Failed to read GeneCutter response body")] ResponseRead {
        #[source]
        source: reqwest::Error,
    },

    #[error("(GC) GeneCutter request failed: {status}")] RequestFailed {
        status: StatusCode,
    },

    #[error("(GC) Failed to create output directory: {path}")] CreateOutputDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("(GC) Failed to create output file: {path}")] CreateOutputFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("(GC) No FASTA blocks found in GeneCutter response")]
    NoFastaBlocks,

    #[error("(GC) Found NA FASTA but no AA FASTA in response")]
    MissingAa,

    #[error("(GC) Found AA FASTA but no NA FASTA in response")]
    MissingNa,

    #[error("(GC) Invalid FASTA record in GeneCutter response")] FastaParse {
        #[source]
        source: io::Error,
    },

    #[error("(GC) Failed to write output FASTA")] WriteOutput {
        #[source]
        source: io::Error,
    },
}

#[derive(Error, Debug)]
pub enum FounderError {
    #[error("(Main) Failed to combine fasta files")] CombineFastaFailed {
        #[from]
        source: CombineError,
    },
    #[error("(Main) Locator output file missing: {path}")] LocatorOutputMissing {
        path: PathBuf,
    },
    #[error("(Main) SimpleGC network or processing error: {message}")] GeneCutterFailed {
        message: String,
    },
    #[error("(Main) Failed to run command: {program}")] CommandFailed {
        program: String,
        #[source]
        source: io::Error,
    },
}
