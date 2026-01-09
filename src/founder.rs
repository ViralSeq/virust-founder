use std::process::Command;

use bio::bio_types::annot::loc;

use crate::helper::{
    args::Args,
    combine_fasta::combine_fasta,
    error::FounderError,
    gene_cutter::process_gene_cutter,
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
    let combined_fasta = match combine_fasta(&args) {
        Ok(combined_fasta) => combined_fasta,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    let locator_output_pathbuf = combined_fasta.with_extension("alignment").with_extension("fasta");

    println!("Running locator -i {}", locator_output_pathbuf.display());
    let _locator_command_output = Command::new("locator")
        .arg("-i")
        .arg(combined_fasta.to_string_lossy().as_ref())
        // .arg("-o")
        // .arg(&locator_output_pathbuf)
        .output()
        .unwrap();

    // println!("status: {}", locator_command_output.status);
    // println!("stdout:\n{}", String::from_utf8_lossy(&locator_command_output.stdout));
    // println!("stderr:\n{}", String::from_utf8_lossy(&locator_command_output.stderr));

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

    // Remove sequences with frameshift and premature stop codons in genecutter.fasta

    // iqtree3 -s alignment.fasta -m GTR+G -asr -nt AUTO
    // alignment.treefile will be generated

    // hyphy /hyphy-analyses/FitMG94/FitMG94.bf --alignment alignment.fasta --tree alignment.treefile --save-fit alignment.fit

    // hyphy /hyphy-analyses/AncestralSequences/AncestralSequences.bf --fit alignment.fit --output alignment.json

    Ok(())
}
