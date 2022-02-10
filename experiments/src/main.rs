extern crate core;

use clap::Parser;

use crate::cache_nodes::OptsCacheNodes;
use crate::chunked_resolver::OptsChunkedResolver;
use crate::utils::timed;
use counter1_utils::OptsCounter1;
use counter2::OptsCounter2;
use node_id_dist::OptsNodeIdDistribution;
use ways::Ways;

mod cache_nodes;
mod chunked_resolver;
mod counter1_utils;
mod counter1a;
mod counter1b;
mod counter2;
mod node_id_dist;
mod utils;
mod ways;

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
    Ways(Ways),
    /// Create a node cache
    CacheNodes(OptsCacheNodes),
    /// Iterate over an OSM PBF file and count the number of features and tags
    NodeDist(OptsNodeIdDistribution),
    /// Resolve all ways to their geopoints via node cache, and calculate total bound box.
    /// Assumes nodes are stored before ways.
    Chunked(OptsChunkedResolver),
}

fn main() {
    let opt: Opt = Opt::parse();
    timed("Complete", || {
        let res = match opt.cmd {
            Command::Count1a(arg) => counter1a::run(arg),
            Command::Count1b(arg) => counter1b::run(arg),
            Command::Count2(arg) => counter2::run(arg),
            Command::Ways(arg) => ways::run(arg),
            Command::NodeDist(arg) => node_id_dist::run(arg),
            Command::CacheNodes(arg) => cache_nodes::run(arg),
            Command::Chunked(arg) => chunked_resolver::run(arg),
        };

        if let Err(v) = res {
            println!("Error: {v}")
        }
    });
}
