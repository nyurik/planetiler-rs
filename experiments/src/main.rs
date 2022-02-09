use clap::Parser;

use crate::cache_nodes::CacheNodes;
use crate::chunked_resolver::ChunkedResolver;
use crate::utils::timed;
use counter1::Counter1;
use counter2::Counter2;
use node_id_dist::NodeIdDistribution;

mod cache_nodes;
mod chunked_resolver;
mod counter1;
mod counter2;
mod node_id_dist;
mod utils;

#[derive(Debug, Parser)]
#[clap(name = "experiments", about = "Run one of the performance test.")]
pub struct Opt {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Count1(Counter1),
    Count2(Counter2),
    CacheNodes(CacheNodes),
    NodeDist(NodeIdDistribution),
    Chunked(ChunkedResolver),
}

fn main() {
    let opt: Opt = Opt::parse();
    timed("Complete", || {
        let res = match opt.cmd {
            Command::Count1(arg) => counter1::run(arg),
            Command::Count2(arg) => counter2::run(arg),
            Command::NodeDist(arg) => node_id_dist::run(arg),
            Command::CacheNodes(arg) => cache_nodes::run(arg),
            Command::Chunked(arg) => chunked_resolver::run(arg),
        };

        if let Err(v) = res {
            println!("Error: {v}")
        }
    });
}
