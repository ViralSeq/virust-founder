# Founder virus workflow

## Requirements

1. ruby (>= 3.0)
2. iqtree3 (>= 3.0)
3. hyhpy (>=2.5.0)
4. hyphy-analyses (https://github.com/veg/hyphy-analyses)
5. R (>= 4.1)
6. R package pharngorn (>= 2.12) and other required packages
7. LANL Gene Cutter tool.

## Steps

1. Concatenate all SGA sequences (in FASTA format) from each patient into one fasta file.

```bash
cd <DIRECTION_OF_FASTA_FILES>
cat *.fasta > combined_sga.fasta
```

2. use `locator` tool from Ruby package 'viral_seq' to check the combined SGA sequences and regenerate sense strand sequences if needed.

```bash
locator -i combined_sga.fasta
```

- `combined_sga.direction.fasta` file has all the sequences in sense strand.
- check `combined_sga.csv` for locator information.
- TODO! Add a filtering script for bad sequences.

3. Use LANL [Gene Cutter tool](https://www.hiv.lanl.gov/content/sequence/GENE_CUTTER/cutter.html) to codon-align and trim sequences only to Env CDS.

- Make sure the checkbox for `Codon align the region` is selected.
- Regions(s) to align and extract: Select `Env CDS`.
- No need to insert references.

4. Remove sequences with

- frameshift and premature stop codons in Env CDS.

- incomplete Env sequence.

- TODO! Add a filtering script.

5. !Important: Remove last stop codon from the Env CDS (last 3 nt if they are TAA, TAG or TGA), need a script.

6. Generate phylogenetic tree using `iqtree3` with trimmed ENV nt sequences (with bad sequences removed). In this example we just call this alignment file as `alignment.fasta`

```bash
iqtree3 -s alignment.fasta -m GTR+G -asr -nt AUTO
```

- a .treefile (along with .iqtree and .state files) will be generated.

7. Use HYPHY for standard MG94 fit (condon substitution), the FigMG94.bf can be found in the `hyphy-analyses` repo.

```bash
hyphy /path/to/FitMG94.bf --alignment alignment.fasta --tree alignment.treefile --save-fit alignment.fit
```

8. Generate ansenstral sequences using HYPHY. The ancenstral batch file can found in the `hyphy-analyses` repo.

```bash
hyphy /path/to/AncestralSequences.bf --fit alignment.fit --output alignment.json
```

- You can find the ancenstral sequences in the json file. Use the sequence at the root of the tree as the founder virus.

- ["joint"]["ancestral_sequences"]["root"]

9. Alternatively, use Phangorn in R to generate ancenstral sequences. See the R example.

10. Add stop codon to the end of the sequence. Can always use `TAA`. need a script.


# Setup and Usage

## Install

```bash
sh ./installer.sh
```

Ensures installation of conda, conda environment for founder, and any other required dependencies.

## Run

```bash
conda run -n founder cargo run ...
```

Or to enter Founder conda environment for multiple runs:

```bash
conda activate founder
cargo run ...
```

## File Structure

```text
project-root/
├── in/                     # Input sequence files
│   ├── patient_file_1.fasta
│   ├── patient_file_2.fasta
│   └── patient_file_n.fasta
│
├── work/                   # Intermediate pipeline artifacts
│   ├── combined_sga.fasta            # Combined sequences
│   ├── combined_sga.direction.fasta  # Locator results
│   ├── genecutter.aa.fasta           # SimpleGC amino acid output
│   ├── genecutter.na.fasta           # SimpleGC nucleotide output
│   ├── alignment.fasta               # Filtered GeneCutter results
│   ├── alignment.treefile            # iqtree3 output
│   └── alignment.fit                 # FitMG94 output
│
└── results/                 # Final outputs
    ├── alignment.json                # AncestralSequences output
    └── alignment.treefile            # iqtree3 output
```