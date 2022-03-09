extern crate core;

use crate::cache_nodes2::OptsCacheNodes2;
use clap::Parser;
mod geostruct;
use crate::cache_nodes::OptsCacheNodes;
use crate::chunked_resolver::OptsChunkedResolver;
use crate::counter1_utils::OptsCounter1;
use crate::counter2::OptsCounter2;
use crate::node_id_dist::OptsNodeIdDistribution;
use crate::track_tiles::OptsTrackTiles;
use crate::utils::timed;

mod cache_nodes;
mod cache_nodes2;
mod cache_nodes3;
mod chunked_resolver;
mod counter1_utils;
mod counter1a;
mod counter1b;
mod counter2;
mod node_id_dist;
mod tile_id;
mod track_tiles;
mod utils;

#[derive(Debug, Parser)]
#[clap(name = "experiments", about = "Run one of the performance test.")]
pub struct Opt {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Parser)]
enum Command {
    /// Iterate over an OSM PBF file. Count features and tags. Use osmpbf lib.
    Count1a(OptsCounter1),
    /// Iterate over an OSM PBF file. Count features and tags. Use osmpbfreader lib.
    Count1b(OptsCounter1),
    /// Resolve all ways to their geopoints via node cache, and calculate total bound box.
    /// Assumes nodes are stored before ways.
    Count2(OptsCounter2),
    /// Create a node cache using memory map.
    CacheNodes(OptsCacheNodes),
    /// Create a node cache using sequential writer.
    CacheNodes2(OptsCacheNodes2),
    /// Create a node cache opening files for each block in parallel.
    CacheNodes3(OptsCacheNodes2),
    /// Iterate over an OSM PBF file and count the number of features and tags
    NodeDist(OptsNodeIdDistribution),
    /// Resolve all ways to their geopoints via node cache, and calculate total bound box.
    /// Assumes nodes are stored before ways.
    Chunked(OptsChunkedResolver),
    /// Create a disk map with (feature ID -> list of tile IDs). Evaluate how to track which feature exists in which tiles.
    Track(OptsTrackTiles),
}

fn main() {
    let opt: Opt = Opt::parse();
    timed("Complete", || {
        let res = match opt.cmd {
            Command::Count1a(arg) => counter1a::run(arg),
            Command::Count1b(arg) => counter1b::run(arg),
            Command::Count2(arg) => counter2::run(arg),
            Command::NodeDist(arg) => node_id_dist::run(arg),
            Command::CacheNodes(arg) => cache_nodes::run(arg),
            Command::CacheNodes2(arg) => cache_nodes2::run(arg),
            Command::CacheNodes3(arg) => cache_nodes3::run(arg),
            Command::Chunked(arg) => chunked_resolver::run(arg),
            Command::Track(arg) => track_tiles::run(arg),
        };

        if let Err(v) = res {
            println!("Error: {v}")
        }
    });
}
