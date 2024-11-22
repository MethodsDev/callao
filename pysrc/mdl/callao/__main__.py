import importlib.metadata
import logging
import sys
from collections import defaultdict
from pathlib import Path

import click
from mdl.log import verbosity_config_option

from ._callao import split_bam

__version__ = importlib.metadata.version("callao")
log = logging.getLogger(__package__)


@click.command()
@click.option(
    "--input-bam",
    required=True,
    type=click.Path(exists=True, path_type=Path, allow_dash=True),
    help="BAM file with barcode tags from lima",
)
@click.option(
    "--output-stem",
    required=True,
    type=click.Path(dir_okay=False, path_type=Path),
    help="Basename for outputs. Will append suffix per index",
)
@click.option(
    "--barcode-fasta",
    required=True,
    type=click.Path(path_type=Path),
    help="fasta file of indexed adapters used for Lima",
)
@click.option(
    "--include-artifacts", is_flag=True, help="Include artifacts (A-A and Q-Q)"
)
@verbosity_config_option(log, "mdl.log")
@click.version_option(__version__)
@click.argument("indexes", type=int, nargs=-1)
def cli(
    input_bam: Path,
    output_stem: Path,
    barcode_fasta: Path,
    include_artifacts: bool = False,
    indexes: list[int] = None,
):
    """This script splits a BAM file previously tagged with lima into separate indexes.
    As input it needs the same fasta file that was used, to identify proper and improper
    pairs of barcodes.

    The adapter pairs should look like A_1, Q_1, A_2, Q_2, etc. where A and Q are the
    names for the 5' and 3' adapters, and 1, 2, ... denotes the sample index.

    In normal mode, only reads with an A-Q index pair will be written. When artifacts
    are included, pairs with A-A or Q-Q will be included.

    If a series of INDEXES is provided, only those pairs will be written. Otherwise, it
    will create a BAM for every pair in the barcode file.
    """

    cli_cmd = f"callao {' '.join(sys.argv[1:])}"
    log.debug(f"Invoked with: {cli_cmd}")
    log.debug(f"Reading barcodes from {barcode_fasta}")
    barcodes = []
    with open(barcode_fasta) as fh:
        for line in fh:
            if line.startswith(">"):
                # convert to pair of (adapter, index)
                barcodes.append(line.strip().split()[0][1:].split("_")[:2])

    try:
        # these are 1-indexed
        barcodes = [(bc, int(ix)) for bc, ix in barcodes]
    except ValueError as exc:
        log.error(
            "Failed to extract indexes from barcode names. Is this the right file?",
            exc_info=exc,
        )
        sys.exit(1)

    # 0-indexed, used in lima tags
    bc_to_i = {(bc, ix): i for i, (bc, ix) in enumerate(barcodes)}

    # should be a pair for each index, i.e. {1: (A_1, Q_1)}
    ix_to_ij = defaultdict(list)
    for bc, ix in barcodes:
        ix_to_ij[ix].append(bc_to_i[bc, ix])

    if not all(len(v) == 2 for v in ix_to_ij.values()):
        log.error("Should only have two adapters (A and Q) for each index")
        sys.exit(1)

    # lima tag will be in sorted order
    ix_to_ij = {ix: tuple(sorted(v)) for ix, v in ix_to_ij.items()}

    include_set = set(ix_to_ij)
    if indexes:
        include_set &= set(indexes)

    log.debug(f"Writing to {len(include_set)} output BAMs")

    output_bams = {ix: output_stem.with_suffix(f".{ix}.bam") for ix in include_set}

    # map from 0-index pairs to output bams
    bc_mapping = {ix_to_ij[ix]: output_bams[ix] for ix in include_set}

    log.debug("Including artifact pairings in output")
    if include_artifacts:
        for i, (bc, ix) in enumerate(barcodes):
            if ix in include_set:
                bc_mapping[i, i] = output_bams[ix]

    split_bam(cli_cmd, input_bam, bc_mapping)
