use std::path::PathBuf;

use crate::helper::{
    args::Args,
    combine_fasta::combine_fasta,
    error::FounderError,
    gene_cutter::process_gene_cutter,
    run_cmd::run_cmd,
};

pub fn founder(input: String, output: String, keep_original: bool) -> Result<(), FounderError> {
    println!(
        "Running founder pipeline with parameters\nInput: {}\nOutput: {}\nKeep Originals {}\n",
        input,
        output,
        keep_original
    );

    let args: Args = Args::new(input, output, keep_original);

    // combine all .fasta in directory to combined_sga.fasta
    println!("Combining sequences.");
    println!("Input directory: {}", args.input.display());
    let combined_fasta = match combine_fasta(&args) {
        Ok(combined_fasta) => combined_fasta,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };
    println!("Combined output location: {}\n", combined_fasta.display());

    let locator_output_pathbuf = combined_fasta.with_extension("alignment").with_extension("fasta");
    println!("Running locator -i {}", locator_output_pathbuf.display());
    run_cmd("location", ["-i", combined_fasta.to_string_lossy().as_ref()]).map_err(|e| {
        FounderError::CommandFailed {
            program: "location".to_string(),
            source: e,
        }
    })?;
    if !locator_output_pathbuf.exists() {
        return Err(FounderError::LocatorOutputMissing {
            path: locator_output_pathbuf,
        });
    }
    println!("Locator output created: {}\n", locator_output_pathbuf.display());

    // send combined_sga.direction.fasta to SimpleGC
    // curl region=env (=ENV CDS , codon_align defaults to yes, insert_ref defualts to no)
    // parse AA/NA results to genecutter.fasta

    let gc_aa_output_pathbuf = args.output_work.join("genecutter.aa.fasta");
    let gc_na_output_pathbuf = args.output_work.join("genecutter.na.fasta");

    println!("Running GeneCutter");
    if
        let Err(e) = process_gene_cutter(
            locator_output_pathbuf,
            gc_aa_output_pathbuf,
            gc_na_output_pathbuf
        )
    {
        return Err(FounderError::GeneCutterFailed {
            message: e.to_string(),
        });
    }
    println!("GeneCutter success.\n");

    return Ok(());

    let frameshift_output_pathbuf = args.output_work.clone();
    // TODO frameshift and filter Gene Cutter results.
    // Remove sequences with frameshift and premature stop codons in genecutter.fasta

    println!("Running iqtree.  Frameshift output path - {}", frameshift_output_pathbuf.display());
    let iqtree_output_location = args.output_work.clone().join("alignment.treefile");
    run_cmd("iqtree", [
        "-s",
        //TODO return this to as_os_str() when it's an actual path
        &frameshift_output_pathbuf.display().to_string(),
        "-m",
        "GTR+G".as_ref(),
        "-asr",
        "-nt",
        "AUTO",
    ]).map_err(|e| FounderError::CommandFailed {
        program: "iqtree".to_string(),
        source: e,
    })?;

    let path_to_hyphy_analyses = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path_to_fitmg94 = path_to_hyphy_analyses.join("FitMG94").join("FitMG94.bf");
    let path_to_ancestral_sequences = path_to_hyphy_analyses
        .join("AncestralSequences")
        .join("AncestralSequences.bf");

    println!("Running FitMG94.");
    let fit_output_pathbuf = args.output_work.clone().join("alignment.fit");
    run_cmd("hyphy", [
        path_to_fitmg94.as_os_str(),
        "--alignment".as_ref(),
        frameshift_output_pathbuf.as_os_str(),
        "--tree".as_ref(),
        iqtree_output_location.as_os_str(),
        "--save-fit".as_ref(),
        fit_output_pathbuf.as_os_str(),
    ]).map_err(|e| FounderError::CommandFailed {
        program: "hyphy".to_string(),
        source: e,
    })?;

    // TODO add an option to use Phanghorn instead of AncestralSequences here

    println!("Running Ancestral Sequences");
    let ancestral_sequences_output_pathbuf = args.output_results.join("alignment.json");
    run_cmd("hyphy", [
        path_to_ancestral_sequences.as_os_str(),
        "--fit".as_ref(),
        fit_output_pathbuf.as_os_str(),
        "--output".as_ref(),
        ancestral_sequences_output_pathbuf.as_os_str(),
    ]).map_err(|e| FounderError::CommandFailed {
        program: "hyphy".to_string(),
        source: e,
    })?;

    // TODO add stop codon

    Ok(())
}
