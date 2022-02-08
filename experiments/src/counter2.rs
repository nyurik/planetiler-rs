use anyhow::Error;
use std::path::PathBuf;

use crate::StructOpt;

#[allow(dead_code)]
#[derive(Debug, StructOpt)]
pub struct Counter2 {
    /// Input pbf data.
    pbf_file: PathBuf,

    /// File for planet-size node cache.
    node_cache: PathBuf,
}

pub fn run(args: Counter2) -> Result<(), Error> {
    println!("Not implemented. Params = {:#?}", args);
    Ok(())
}
