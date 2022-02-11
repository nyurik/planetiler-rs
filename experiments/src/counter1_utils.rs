use clap::Parser;
use std::ops;
use std::path::PathBuf;

/// Iterate over an OSM PBF file and count the number of features and tags
#[derive(Debug, Parser)]
pub struct OptsCounter1 {
    /// Input pbf data.
    pub pbf_file: PathBuf,
}

//noinspection DuplicatedCode
#[derive(Clone, Default, Debug)]
pub struct Stats {
    pub node_max_id: i64,
    pub nodes: usize,
    pub empty_nodes: usize,
    pub node_tags: usize,
    pub ways: usize,
    pub way_tags: usize,
    pub rels: usize,
    pub rel_tags: usize,
}

impl Stats {
    #[inline]
    pub fn add_node(&mut self, node_id: i64, tag_count: usize) {
        if tag_count > 0 {
            self.nodes += 1;
            self.node_tags += tag_count;
        } else {
            self.empty_nodes += 1;
        }
        if node_id > self.node_max_id {
            self.node_max_id = node_id
        }
    }

    #[inline]
    pub fn add_way(&mut self, tag_count: usize) {
        self.ways += 1;
        self.way_tags += tag_count;
    }

    #[inline]
    pub fn add_rel(&mut self, tag_count: usize) {
        self.rels += 1;
        self.rel_tags += tag_count;
    }
}

impl ops::Add for Stats {
    type Output = Stats;

    fn add(self, other: Self) -> Self::Output {
        Self {
            node_max_id: self.node_max_id.max(other.node_max_id),
            nodes: self.nodes + other.nodes,
            empty_nodes: self.empty_nodes + other.empty_nodes,
            node_tags: self.node_tags + other.node_tags,
            ways: self.ways + other.ways,
            way_tags: self.way_tags + other.way_tags,
            rels: self.rels + other.rels,
            rel_tags: self.rel_tags + other.rel_tags,
        }
    }
}
