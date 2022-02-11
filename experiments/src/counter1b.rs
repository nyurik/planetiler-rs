use crate::counter1_utils::Stats;
use crate::OptsCounter1;
use anyhow::Error;
use osmpbfreader::blobs::result_blob_into_iter;
use osmpbfreader::OsmPbfReader;
use par_map::ParMap;

pub fn run(args: OptsCounter1) -> Result<(), Error> {
    let stats = OsmPbfReader::new(std::fs::File::open(args.pbf_file).unwrap())
        .blobs()
        .par_map(|x| {
            let mut stats = Stats::default();
            for obj in result_blob_into_iter(x) {
                match obj.unwrap() {
                    osmpbfreader::OsmObj::Node(node) => {
                        stats.add_node(node.id.0, node.tags.into_inner().len());
                    }
                    osmpbfreader::OsmObj::Way(way) => {
                        stats.add_way(way.tags.into_inner().len());
                    }
                    osmpbfreader::OsmObj::Relation(rel) => {
                        stats.add_rel(rel.tags.into_inner().len());
                    }
                }
            }
            stats
        })
        .reduce(|a, i| a + i)
        .unwrap();
    println!("Single pass counting using osmpbfreader lib: {:#?}", stats);
    Ok(())
}
