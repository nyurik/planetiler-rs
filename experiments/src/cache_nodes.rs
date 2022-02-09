use std::ops::AddAssign;
use std::path::PathBuf;
use std::sync::mpsc::channel;

use crate::utils::spawn_stats_aggregator;
use anyhow::Error;
use clap::Parser;
use osmnodecache::{CacheStore, DenseFileCacheOpts};
use osmpbf::{BlobDecode, BlobReader};
use rayon::iter::{ParallelBridge, ParallelIterator};

#[derive(Debug, Parser)]
/// Create a node cache
pub struct CacheNodes {
    /// Input pbf data.
    pbf_file: PathBuf,

    /// File for planet-size node cache.
    node_cache: PathBuf,
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

pub fn run(args: CacheNodes) -> Result<(), Error> {
    parse_nodes(&args.pbf_file, args.node_cache)
}

pub fn parse_nodes(pbf_file: &PathBuf, node_cache_file: PathBuf) -> Result<(), Error> {
    let cache = DenseFileCacheOpts::new(node_cache_file)
        .page_size(10 * 1024 * 1024 * 1024)
        .open()?;
    let (sender, receiver) = channel();
    let stats_collector = spawn_stats_aggregator("Nodes to cache file", receiver);

    // Read PBF file using multiple threads, and in each thread store node positions into cache
    BlobReader::from_path(pbf_file)?.par_bridge().for_each_with(
        (cache, sender),
        |(dfc, sender), blob| {
            let mut cache = dfc.get_accessor();
            let mut stats = Stats::default();
            if let BlobDecode::OsmData(block) = blob.unwrap().decode().unwrap() {
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
                }
            };
            sender.send(stats).unwrap();
        },
    );

    stats_collector.join().unwrap();

    Ok(())
}
