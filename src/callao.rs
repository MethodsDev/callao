use std::path::PathBuf;

use pyo3::prelude::*;

use futures::TryStreamExt;
use hashbrown::HashMap;
use log::{debug, info, warn};
use noodles::sam::record::data::field::{value::Array, Value};

use crate::io::{make_reader, make_writers};

#[tokio::main]
async fn async_split_bam(
    input_bam: PathBuf,
    barcode_map: HashMap<(u16, u16), PathBuf>,
) -> PyResult<()> {
    const BC: [u8; 2] = [b'b', b'c'];

    info!("Reading from {}", input_bam.display());
    let (mut reader, header) = make_reader(&input_bam).await?;

    // check that lima was run on this file, otherwise it won't have the bc tag
    // (or it will but they'll be something else)
    if !header.programs().contains_key("lima") {
        warn!("lima not found in BAM header, callao may not work properly!");
    }

    // get a unique list of paths by collecting into a set
    let output_bams = barcode_map.values().cloned().collect();

    let mut writers = make_writers(&header, output_bams).await?;

    debug!("Reading records from BAM");
    let mut records = reader.records(&header);

    while let Some(record) = records.try_next().await? {
        if let Some(Value::Array(Array::UInt16(bc_val))) = record.data().get(&BC) {
            if bc_val.len() != 2 {
                debug!("bc array with length {}, that's weird!", bc_val.len());
            } else {
                let ij = (bc_val[0], bc_val[1]);
                if let Some(p) = barcode_map.get(&ij) {
                    if let Some(writer) = writers.get_mut(p) {
                        writer.write_record(&header, &record).await?;
                    }
                }
            }
        }
    }

    for writer in writers.values_mut() {
        writer.shutdown().await?;
    }

    info!("Done");
    Ok(())
}

/// Splits a BAM tagged by lima, based on the barcode pairs in the bc tag
///
/// ## Arguments
///  * `input_bam` - a BAM file, with index values in the `bc` tag (0-indexed)
///  * `barcode_map` - a mapping from barcode pair (i, j) to the BAM path those
///                    records should be written to. Multiple pairs can be written to
///                    one file, if they point to the same value.
#[pyfunction]
fn split_bam(input_bam: PathBuf, barcode_map: HashMap<(u16, u16), PathBuf>) -> PyResult<()> {
    async_split_bam(input_bam, barcode_map)
}

/// Rust module to split a Lima BAM by different indexes
#[pymodule]
#[pyo3(name = "_callao")]
fn callao(_py: Python, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();

    m.add_function(wrap_pyfunction!(split_bam, m)?)?;
    Ok(())
}
