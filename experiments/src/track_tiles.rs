use rand::{thread_rng, Rng};
use rayon::prelude::*;
use std::path::PathBuf;

use anyhow::Error;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct OptsTrackTiles {
    /// Tile tracking database file.
    db_file: PathBuf,

    /// Number of simultaneous batches to write.
    batches: u32,

    /// Number of features in each batch.
    count: u32,

    /// For each feature ID create random tiles between 1 and this value.
    max_tiles_per_id: u16,

    /// Use DB compression
    #[clap(short, long)]
    zip: bool,

    /// Flash every N seconds
    #[clap(short, long, default_value_t = 10)]
    flash: u64,

    /// Cache capacity, in MB
    #[clap(short, long, default_value_t = 1048576)]
    cache: u64,
}

pub fn run(args: OptsTrackTiles) -> Result<(), Error> {
    let db = sled::Config::new()
        .flush_every_ms(Some(args.flash * 1000))
        .path(args.db_file)
        .use_compression(args.zip)
        .cache_capacity(args.cache * 1024)
        .open()?;
    (0..args.batches)
        .collect::<Vec<_>>()
        .par_iter()
        .map(|batch_id| {
            let mut rng = thread_rng();
            let mut batch = sled::Batch::default();
            (0..args.count)
                .map(|v| {
                    let key = (*batch_id as u64 * v as u64).to_be_bytes();
                    let tile_ids: u32 = rng.gen_range(1..=args.max_tiles_per_id as u32);
                    let value: Vec<_> = (0..tile_ids).flat_map(|v| v.to_be_bytes()).collect();
                    let value2 = as_u8_slice(value.as_slice());
                    let value3 = sled::IVec::from(value2);
                    batch.insert(&key, value3);
                })
                .for_each(drop);
            db.apply_batch(batch).unwrap();
        })
        .for_each(drop);
    println!("Created {} entries", args.count * args.batches);
    Ok(())
}

// https://stackoverflow.com/a/29042896/177275
fn as_u8_slice<T>(v: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * std::mem::size_of::<T>())
    }
}
