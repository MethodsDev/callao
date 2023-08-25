# callao

`callao` is a command-line tool for processing PacBio BAM files. It will split a lima-tagged BAM into its different indexes.

The input to this tool is the output of running

```
lima [--no-clip] <INPUT.BAM> <BARCODES.fa> <OUTPUT.BAM>
```

### Usage

Run `pip install .` to compile the Rust extension and put the `callao` command on the path.

```bash
Usage: callao [OPTIONS] [INDEXES]...

  This script splits a BAM file previously tagged with lima into separate
  indexes. As input it needs the same fasta file that was used, to identify
  proper and improper pairs of barcodes.

  Currently it supports adapter pairs like A_1 and Q_1, where A and Q are any
  string names for the 5' and 3' adapters, and 1 is the sample index.

  In normal mode, only reads with an A-Q index pair will be output. When
  artifacts are included, pairs with A-A or Q-Q will be included.

Options:
  -v, --verbosity LVL  Either CRITICAL, ERROR, WARNING, INFO or DEBUG
  --input-bam PATH     BAM file with barcode tags from lima
  --output-stem FILE   Basename for outputs. Will append suffix per index
  --index-fasta PATH   fasta file of indexed adapters used for Lima
  --include-artifacts  Include artifacts (A-A and Q-Q)
  --help               Show this message and exit.

```
