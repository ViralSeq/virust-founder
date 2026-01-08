use crate::helper::{ combine_fasta::combine_fasta, error::FounderError };

pub fn founder(input: String, output: String, keep_original: bool) -> Result<(), FounderError> {
    println!(
        "Running founder pipeline with parameters\nInput: {}\nOutput: {}\nKeep Originals {}",
        input,
        output,
        keep_original
    );

    // combine all .fasta in directory to combined_sga.fasta
    println!("Combining sequences.");
    let _combined_fasta = match combine_fasta(input, output) {
        Ok(combined_fasta) => combined_fasta,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    // run `locator -i combined_sga.fasta`

    // send combined_sga.direction.fasta to SimpleGC
    // curl region=env (=ENV CDS , codon_align defaults to yes, insert_ref defualts to no)
    // parse AA/NA results to genecutter.fasta

    // Remove sequences with frameshift and premature stop codons in genecutter.fasta

    // iqtree3 -s alignment.fasta -m GTR+G -asr -nt AUTO
    // alignment.treefile will be generated

    // hyphy /hyphy-analyses/FitMG94/FitMG94.bf --alignment alignment.fasta --tree alignment.treefile --save-fit alignment.fit

    // hyphy /hyphy-analyses/AncestralSequences/AncestralSequences.bf --fit alignment.fit --output alignment.json

    Ok(())
}
