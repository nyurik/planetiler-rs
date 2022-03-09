use std::cmp::Ordering;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::prelude::FileExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering::Relaxed};
use std::sync::mpsc::channel;

use crate::utils::{advise_cache, spawn_stats_aggregator, NodeStats, OptAdvice};
use anyhow::Error;
use anyhow::Result;
use clap::Parser;
use osmnodecache::{CacheStore, DenseFileCacheOpts};
use osmpbf::{Blob, BlobDecode, BlobReader};
use pariter::{scope, IteratorExt};
use rayon::iter::{ParallelBridge, ParallelIterator};
use separator::{usize, Separatable};
use zerocopy::{insert_vec_zeroed, AsBytes, LittleEndian, U64};

#[derive(Debug, Parser)]
pub struct OptsCacheNodes2 {
    /// Input pbf data.
    pub pbf_file: PathBuf,

    /// File for planet-size node cache.
    pub node_cache: PathBuf,
}

pub fn run(args: OptsCacheNodes2) -> Result<()> {
    parse_nodes(&args.pbf_file, args.node_cache)?;
    Ok(())
}

/// Create a flat node cache file using block approach
/// Returns offset of the first block with ways or relations
pub fn parse_nodes(pbf_file: &Path, node_cache_file: PathBuf) -> Result<u64> {
    let mut cache = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(node_cache_file)?;

    let first_way_block = AtomicU64::new(u64::MAX);
    let sum_block_size = AtomicUsize::default();
    let block_count = AtomicU64::default();
    scope(|s| {
        let reader = BlobReader::from_path(pbf_file)?;
        reader
            .parallel_map_scoped(s, |blob| parse_blob(&first_way_block, &blob.unwrap()))
            .for_each(|v| match v {
                Some((data, starts_at)) => {
                    cache.seek(SeekFrom::Start(starts_at as u64)).unwrap();
                    cache.write(data.as_bytes()).unwrap();
                    sum_block_size.fetch_add(data.len(), Relaxed);
                    block_count.fetch_add(1, Relaxed);
                }
                None => {}
            });
        Result::<()>::Ok(())
    })
    .unwrap()?;

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

pub fn parse_blob(
    first_way_block: &AtomicU64,
    blob: &Blob,
) -> Option<(Vec<U64<LittleEndian>>, usize)> {
    if let BlobDecode::OsmData(block) = blob.decode().unwrap() {
        let mut blob_has_ways = false;
        let mut first_index = 0_usize;
        let mut last_index = 0_usize;
        let mut result: Vec<U64<LittleEndian>> = Vec::new(); // 1*1024*1024 ?
        let mut add_node = |id, lat, lon| {
            assert!(id > last_index);
            last_index = id;
            if first_index == 0 {
                first_index = id;
            }
            let relative_id = id - first_index;
            let exists = result.len();
            let needed = relative_id - exists;
            if needed > 0 {
                insert_vec_zeroed(&mut result, exists, needed);
            }
            result.push(U64::new((lat as u64) << 32 | lon as u64));
        };
        for group in block.groups() {
            for node in group.nodes() {
                add_node(
                    node.id() as usize,
                    node.decimicro_lat(),
                    node.decimicro_lon(),
                );
            }
            for node in group.dense_nodes() {
                add_node(
                    node.id() as usize,
                    node.decimicro_lat(),
                    node.decimicro_lon(),
                );
            }
            if group.ways().next().is_some() || group.relations().next().is_some() {
                blob_has_ways = true;
            }
        }
        if blob_has_ways {
            first_way_block.fetch_min(blob.offset().unwrap().0, Relaxed);
        }
        Some((result, first_index))
    } else {
        None
    }
}
