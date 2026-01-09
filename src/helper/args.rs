use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Args {
    pub input: PathBuf,
    pub output: PathBuf,
    pub output_work: PathBuf,
    pub output_results: PathBuf,
    pub keep_original: bool,
    pub use_phanghorn: bool,
}

impl Args {
    pub fn new(
        input: impl Into<PathBuf>,
        output: impl Into<PathBuf>,
        keep_original: bool,
        use_phanghorn: bool
    ) -> Self {
        let input = input.into();
        let output = output.into();

        let output_work = output.join("work");
        let output_results = output.join("results");

        Self {
            input,
            output,
            output_work,
            output_results,
            keep_original,
            use_phanghorn,
        }
    }
}
