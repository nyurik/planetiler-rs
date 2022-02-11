use std::ops::AddAssign;
use std::path::PathBuf;
use std::sync::mpsc::channel;

use anyhow::Error;
use clap::Parser;
use geos::{CoordSeq, GResult, Geom, Geometry};
use osmnodecache::{CacheStore, DenseFileCache};
use osmpbf::{BlobDecode, BlobReader};
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::utils::{spawn_stats_aggregator, timed};

#[derive(Debug, Parser)]
/// Extract ways from OSM PBF file using a node cache.
/// Assumes nodes stored before ways
pub struct Ways {
    /// Input pbf data.
    pbf_file: PathBuf,

    /// File for planet-size node cache.
    node_cache: PathBuf,
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

pub fn run(args: Ways) -> Result<(), Error> {
    timed("Ways parsed", || parse_ways(args))
}

pub fn parse_ways(args: Ways) -> Result<(), Error> {
    let cache = DenseFileCache::new(args.node_cache)?;
    let (sender, receiver) = channel();
    let stats_collector = spawn_stats_aggregator("Resolved ways", receiver);

    // Read PBF file using multiple threads, and in each thread it will
    // decode ways into arrays of points
    BlobReader::from_path(args.pbf_file)?
        .par_bridge()
        .for_each_with((cache, sender), |(dfc, sender), blob| {
            let cache = dfc.get_accessor();
            let mut stats = Stats::default();
            if let BlobDecode::OsmData(block) = blob.unwrap().decode().unwrap() {
                for group in block.groups() {
                    for way in group.ways() {
                        let refs: Vec<[f64; 2]> = way
                            .refs()
                            .map(|id| {
                                let (lat, lng) = cache.get_lat_lon(id as usize);
                                [lat as f64, lng as f64]
                            })
                            .collect();
                        match to_line(&refs) {
                            Ok(geom) => {
                                let env = geom.envelope().unwrap();
                                stats += Stats {
                                    count: 1,
                                    errors: 0,
                                    min_latitude: env.get_y_min().unwrap(),
                                    max_latitude: env.get_y_max().unwrap(),
                                    min_longitude: env.get_x_min().unwrap(),
                                    max_longitude: env.get_x_max().unwrap(),
                                }
                            }
                            Err(_) => {
                                stats.errors += 1;
                            }
                        };
                    }
                }
            };
            sender.send(stats).unwrap();
        });

    stats_collector.join().unwrap();

    Ok(())
}

fn to_line(refs: &[[f64; 2]]) -> GResult<Geometry> {
    Geometry::create_line_string(CoordSeq::new_from_vec(refs)?)
}


// struct OsmFeature

// fn extract_way(profile, cache, group) -> OsmFeature

// struct OsmFeatureReader;

// let mut osm = OsmFeatureReader::open(pbf_fn)?;
// osm.set_profile(profile);
// //osm.select_bbox(8.8, 47.2, 9.5, 55.3)?
// while let Some(feature) = osm.next()? {
//     let _layer = feature.layer()?;
//     let _props = feature.properties()?;
//     let _geometry = feature.geometry().unwrap();
// }
