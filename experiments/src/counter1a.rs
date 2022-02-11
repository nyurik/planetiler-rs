use crate::counter1_utils::{OptsCounter1, Stats};
use anyhow::Error;
use osmpbf::{BlobDecode, BlobReader};
use rayon::iter::{ParallelBridge, ParallelIterator};

pub fn run(args: OptsCounter1) -> Result<(), Error> {
    // Read PBF file using multiple threads, and in each thread it will
    // decode blocks, count stats, and aggregate stats.
    let stats = BlobReader::from_path(args.pbf_file)?
        .par_bridge()
        .map(|blob| {
            let mut stats = Stats::default();
            if let BlobDecode::OsmData(block) = blob.unwrap().decode().unwrap() {
                for group in block.groups() {
                    for node in group.nodes() {
                        stats.add_node(node.id(), node.tags().count());
                    }
                    for node in group.dense_nodes() {
                        stats.add_node(node.id(), node.tags().count());
                    }
                    for way in group.ways() {
                        stats.add_way(way.tags().count());
                    }
                    for rel in group.relations() {
                        stats.add_rel(rel.tags().count());
                    }
                }
            };
            stats
        })
        .reduce(Stats::default, |a, b| a + b);
    println!("Single pass counting using osmpbf lib: {:#?}", stats);
    Ok(())
}
