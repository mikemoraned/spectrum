use osmpbf::{Element, ElementReader};

fn main() -> Result<(), ()> {
    let reader = ElementReader::from_path("data/edinburgh_scotland.osm.pbf").unwrap();

    reader
        .for_each(|element| {
            if let Element::Way(way) = element {
                if way
                    .tags()
                    .any(|(name, value)| name == "name" && value == "The Meadows")
                {
                    println!("Found meadows: {}", way.id());
                }
            }
        })
        .unwrap();

    Ok(())
}
