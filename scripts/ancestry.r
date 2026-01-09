#!/usr/bin/env Rscript

suppressPackageStartupMessages({
  library(phangorn)
  library(ape)
})

args <- commandArgs(trailingOnly = TRUE)

tryCatch({

  if (length(args) != 3) {
    stop("Usage: Rscript ancestry.r <alignment.fasta> <treefile> <output.fasta>")
  }

  alignment_path <- args[1]
  tree_path <- args[2]
  output_path <- args[3]

  alignment <- read.phyDat(alignment_path, format = "fasta")
  tree <- read.tree(tree_path)

  fit <- pml(tree, alignment)
  fit <- optim.pml(
    fit,
    model = "GTR",
    optInv = FALSE,
    optGamma = TRUE,
    optEdge = FALSE,
    control = pml.control(trace = 0)
  )

  ancestral <- ancestral.pml(fit, type = "ml")

  write.phyDat(
    as.phyDat(ancestral),
    file = output_path,
    format = "fasta"
  )

}, error = function(e) {
  writeLines(e$message, con = stderr())
  quit(status = 1, save = "no")
})