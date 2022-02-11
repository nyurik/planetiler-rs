use std::ops::AddAssign;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::mpsc::channel;

use crate::utils::{advise_cache, spawn_stats_aggregator, OptAdvice};
use anyhow::Error;
use clap::Parser;
use osmnodecache::{CacheStore, DenseFileCacheOpts};
use osmpbf::{BlobDecode, BlobReader};
use rayon::iter::{ParallelBridge, ParallelIterator};

#[derive(Debug, Parser)]
pub struct OptsCacheNodes {
    /// Input pbf data.
    pbf_file: PathBuf,

    /// File for planet-size node cache.
    node_cache: PathBuf,

    #[clap(flatten)]
    advice: OptAdvice,
}

#[derive(Clone, Debug)]
struct Stats {
    pub node_count: usize,
    pub min_node_id: i64,
    pub max_node_id: i64,
    pub min_latitude: f64,
    pub max_latitude: f64,
    pub min_longitude: f64,
    pub max_longitude: f64,
}

impl Stats {
    pub fn add_node(&mut self, node_id: i64, lat: f64, lng: f64) {
        *self = Self {
            node_count: self.node_count + 1,
            min_node_id: self.min_node_id.min(node_id),
            max_node_id: self.max_node_id.max(node_id),
            min_latitude: self.min_latitude.min(lat),
            max_latitude: self.max_latitude.max(lat),
            min_longitude: self.min_longitude.min(lng),
            max_longitude: self.max_longitude.max(lng),
        };
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            node_count: 0,
            min_node_id: i64::MAX,
            max_node_id: i64::MIN,
            min_latitude: 0.0,
            max_latitude: 0.0,
            min_longitude: 0.0,
            max_longitude: 0.0,
        }
    }
}

impl AddAssign for Stats {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            node_count: self.node_count + other.node_count,
            min_node_id: self.min_node_id.min(other.min_node_id),
            max_node_id: self.max_node_id.max(other.max_node_id),
            min_latitude: self.min_latitude.min(other.min_latitude),
            max_latitude: self.max_latitude.max(other.max_latitude),
            min_longitude: self.min_longitude.min(other.min_longitude),
            max_longitude: self.max_longitude.max(other.max_longitude),
        };
    }
}

pub fn run(args: OptsCacheNodes) -> Result<(), Error> {
    parse_nodes(&args.pbf_file, args.node_cache, &args.advice)?;
    Ok(())
}

/// Returns offset of the first block with ways or relations
pub fn parse_nodes(
    pbf_file: &Path,
    node_cache_file: PathBuf,
    advice: &OptAdvice,
) -> Result<u64, Error> {
    let cache = DenseFileCacheOpts::new(node_cache_file)
        .page_size(10 * 1024 * 1024 * 1024)
        .open()?;

    advise_cache(&cache, advice)?;

    let first_way_block = AtomicU64::new(u64::MAX);
    let (sender, receiver) = channel();
    let stats_collector = spawn_stats_aggregator("Nodes to cache file", receiver);

    // Read PBF file using multiple threads, and in each thread store node positions into cache
    BlobReader::from_path(pbf_file)?.par_bridge().for_each_with(
        (cache, sender),
        |(dfc, sender), blob| {
            let mut cache = dfc.get_accessor();
            let mut stats = Stats::default();
            let blob = blob.unwrap();
            if let BlobDecode::OsmData(block) = blob.decode().unwrap() {
                let mut blob_has_ways = false;
                for group in block.groups() {
                    for node in group.nodes() {
                        let lat = node.lat();
                        let lon = node.lon();
                        cache.set_lat_lon(node.id() as usize, lat, lon);
                        stats.add_node(node.id(), lat, lon);
                    }
                    for node in group.dense_nodes() {
                        let lat = node.lat();
                        let lon = node.lon();
                        cache.set_lat_lon(node.id() as usize, lat, lon);
                        stats.add_node(node.id(), lat, lon);
                    }
                    // TBD: is this the quickest way to test for empty?
                    if group.ways().next().is_some() || group.relations().next().is_some() {
                        blob_has_ways = true;
                    }
                }
                if blob_has_ways {
                    first_way_block.fetch_min(blob.offset().unwrap().0, Relaxed);
                }
            };
            sender.send(stats).unwrap();
        },
    );

    stats_collector.join().unwrap();

    Ok(first_way_block.load(Relaxed))
}
