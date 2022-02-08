use std::time::Instant;

use structopt::StructOpt;

use counter1::Counter1;
use counter2::Counter2;

mod counter1;
mod counter2;

#[derive(Debug, StructOpt)]
#[structopt(name = "experiments", about = "Run one of the performance test.")]
pub struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    Count1(Counter1),
    Count2(Counter2),
}

fn main() {
    let opt: Opt = Opt::from_args();
    let start = Instant::now();

    let res = match opt.cmd {
        Command::Count1(arg) => counter1::run(arg),
        Command::Count2(arg) => counter2::run(arg),
    };

    if let Err(v) = res {
        println!("Error: {v}")
    }

    println!("Complete in {:.1} seconds", start.elapsed().as_secs_f32());
}
