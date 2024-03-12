use std::path::PathBuf;

use hashbrown::{HashMap, HashSet};
use noodles::sam::header::record::value::{
    map::{Program, Tag},
    Map,
};
use noodles::{bam, bgzf, sam};
use tokio::fs::{File, OpenOptions};
use tokio::io;

const NAME: &str = "callao";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn add_pg(cli_cmd: String, header: &mut sam::Header) -> () {
    let pn = match Tag::try_from([b'P', b'N']) {
        Ok(Tag::Other(tag)) => tag,
        _ => unreachable!(),
    };
    let vn = match Tag::try_from([b'V', b'N']) {
        Ok(Tag::Other(tag)) => tag,
        _ => unreachable!(),
    };
    let pp = match Tag::try_from([b'P', b'P']) {
        Ok(Tag::Other(tag)) => tag,
        _ => unreachable!(),
    };
    let cl = match Tag::try_from([b'C', b'L']) {
        Ok(Tag::Other(tag)) => tag,
        _ => unreachable!(),
    };

    let program = Map::<Program>::builder().insert(pn, NAME);

    let program = if let Some(last_pg) = header.programs().iter().last() {
        program.insert(pp, last_pg.0.clone())
    } else {
        program
    };

    let program = program
        .insert(vn, VERSION)
        .insert(cl, cli_cmd)
        .build()
        .unwrap();
    header
        .programs_mut()
        .insert(String::from("callao").into(), program);
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
