use std::path::PathBuf;

use hashbrown::{HashMap, HashSet};
use noodles::sam::header::record::value::{
    map::{program::tag, Program},
    Map,
};
use noodles::{bam, bgzf, sam};
use tokio::fs::{File, OpenOptions};
use tokio::io;

const NAME: &str = "callao";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn add_pg(cli_cmd: String, header: &mut sam::Header) -> () {
    let program = Map::<Program>::builder().insert(tag::NAME, NAME);

    // note: this is not guaranteed to be correct
    let program = if let Some(last_pg) = header.programs().keys().last() {
        program.insert(tag::PREVIOUS_PROGRAM_ID, last_pg.clone())
    } else {
        program
    };

    let program = program
        .insert(tag::VERSION, VERSION)
        .insert(tag::COMMAND_LINE, cli_cmd)
        .build()
        .unwrap();
    header.programs_mut().insert(NAME.into(), program);
}

pub(crate) async fn make_reader(
    cli_cmd: String,
    input_bam: &PathBuf,
) -> io::Result<(bam::AsyncReader<bgzf::AsyncReader<File>>, sam::Header)> {
    let mut reader = File::open(input_bam).await.map(bam::AsyncReader::new)?;
    let mut header = reader.read_header().await?;

    add_pg(cli_cmd, &mut header);

    Ok((reader, header))
}

pub(crate) async fn make_writer(
    header: &sam::Header,
    output_bam: &PathBuf,
) -> io::Result<bam::AsyncWriter<bgzf::AsyncWriter<File>>> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(output_bam)
        .await?;
    let mut writer = bam::AsyncWriter::new(file);

    writer.write_header(&header).await?;

    Ok(writer)
}

/// creates a hashmap of BAM writers
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
