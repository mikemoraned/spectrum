use osmpbf::{Element, ElementReader};

fn main() -> Result<(), ()> {
    let reader = ElementReader::from_path("data/edinburgh_scotland.osm.pbf").unwrap();
    let mut ways = 0_u64;

    // Increment the counter by one for each way.
    reader
        .for_each(|element| {
            if let Element::Way(_) = element {
                ways += 1;
            }
        })
        .unwrap();

    println!("Number of ways: {ways}");

    Ok(())
}
