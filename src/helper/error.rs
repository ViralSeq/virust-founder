use std::io;
use std::path::PathBuf;
use thiserror::Error;

use crate::helper::combine_fasta::CombineError;

#[derive(Error, Debug)]
pub enum FounderError {
    #[error("Failed to combine fasta files")]
    CombineFastaFailed {
        #[from]
        source: CombineError,
    },
    #[error("Locator output file missing: {path}")] LocatorOutputMissing { path: PathBuf },
    #[error("SimpleGC network or processing error: {message}")]
    GeneCutterFailed { message: String },
    #[error("Failed to run command: {program}")]
    CommandFailed {
        program: String,
        #[source]
        source: io::Error,
    },
}
