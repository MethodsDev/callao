use std::path::PathBuf;

use log::{debug, info};
use pyo3::prelude::*;

use hashbrown::{HashMap, HashSet};
use std::fs::File;
use std::io;

use noodles::bam;
use noodles::bgzf;
use noodles::sam;
use noodles::sam::record::data::field::value::Array;
use noodles::sam::record::data::field::Value;

fn make_writer(
    header: &sam::Header,
    output_bam: &PathBuf,
) -> io::Result<bam::Writer<bgzf::Writer<File>>> {
    let mut writer = bam::writer::Builder::default().build_from_path(output_bam)?;

    writer.write_header(&header)?;

    Ok(writer)
}

/// creates a hashmap of BAM writers,
fn make_writers(
    header: &sam::Header,
    output_bams: HashSet<PathBuf>,
) -> io::Result<HashMap<PathBuf, bam::Writer<bgzf::Writer<File>>>> {
    let mut output_writers = HashMap::new();

    for v in output_bams.iter() {
        if let Ok(new_writer) = make_writer(&header, &v) {
            output_writers.insert(v.clone(), new_writer);
        }
    }

    Ok(output_writers)
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
    info!("Reading from {}", input_bam.display());

    const BC: [u8; 2] = [b'b', b'c'];

    info!("Reading from {}", input_bam.display());
    let mut reader = File::open(input_bam).map(bam::Reader::new)?;
    let header = reader.read_header()?;

    // get a unique list of paths by collecting into a set
    let output_bams = barcode_map.values().cloned().collect();

    let mut writers = make_writers(&header, output_bams)?;

    debug!("Reading records from BAM");
    for result in reader.records(&header) {
        let record = result?;

        if let Some(Value::Array(Array::UInt16(bc_val))) = record.data().get(&BC) {
            if bc_val.len() != 2 {
                debug!("bc array with length {}, that's weird!", bc_val.len());
            } else {
                let ij = (bc_val[0], bc_val[1]);
                if let Some(p) = barcode_map.get(&ij) {
                    if let Some(writer) = writers.get_mut(p) {
                        writer.write_record(&header, &record)?;
                    }
                }
            }
        }
    }

    info!("Done");
    Ok(())
}

/// Rust module to split a Lima BAM by different indexes
#[pymodule]
#[pyo3(name = "_callao")]
fn callao(_py: Python, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();

    m.add_function(wrap_pyfunction!(split_bam, m)?)?;
    Ok(())
}
