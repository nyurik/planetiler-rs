use std::ops::AddAssign;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::mpsc::channel;

use crate::timed;
use anyhow::Error;
use clap::{ArgEnum, Parser};
use osmnodecache::{CacheStore, DenseFileCache};
use osmpbf::{BlobDecode, BlobReader};
use rayon::iter::{ParallelBridge, ParallelIterator};
use separator::Separatable;

// use geos::{CoordSeq, GResult, Geom, Geometry};

use crate::utils::spawn_stats_aggregator;

#[derive(Debug, Parser)]
pub struct OptsChunkedResolver {
    #[clap(arg_enum)]
    /// Skip - ignore ways whose IDs didn't fit into slice
    mode: Mode,

    /// Input pbf data.
    pbf_file: PathBuf,

    /// File for planet-size node cache.
    node_cache: PathBuf,

    /// Size of memory to store nodes in one run, in GB.
    /// Example: value 1 will process node IDs in the range 0..1*1024*1024*1024/8-1 in the first iteration,
    /// followed by 1*1024*1024*1024/8..2*1024*1024*1024/8-1, etc.
    mem_slice: usize,
}

#[derive(ArgEnum, Debug, Clone, Copy)]
enum Mode {
    Skip,
}

#[derive(Clone, Default, Debug)]
struct Stats {
    pub ways_viewed: usize,
    pub ways_resolved: usize,
    pub nodes_resolved: usize,
    pub errors: usize,
    pub skipped: usize,
    pub min_latitude: f64,
    pub max_latitude: f64,
    pub min_longitude: f64,
    pub max_longitude: f64,
}

impl Stats {
    fn add_point(&mut self, lat: f64, lng: f64) {
        self.nodes_resolved += 1;
        self.min_latitude = self.min_latitude.min(lat);
        self.max_latitude = self.max_latitude.max(lat);
        self.min_longitude = self.min_longitude.min(lng);
        self.max_longitude = self.max_longitude.max(lng);
    }
}

impl AddAssign for Stats {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            ways_viewed: self.ways_viewed + other.ways_viewed,
            ways_resolved: self.ways_resolved + other.ways_resolved,
            nodes_resolved: self.nodes_resolved + other.nodes_resolved,
            errors: self.errors + other.errors,
            skipped: self.skipped + other.skipped,
            min_latitude: self.min_latitude + other.min_latitude,
            max_latitude: self.max_latitude + other.max_latitude,
            min_longitude: self.min_longitude + other.min_longitude,
            max_longitude: self.max_longitude + other.max_longitude,
        };
    }
}

pub fn run(args: OptsChunkedResolver) -> Result<(), Error> {
    let cache = DenseFileCache::new(args.node_cache)?;
    let mut start_idx = 0;
    let chunk_size = (args.mem_slice * 1024 * 1024 * 1024 / 8) as i64;
    let max_node_id = AtomicI64::new(0);

    while start_idx <= max_node_id.load(Ordering::Relaxed) {
        timed(
            format!(
                "Iteration for node IDs {}..{}",
                start_idx.separated_string(),
                (start_idx + chunk_size).separated_string()
            )
            .as_str(),
            || run_one_pass(&cache, &args.pbf_file, &max_node_id, start_idx, chunk_size),
        )?;
        start_idx += chunk_size;
    }

    Ok(())
}

fn run_one_pass(
    cache: &DenseFileCache,
    pbf_file: &Path,
    shared_max_node_id: &AtomicI64,
    start_idx: i64,
    chunk_size: i64,
) -> Result<(), Error> {
    let (sender, receiver) = channel();
    let stats_collector = spawn_stats_aggregator("Chunked parser", receiver);
    BlobReader::from_path(pbf_file)?.par_bridge().for_each_with(
        (cache, sender),
        |(dfc, sender), blob| {
            if let BlobDecode::OsmData(block) = blob.unwrap().decode().unwrap() {
                let cache = dfc.get_accessor();
                let mut stats = Stats::default();
                let mut max_node_id = 0;
                let last_idx = start_idx + chunk_size;
                for group in block.groups() {
                    'way: for way in group.ways() {
                        for id in way.refs() {
                            if id > max_node_id {
                                max_node_id = id;
                            }
                            if id < start_idx || id >= last_idx {
                                stats.ways_viewed += 1;
                                continue 'way;
                            }
                        }
                        for id in way.refs() {
                            let (lat, lng) = cache.get_lat_lon(id as usize);
                            stats.add_point(lat, lng)
                        }
                        stats.ways_resolved += 1;

                        // if let Mode::Resolve = mode {
                        //     for id in way.refs() {
                        //         let (lat, lng) = cache.get_lat_lon(id as usize);
                        //         stats += (lat as f64, lng as f64)
                        //     }
                        //     continue;
                        // }
                        // let refs: Vec<[f64; 2]> = way
                        //     .refs()
                        //     .map(|id| {
                        //         let (lat, lng) = cache.get_lat_lon(id as usize);
                        //         [lat as f64, lng as f64]
                        //     })
                        //     .collect();
                        // if let Mode::Vector = mode {
                        //     for [lat, lng] in refs {
                        //         stats += (lat as f64, lng as f64)
                        //     }
                        //     continue;
                        // }
                        // match get_bbox(&refs) {
                        //     Ok((min_lat, max_lat, min_lng, max_lng)) => {
                        //         stats += Stats {
                        //             count: 1,
                        //             errors: 0,
                        //             min_latitude: min_lat,
                        //             max_latitude: max_lat,
                        //             min_longitude: min_lng,
                        //             max_longitude: max_lng,
                        //         }
                        //     }
                        //     Err(_) => {
                        //         stats.errors += 1;
                        //     }
                        // }
                    }
                }
                shared_max_node_id.fetch_max(max_node_id, Ordering::Relaxed);
                sender.send(stats).unwrap();
            };
        },
    );
    stats_collector.join().unwrap();
    Ok(())
}

// fn get_bbox(refs: &[[f64; 2]]) -> GResult<(f64, f64, f64, f64)> {
//     let geometry = Geometry::create_line_string(CoordSeq::new_from_vec(refs)?)?;
//     let geom = geometry.envelope()?;
//     Ok((
//         geom.get_y_min()?,
//         geom.get_y_max()?,
//         geom.get_x_min()?,
//         geom.get_x_max()?,
//     ))
// }
