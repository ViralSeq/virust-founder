use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum FounderError {
    #[error("Failed to combine fasta files.")] FastaCombineFailed,
    #[error("Locator output file missing: {path}")] LocatorOutputMissing { path: PathBuf },
    #[error("SimpleGC network or processing error: {message}")]
    GeneCutterFailed { message: String },
}
