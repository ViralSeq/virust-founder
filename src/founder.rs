use std::path::PathBuf;

use crate::helper::{
    args::Args,
    combine_fasta::combine_fasta,
    error::FounderError,
    gene_cutter::process_gene_cutter,
    run_cmd::run_cmd,
};

pub fn founder(
    input: String,
    output: String,
    keep_original: bool,
    use_phanghorn: bool
) -> Result<(), FounderError> {
    println!(
        "Running founder pipeline with parameters\nInput: {}\nOutput: {}\nKeep Originals {}\n",
        input,
        output,
        keep_original
    );

    let args: Args = Args::new(input, output, keep_original, use_phanghorn);

    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let combined_fasta_pathbuf = args.output_work.join("combined_sga.fasta");
    let locator_output_pathbuf = combined_fasta_pathbuf
        .with_extension("alignment")
        .with_extension("fasta");
    let gc_aa_output_pathbuf = args.output_work.join("genecutter.aa.fasta");
    let gc_na_output_pathbuf = args.output_work.join("genecutter.na.fasta");
    let frameshift_output_pathbuf = args.output_work.join("alignment.fasta");
    let iqtree_output_location = args.output_work.join("alignment.treefile");
    let fit_output_pathbuf = args.output_work.join("alignment.fit");
    let ancestral_sequences_output_pathbuf = args.output_results.join("alignment.json");

    let path_to_hyphy_analyses = project_root.join("hyphy-analyses");
    let path_to_fitmg94 = path_to_hyphy_analyses.join("FitMG94").join("FitMG94.bf");
    let path_to_ancestral_sequences = path_to_hyphy_analyses
        .join("AncestralSequences")
        .join("AncestralSequences.bf");

    let ancestry_phanghorn_pathbuf = project_root.join("scripts").join("ancestry.r");

    // combine all .fasta in directory to combined_sga.fasta
    println!("Combining sequences.");
    println!("Input directory: {}", args.input.display());

    println!("Output file: {}", combined_fasta_pathbuf.display());
    combine_fasta(&args.input, &combined_fasta_pathbuf).map_err(|e| {
        FounderError::CombineFastaFailed { source: e }
    })?;
    println!("Running locator -i {}", locator_output_pathbuf.display());
    run_cmd("locator", ["-i", combined_fasta_pathbuf.to_string_lossy().as_ref()]).map_err(|e| {
        FounderError::CommandFailed {
            program: "locator".to_string(),
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

    println!("Running GeneCutter");
    process_gene_cutter(locator_output_pathbuf, gc_aa_output_pathbuf, gc_na_output_pathbuf).map_err(
        |e| FounderError::GeneCutterFailed { message: e.to_string() }
    )?;
    println!("GeneCutter success.\n");

    if true {
        return Ok(());
    }
    // TODO frameshift and filter Gene Cutter results.
    // Remove sequences with frameshift and premature stop codons in genecutter.fasta

    println!("Running iqtree.  Frameshift output path - {}", frameshift_output_pathbuf.display());
    let iqtree_args = vec![
        "-s".into(),
        frameshift_output_pathbuf.as_os_str().to_os_string(),
        "-m".into(),
        "GTR+G".into(),
        "-asr".into(),
        "-nt".into(),
        "AUTO".into()
    ];
    run_cmd("iqtree", iqtree_args).map_err(|e| FounderError::CommandFailed {
        program: "iqtree".to_string(),
        source: e,
    })?;

    println!("Running FitMG94.");
    let fit_args = vec![
        path_to_fitmg94.as_os_str().to_os_string(),
        "--alignment".into(),
        frameshift_output_pathbuf.as_os_str().to_os_string(),
        "--tree".into(),
        iqtree_output_location.as_os_str().to_os_string(),
        "--save-fit".into(),
        fit_output_pathbuf.as_os_str().to_os_string()
    ];
    run_cmd("hyphy", fit_args).map_err(|e| FounderError::CommandFailed {
        program: "hyphy".to_string(),
        source: e,
    })?;

    if args.use_phanghorn {
        println!("Running Phanghorn ancestry.R");
        let phanghorn_args = vec![
            ancestry_phanghorn_pathbuf.as_os_str().to_os_string(),
            frameshift_output_pathbuf.as_os_str().to_os_string(),
            iqtree_output_location.as_os_str().to_os_string(),
            ancestral_sequences_output_pathbuf.as_os_str().to_os_string()
        ];
        run_cmd("Rscript", phanghorn_args).map_err(|e| FounderError::CommandFailed {
            program: "Rscript".to_string(),
            source: e,
        })?;
    } else {
        // use hyphy
        println!("Running Ancestral Sequences");
        let ancestral_args = vec![
            path_to_ancestral_sequences.as_os_str().to_os_string(),
            "--fit".into(),
            fit_output_pathbuf.as_os_str().to_os_string(),
            "--output".into(),
            ancestral_sequences_output_pathbuf.as_os_str().to_os_string()
        ];
        run_cmd("hyphy", ancestral_args).map_err(|e| FounderError::CommandFailed {
            program: "hyphy".to_string(),
            source: e,
        })?;
    }

    // TODO add stop codon

    Ok(())
}
