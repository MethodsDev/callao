# callao

`callao` is a command-line tool for processing PacBio BAM files. It will split a `lima`-tagged BAM into separate BAMs for the different barcodes.

The input to this tool is the output of running

```
lima [--no-clip] <INPUT.BAM> <BARCODES.fa> <OUTPUT.BAM>
```

The `--no-clip` option is optional but recommended if the next step will be running `skera`.

### Usage

Run `pip install .` to compile the Rust extension and put the `callao` command on the path.

```
Usage: callao [OPTIONS] [INDEXES]...

  This script splits a BAM file previously tagged with lima into separate
  indexes. As input it needs the same fasta file that was used, to identify
  proper and improper pairs of barcodes.

  The adapter pairs should look like A_1, Q_1, A_2, Q_2, etc. where A and Q
  are the names for the 5' and 3' adapters, and 1, 2, ... denotes the sample
  index.

  In normal mode, only reads with an A-Q index pair will be written. When
  artifacts are included, pairs with A-A or Q-Q will be included.

  If a series of INDEXES is provided, only those pairs will be written.
  Otherwise, it will create a BAM for every pair in the barcode file.

Options:
  --input-bam PATH      BAM file with barcode tags from lima  [required]
  --output-stem FILE    Basename for outputs. Will append suffix per index
                        [required]
  --barcode-fasta PATH  fasta file of indexed adapters used for Lima
                        [required]
  --include-artifacts   Include artifacts (A-A and Q-Q)
  -v, --verbosity LVL   Either CRITICAL, ERROR, WARNING, INFO or DEBUG
  --help                Show this message and exit.
```
