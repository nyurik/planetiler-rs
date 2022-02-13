use std::ops::AddAssign;
use std::path::PathBuf;
use std::sync::mpsc::channel;

use anyhow::Error;
use clap::Parser;
use geos::Geom;
use geozero::geos::GeosWriter;
use osmnodecache::{Cache, CacheStore, DenseFileCache};
use osmpbf::{BlobDecode, BlobReader, ByteOffset};
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::utils::MemAdvice::{Random, Sequential};
use crate::utils::{advise_cache, spawn_stats_aggregator, timed, OptAdvice};

#[derive(Debug, Parser)]
/// Extract ways from OSM PBF file using a node cache.
/// Assumes nodes stored before ways
pub struct WayStats {
    /// Input pbf data.
    pbf_file: PathBuf,

    /// File for planet-size node cache.
    node_cache: PathBuf,

    #[clap(flatten)]
    advice: OptAdvice,
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

pub fn run(args: WayStats) -> Result<(), Error> {
    let (_advice1, advice2) = if args.advice.advice.is_empty() {
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
    let first_way_block_offset = 0;

    timed("Ways parsed", || {
        parse_ways(args, &advice2, first_way_block_offset)
    })
}

pub fn parse_ways(args: WayStats, advice: &OptAdvice, starting_offset: u64) -> Result<(), Error> {
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
            let mut stats_processor = StatsProcessor::new();
            if let BlobDecode::OsmData(block) = blob.unwrap().decode().unwrap() {
                for group in block.groups() {
                    for way in group.ways() {
                        stats_processor.process_way(&cache, way);
                    }
                }
            };
            sender.send(stats_processor.stats).unwrap();
        });

    stats_collector.join().unwrap();

    Ok(())
}

struct StatsProcessor<'a> {
    geom_writer: GeosWriter<'a>,
    stats: Stats,
}

impl<'a> StatsProcessor<'a> {
    fn new() -> Self {
        StatsProcessor {
            geom_writer: GeosWriter::new(),
            stats: Stats::default(),
        }
    }
}

impl<'a> StatsProcessor<'a> {
    fn process_way(&mut self, cache: &'a Box<dyn Cache + 'a>, way: osmpbf::Way) {
        OsmGeomProcessor::process_way(cache, way, &mut self.geom_writer);
        let geom = self.geom_writer.geometry();
        let env = geom.envelope().unwrap();
        self.stats += Stats {
            count: 1,
            errors: 0,
            min_latitude: env.get_y_min().unwrap(),
            max_latitude: env.get_y_max().unwrap(),
            min_longitude: env.get_x_min().unwrap(),
            max_longitude: env.get_x_max().unwrap(),
        }
    }
}

struct OsmGeomProcessor;

impl OsmGeomProcessor {
    fn process_way<'a>(
        cache: &'a Box<dyn Cache + 'a>,
        way: osmpbf::Way,
        processor: &mut dyn geozero::GeomProcessor,
    ) {
        let tagged = true;
        let line_idx = 0;
        processor
            .linestring_begin(tagged, way.refs().len(), line_idx)
            .unwrap();
        for (idx, node_id) in way.refs().enumerate() {
            let (lat, lng) = cache.get_lat_lon(node_id as usize);
            processor.xy(lat, lng, idx).unwrap();
        }
        processor.linestring_end(tagged, line_idx).unwrap();
    }
}

/*

## Use case 1: planetiler passes for OpenMaptiles MVT creation

1. osm reader pass 1:
   - read nodes into node cache
   - read relations into cache using profile

2. osm reader pass 2:
   - nodes: emit a point source feature
   - ways:
     - emit line or polygon feature
     - cache multipolygon parts
   - relations: emit multipolygon feature

3. prepare each feature for MVT (1 worker thread per core)
   - scale to zoom level
   - simplifiy in screen pixel coordinates
   - slice geometries on tile borders
   - fix topology erros
   - Encode into compact binary format

4. Write tile features to disk (single-threaded worker)

5. Sort features

6. Emit vector tiles


## Use case 2: Geo feature extraction point/line/polygon/multipolygon

1. osm reader pass 1:
  - if node cache doesn't exist:
      read nodes into node cache using profile filter
  - read relations into cache using profile filter

2. osm reader pass 2

3. Optional: reproject to destination projection


## Use case 3: Complex Geo feature extraction e.g. streets

1. osm reader pass 1
2. osm reader pass 2
3. Postprocess features
   - Build streets from ways


## Use case 4: POI extraction

1. osm reader pass 1
2. osm reader pass 2
3. Postprocess features
   - Emit POIs


## Use case 5: Routing network extraction e.g. railway tracks

1. osm reader pass 1
2. osm reader pass 2
3. Postprocess features
   - Build network from ways

More use cases:
- Statistics
- Building extraction for 3D rendering

*/
