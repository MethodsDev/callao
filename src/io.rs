
use std::path::PathBuf;

use hashbrown::{HashMap, HashSet};
use noodles::{bam, bgzf, sam};
use tokio::io;
use tokio::fs::{File, OpenOptions};


pub(crate) async fn make_writer(
    header: &sam::Header,
    output_bam: &PathBuf,
) -> io::Result<bam::AsyncWriter<bgzf::AsyncWriter<File>>> {
    let file = OpenOptions::new().write(true).create(true).open(output_bam).await?;
    let mut writer = bam::AsyncWriter::new(file);

    writer.write_header(&header).await?;
    writer
        .write_reference_sequences(header.reference_sequences())
        .await?;

    Ok(writer)
}

/// creates a hashmap of BAM writers,
pub(crate) async fn make_writers(
    header: &sam::Header,
    output_bams: HashSet<PathBuf>,
) -> io::Result<HashMap<PathBuf, bam::AsyncWriter<bgzf::AsyncWriter<File>>>> {
    let mut output_writers = HashMap::new();

    for v in output_bams.iter() {
        let new_writer = make_writer(&header, &v).await?;
        output_writers.insert(v.clone(), new_writer);
    }

    Ok(output_writers)
}
