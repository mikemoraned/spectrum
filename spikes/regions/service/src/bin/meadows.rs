use osmpbf::{Element, IndexedReader};

fn main() -> Result<(), ()> {
    let mut reader = IndexedReader::from_path("data/edinburgh_scotland.osm.pbf").unwrap();

    reader
        .read_ways_and_deps(
            |way| {
                way.tags()
                    .any(|key_value| key_value == ("name", "The Meadows"))
            },
            |element| {
                // Increment counter
                match element {
                    Element::Way(way) => {
                        println!("way: {:?}", way.id());
                    }
                    Element::Node(node) => {
                        println!("node: lat: {}, lon: {}", node.lat(), node.lon());
                    }
                    Element::DenseNode(dense_node) => {
                        println!(
                            "dense_node: lat: {}, lon: {}",
                            dense_node.lat(),
                            dense_node.lon()
                        );
                    }
                    _ => (),
                }
            },
        )
        .unwrap();

    Ok(())
}
