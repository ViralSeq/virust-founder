use std::path::PathBuf;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FounderError {
    #[error("Failed to combine fasta files.")] FastaCombineFailed,
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
