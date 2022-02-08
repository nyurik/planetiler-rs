use std::ops::AddAssign;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Instant;

use anyhow::Error;
use clap::{ArgEnum, Parser};
use geos::{CoordSeq, GResult, Geom, Geometry};
use osmnodecache::{CacheStore, DenseFileCache, DenseFileCacheOpts};
use osmpbf::{BlobDecode, BlobReader};
use rayon::iter::{ParallelBridge, ParallelIterator};

#[derive(Debug, Parser)]
/// Resolve all ways to their geopoints via node cache, and calculate total bound box.
/// Assumes nodes stored before ways
pub struct Counter2 {
    /// What operations should be done with ways
    /// * Resolve - Resolve each node ID to lat/lng
    /// * Vector - Create a vector of lat/lng pairs
    /// * Geometry - Create a geometry from vector
    #[clap(arg_enum)]
    mode: Mode,

    /// Input pbf data.
    pbf_file: PathBuf,

    /// File for planet-size node cache.
    node_cache: PathBuf,
}

#[derive(ArgEnum, Debug, Clone, Copy)]
enum Mode {
    Resolve,
    Vector,
    Geometry,
}


#[derive(Clone, Default, Debug)]
struct Stats {
    pub count: usize,
    pub errors: usize,
    pub min_latitude: f64,
    pub max_latitude: f64,
    pub min_longitude: f64,
    pub max_longitude: f64,
}

impl AddAssign for Stats {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            count: self.count + other.count,
            errors: self.errors + other.errors,
            min_latitude: self.min_latitude.min(other.min_latitude),
            max_latitude: self.max_latitude.max(other.max_latitude),
            min_longitude: self.min_longitude.min(other.min_longitude),
            max_longitude: self.max_longitude.max(other.max_longitude),
        };
    }
}

impl AddAssign<(f64, f64)> for Stats {
    fn add_assign(&mut self, (other_lat, other_lng): (f64, f64)) {
        *self = Self {
            count: self.count + 1,
            errors: self.errors,
            min_latitude: self.min_latitude.min(other_lat),
            max_latitude: self.max_latitude.max(other_lat),
            min_longitude: self.min_longitude.min(other_lng),
            max_longitude: self.max_longitude.max(other_lng),
        };
    }
}

pub fn run(args: Counter2) -> Result<(), Error> {
    let start = Instant::now();
    let res = parse_nodes(&args);
    println!(
        "Nodes parsed in {:.1} seconds",
        start.elapsed().as_secs_f32()
    );
    res?;

    let start = Instant::now();
    let res = parse_ways(args);
    println!(
        "Ways parsed in {:.1} seconds",
        start.elapsed().as_secs_f32()
    );
    res
}

pub fn parse_nodes(args: &Counter2) -> Result<(), Error> {
    let cache = DenseFileCacheOpts::new(args.node_cache.clone())
        .page_size(10 * 1024 * 1024 * 1024)
        .open()?;

    let (sender, receiver) = channel();

    // This thread will wait for all stats objects and sum them up
    let stats_collector = thread::spawn(move || {
        let mut stats = Stats::default();
        while let Ok(v) = receiver.recv() {
            stats += v;
        }
        println!("Node parsing results: {:#?}", stats);
    });

    // Read PBF file using multiple threads, and in each thread store node positions into cache
    BlobReader::from_path(args.pbf_file.clone())?
        .par_bridge()
        .for_each_with((cache, sender), |(dfc, sender), blob| {
            let mut cache = dfc.get_accessor();
            let mut stats = Stats::default();
            if let BlobDecode::OsmData(block) = blob.unwrap().decode().unwrap() {
                for group in block.groups() {
                    for node in group.nodes() {
                        let lat = node.lat();
                        let lon = node.lon();
                        cache.set_lat_lon(node.id() as usize, lat, lon);
                        stats += (lat, lon);
                    }
                    for node in group.dense_nodes() {
                        let lat = node.lat();
                        let lon = node.lon();
                        cache.set_lat_lon(node.id() as usize, lat, lon);
                        stats += (lat, lon);
                    }
                }
            };
            sender.send(stats).unwrap();
        });

    stats_collector.join().unwrap();

    Ok(())
}

pub fn parse_ways(args: Counter2) -> Result<(), Error> {
    let cache = DenseFileCache::new(args.node_cache)?;

    let (sender, receiver) = channel();

    // This thread will wait for all stats objects and sum them up
    let stats_collector = thread::spawn(move || {
        let mut stats = Stats::default();
        while let Ok(v) = receiver.recv() {
            stats += v;
        }
        println!("Ways parsing results: {:#?}", stats);
    });

    // Read PBF file using multiple threads, and in each thread it will
    // decode ways into arrays of points
    BlobReader::from_path(args.pbf_file)?
        .par_bridge()
        .for_each_with((cache, sender), |(dfc, sender), blob| {
            let cache = dfc.get_accessor();
            let mode = args.mode;
            let mut stats = Stats::default();
            if let BlobDecode::OsmData(block) = blob.unwrap().decode().unwrap() {
                for group in block.groups() {
                    for way in group.ways() {
                        if let Mode::Resolve = mode {
                            for id in way.refs() {
                                let (lat, lng) = cache.get_lat_lon(id as usize);
                                stats += (lat as f64, lng as f64)
                            }
                            continue;
                        }
                        let refs: Vec<[f64; 2]> = way
                            .refs()
                            .map(|id| {
                                let (lat, lng) = cache.get_lat_lon(id as usize);
                                [lat as f64, lng as f64]
                            })
                            .collect();
                        if let Mode::Vector = mode {
                            for [lat, lng] in refs {
                                stats += (lat as f64, lng as f64)
                            }
                            continue;
                        }
                        match get_bbox(&refs) {
                            Ok((min_lat, max_lat, min_lng, max_lng)) => {
                                stats += Stats {
                                    count: 1,
                                    errors: 0,
                                    min_latitude: min_lat,
                                    max_latitude: max_lat,
                                    min_longitude: min_lng,
                                    max_longitude: max_lng,
                                }
                            }
                            Err(_) => {
                                stats.errors += 1;
                            }
                        }
                    }
                }
            };
            sender.send(stats).unwrap();
        });

    stats_collector.join().unwrap();

    Ok(())
}

fn get_bbox(refs: &[[f64; 2]]) -> GResult<(f64, f64, f64, f64)> {
    let geometry = Geometry::create_line_string(CoordSeq::new_from_vec(refs)?)?;
    let geom = geometry.envelope()?;
    Ok((
        geom.get_y_min()?,
        geom.get_y_max()?,
        geom.get_x_min()?,
        geom.get_x_max()?,
    ))
}
