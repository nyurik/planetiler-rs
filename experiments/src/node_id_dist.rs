use std::fmt::{Debug, Formatter};
use std::ops;
use std::path::PathBuf;
use std::sync::mpsc::channel;

use crate::utils::{spawn_stats_aggregator, Histogram};
use anyhow::Error;
use clap::Parser;
use osmpbf::{BlobDecode, BlobReader};
use rayon::iter::{ParallelBridge, ParallelIterator};
use separator::Separatable;

#[derive(Debug, Parser)]
pub struct OptsNodeIdDistribution {
    /// Input pbf data.
    pbf_file: PathBuf,
}

struct Stats {
    pub ways: usize,
    pub node_counts: Histogram,
    pub node_distance: Histogram,
}

const LOG_BASE: f64 = 1.3;

impl Default for Stats {
    fn default() -> Self {
        Stats {
            ways: 0,
            node_counts: Histogram::new(
                |v| v.min(50),
                |v| {
                    if v < 50 {
                        v.to_string()
                    } else {
                        "50+".to_string()
                    }
                },
            ),
            node_distance: Histogram::new(
                |v| {
                    if v <= 1 {
                        0
                    } else {
                        (v as f64).log(LOG_BASE).round() as usize
                    }
                },
                |v| LOG_BASE.powf(v as f64).round().separated_string(),
            ),
        }
    }
}

impl Debug for Stats {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Total ways: {:}", self.ways.separated_string()).unwrap();
        writeln!(
            f,
            "{}",
            self.node_counts
                .to_string("Number of nodes in a way", false)
        )
        .unwrap();
        writeln!(
            f,
            "{}",
            self.node_distance.to_string(
                "Distance between min and max Node ID in a way feature, on a log scale",
                true
            )
        )
        .unwrap();
        Ok(())
    }
}
// impl Display for Stats{
// fn default() -> Self {
// }
// }

impl Stats {
    pub fn add_way(&mut self, min: i64, max: i64, len: i32) {
        self.node_counts.add(len as usize, 0);
        self.node_distance
            .add(if len < 1 { 0 } else { (max - min) as usize }, len as usize);
        self.ways += 1;
    }
}

impl ops::AddAssign for Stats {
    fn add_assign(&mut self, other: Self) {
        self.ways += other.ways;
        self.node_counts += other.node_counts;
        self.node_distance += other.node_distance;
    }
}

pub fn run(args: OptsNodeIdDistribution) -> Result<(), Error> {
    let (sender, receiver) = channel();
    let stats_collector = spawn_stats_aggregator("Node distribution", receiver);

    // For each way, find min & max node IDs used, and create a histogram of the int(log(max-min))
    BlobReader::from_path(args.pbf_file)?
        .par_bridge()
        .for_each_with(sender, |sender, blob| {
            let mut stats = Stats::default();
            if let BlobDecode::OsmData(block) = blob.unwrap().decode().unwrap() {
                for group in block.groups() {
                    for way in group.ways() {
                        let mut min_id = i64::MAX;
                        let mut max_id = i64::MIN;
                        let mut count = 0;
                        for id in way.refs() {
                            count += 1;
                            min_id = min_id.min(id);
                            max_id = max_id.max(id);
                        }
                        stats.add_way(min_id, max_id, count)
                    }
                }
            };
            sender.send(stats).unwrap();
        });

    stats_collector.join().unwrap();

    Ok(())
}
