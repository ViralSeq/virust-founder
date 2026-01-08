use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum FounderError {
    #[error("Failed to combine fasta files.")] FastaCombineFailed,
}
