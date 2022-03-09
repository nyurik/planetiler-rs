use std::cmp::Ordering;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::prelude::FileExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering::Relaxed};
use std::sync::mpsc::channel;

use crate::cache_nodes2::parse_blob;
use crate::utils::{advise_cache, spawn_stats_aggregator, NodeStats, OptAdvice};
use crate::OptsCacheNodes2;
use anyhow::Error;
use anyhow::Result;
use clap::Parser;
use osmnodecache::{CacheStore, DenseFileCacheOpts};
use osmpbf::{Blob, BlobDecode, BlobReader};
use pariter::{scope, IteratorExt};
use rayon::iter::{ParallelBridge, ParallelIterator};
use separator::{usize, Separatable};
use zerocopy::{insert_vec_zeroed, AsBytes, LittleEndian, U64};

pub fn run(args: OptsCacheNodes2) -> Result<()> {
    parse_nodes(&args.pbf_file, args.node_cache)?;
    Ok(())
}

/// Create a flat node cache file using block approach
/// Returns offset of the first block with ways or relations
pub fn parse_nodes(pbf_file: &Path, node_cache_file: PathBuf) -> Result<u64> {
    let first_way_block = AtomicU64::new(u64::MAX);
    let sum_block_size = AtomicUsize::default();
    let block_count = AtomicU64::default();

    let reader = BlobReader::from_path(pbf_file)?;
    reader.par_bridge().for_each(|blob| {
        if let Some((data, starts_at)) = parse_blob(&first_way_block, &blob.unwrap()) {
            let mut cache = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&node_cache_file)
                .unwrap();
            cache.seek(SeekFrom::Start(starts_at as u64)).unwrap();
            cache.write(data.as_bytes()).unwrap();
            sum_block_size.fetch_add(data.len(), Relaxed);
            block_count.fetch_add(1, Relaxed);
        }
    });

    let sum = sum_block_size.load(Relaxed);
    let count = block_count.load(Relaxed);
    println!(
        "Saved {} bytes, {} blocks, average {} per block",
        sum.separated_string(),
        count.separated_string(),
        (sum as u64 / count).separated_string()
    );

    Ok(first_way_block.load(Relaxed))
}
