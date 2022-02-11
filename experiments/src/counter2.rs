use std::ops::AddAssign;
use std::path::PathBuf;
use std::sync::mpsc::channel;

use anyhow::Error;
use clap::{ArgEnum, Parser};
use geos::{CoordSeq, GResult, Geom, Geometry};
use osmnodecache::{CacheStore, DenseFileCache};
use osmpbf::{BlobDecode, BlobReader, ByteOffset};
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::cache_nodes::parse_nodes;
use crate::utils::MemAdvice::{Random, Sequential};
use crate::utils::{advise_cache, spawn_stats_aggregator, timed, OptAdvice};

#[derive(Debug, Parser)]
pub struct OptsCounter2 {
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

    #[clap(flatten)]
    advice: OptAdvice,
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

impl Stats {
    fn add_point(&mut self, lat: f64, lng: f64) {
        self.count += 1;
        self.min_latitude = self.min_latitude.min(lat);
        self.max_latitude = self.max_latitude.max(lat);
        self.min_longitude = self.min_longitude.min(lng);
        self.max_longitude = self.max_longitude.max(lng);
    }
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

pub fn run(args: OptsCounter2) -> Result<(), Error> {
    let (advice1, advice2) = if args.advice.advice.is_empty() {
        // By default, use sequential memmap creation, but random during node resolution
        (
            OptAdvice {
                advice: vec![Sequential],
            },
            OptAdvice {
                advice: vec![Random],
            },
        )
    } else {
        (args.advice.clone(), args.advice.clone())
    };
    let first_way_block_offset = timed("Node cache created", || {
        parse_nodes(&args.pbf_file, args.node_cache.clone(), &advice1)
    })?;

    timed("Ways parsed", || {
        parse_ways(args, &advice2, first_way_block_offset)
    })
}

pub fn parse_ways(
    args: OptsCounter2,
    advice: &OptAdvice,
    starting_offset: u64,
) -> Result<(), Error> {
    let cache = DenseFileCache::new(args.node_cache.clone())?;
    advise_cache(&cache, advice)?;

    let (sender, receiver) = channel();
    let stats_collector = spawn_stats_aggregator("Resolved ways", receiver);
    let mut reader = BlobReader::from_path(args.pbf_file)?;

    if starting_offset > 0 {
        println!("Skipping to offset {starting_offset}");
        reader.seek(ByteOffset(starting_offset))?;
    }

    // Read PBF file using multiple threads, and in each thread it will
    // decode ways into arrays of points
    reader
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
                                stats.add_point(lat as f64, lng as f64)
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
                                stats.add_point(lat as f64, lng as f64)
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
