use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::{ops, thread};

use anyhow::Error;
use clap::Parser;
use osmpbf::{BlobDecode, BlobReader};
use rayon::iter::{ParallelBridge, ParallelIterator};

#[derive(Debug, Parser)]
/// Iterate over an OSM PBF file and count the number of features and tags
pub struct Counter1 {
    /// Input pbf data.
    pbf_file: PathBuf,
}

#[derive(Clone, Default, Debug)]
struct Stats {
    pub node_max_id: i64,
    pub nodes: usize,
    pub node_tags: usize,
    pub dense_nodes: usize,
    pub dense_node_tags: usize,
    pub ways: usize,
    pub way_tags: usize,
    pub rels: usize,
    pub rel_tags: usize,
}

impl ops::AddAssign for Stats {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            nodes: self.nodes + other.nodes,
            node_tags: self.node_tags + other.node_tags,
            node_max_id: self.node_max_id.max(other.node_max_id),
            dense_nodes: self.dense_nodes + other.dense_nodes,
            dense_node_tags: self.dense_node_tags + other.dense_node_tags,
            ways: self.ways + other.ways,
            way_tags: self.way_tags + other.way_tags,
            rels: self.rels + other.rels,
            rel_tags: self.rel_tags + other.rel_tags,
        };
    }
}

//noinspection DuplicatedCode
pub fn run(args: Counter1) -> Result<(), Error> {
    let (sender, receiver) = channel();

    // This thread will wait for all stats objects and sum them up
    let stats_collector = thread::spawn(move || {
        let mut stats = Stats::default();
        while let Ok(v) = receiver.recv() {
            stats += v;
        }
        println!("{:#?}", stats);
    });

    // Read PBF file using multiple threads, and in each thread it will
    // decode blocks, count stats, and send block stats to the channel
    BlobReader::from_path(args.pbf_file)?
        .par_bridge()
        .for_each_with(sender, |sender, blob| {
            let mut stats = Stats::default();
            if let BlobDecode::OsmData(block) = blob.unwrap().decode().unwrap() {
                for group in block.groups() {
                    for node in group.nodes() {
                        stats.nodes += 1;
                        stats.node_tags += node.tags().count();
                        stats.node_max_id = stats.node_max_id.max(node.id())
                    }
                    for node in group.dense_nodes() {
                        stats.dense_nodes += 1;
                        stats.dense_node_tags += node.tags().count();
                        stats.node_max_id = stats.node_max_id.max(node.id())
                    }
                    for way in group.ways() {
                        stats.ways += 1;
                        stats.way_tags += way.tags().count();
                    }
                    for rel in group.relations() {
                        stats.rels += 1;
                        stats.rel_tags += rel.tags().count();
                    }
                }
            };
            sender.send(stats).unwrap();
        });

    stats_collector.join().unwrap();

    Ok(())
}
