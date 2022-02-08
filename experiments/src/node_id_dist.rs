use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::{ops, thread};

use anyhow::Error;
use clap::Parser;
use osmpbf::{BlobDecode, BlobReader};
use rayon::iter::{ParallelBridge, ParallelIterator};
use separator::Separatable;
use crate::utils::Histogram;

#[derive(Debug, Parser)]
/// Iterate over an OSM PBF file and count the number of features and tags
pub struct NodeIdDistribution {
    /// Input pbf data.
    pbf_file: PathBuf,
}

#[derive(Debug)]
struct Stats {
    pub ways: usize,
    pub node_counts: Histogram,
    pub node_distance: Histogram,
}

const LOG_BASE: f64 = 1.3;

impl Stats {
    pub fn new() -> Self {
        Stats {
            ways: 0,
            node_counts: Histogram::new(
                |v| v.min(50),
                |v| if v < 50 { v.to_string() } else { "50+".to_string() }),
            node_distance: Histogram::new(
                |v| if v <= 1 { 0 } else { (v as f64).log(LOG_BASE).round() as usize },
                |v| LOG_BASE.powf(v as f64).round().separated_string()),
        }
    }

    pub fn add_way(&mut self, min: i64, max: i64, len: i32) {
        self.node_counts.add(len as usize);
        self.node_distance.add(if len < 1 { 0 } else { (max - min) as usize });
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

pub fn run(args: NodeIdDistribution) -> Result<(), Error> {
    let (sender, receiver) = channel();

    // This thread will wait for all stats objects and sum them up
    let stats_collector = thread::spawn(move || {
        let mut stats = Stats::new();
        while let Ok(v) = receiver.recv() {
            stats += v;
        }
        println!("Total ways: {:}", stats.ways);
        stats.node_counts.print("Number of nodes in a way");
        stats.node_distance.print("Distance between min and max Node ID in a way feature, on a log scale");
    });

    // For each way, find min & max node IDs used, and create a histogram of the int(log(max-min))
    BlobReader::from_path(args.pbf_file)?
        .par_bridge()
        .for_each_with(sender, |sender, blob| {
            let mut stats = Stats::new();
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
