use std::time::Instant;

use clap::Parser;

use counter1::Counter1;
use counter2::Counter2;
use node_id_dist::NodeIdDistribution;

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
    NodeDist(NodeIdDistribution),
}

fn main() {
    let opt: Opt = Opt::parse();
    let start = Instant::now();

    let res = match opt.cmd {
        Command::Count1(arg) => counter1::run(arg),
        Command::Count2(arg) => counter2::run(arg),
        Command::NodeDist(arg) => node_id_dist::run(arg),
    };

    if let Err(v) = res {
        println!("Error: {v}")
    }

    println!("Complete in {:.1} seconds", start.elapsed().as_secs_f32());
}
