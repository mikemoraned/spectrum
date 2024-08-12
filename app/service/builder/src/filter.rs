use std::collections::HashSet;

pub struct GreenTags<'a> {
    generic_tag_set: HashSet<(&'a str, &'a str)>,
}

impl<'a> Default for GreenTags<'a> {
    fn default() -> Self {
        let generic: Vec<(&str, &str)> = vec![
            ("leisure", "common"),
            ("leisure", "dog_park"),
            ("leisure", "golf_course"),
            ("leisure", "horse_riding"),
            ("leisure", "nature_reserve"),
            ("leisure", "park"),
            ("leisure", "pitch"),
            ("leisure", "wildlife_hide"),
            ("natural", "fell"),
            ("natural", "grassland"),
            ("natural", "heath"),
            ("natural", "moor"),
            ("natural", "scrub"),
            ("natural", "shrubbery"),
            ("natural", "tree"),
            ("natural", "tree_row"),
            ("natural", "tree_stump"),
            ("natural", "tundra"),
            ("natural", "wood"),
            ("amenity", "grave_yard"),
            ("landuse", "farmland"),
            ("landuse", "farmyard"),
            ("landuse", "forest"),
            ("landuse", "meadow"),
            ("landuse", "orchard"),
            ("landuse", "vineyard"),
            ("landuse", "cemetery"),
            ("landuse", "grass"),
            ("landuse", "recreation_ground"),
            ("landuse", "village_green"),
        ];
        let generic_tag_set: HashSet<(&str, &str)> = generic.into_iter().collect();
        Self { generic_tag_set }
    }
}

impl<'a> GreenTags<'a> {
    pub fn filter(&self, tag_set: HashSet<(&str, &str)>) -> bool {
        if tag_set.contains(&("leisure", "garden")) {
            tag_set.contains(&("access", "yes")) || tag_set.contains(&("garden:type", "community"))
        } else {
            tag_set.intersection(&self.generic_tag_set).count() > 0
        }
    }
}

pub fn green_tag_filter(tag_set: HashSet<(&str, &str)>) -> bool {
    let generic: Vec<(&str, &str)> = vec![
        ("leisure", "common"),
        ("leisure", "dog_park"),
        ("leisure", "golf_course"),
        ("leisure", "horse_riding"),
        ("leisure", "nature_reserve"),
        ("leisure", "park"),
        ("leisure", "pitch"),
        ("leisure", "wildlife_hide"),
        ("natural", "fell"),
        ("natural", "grassland"),
        ("natural", "heath"),
        ("natural", "moor"),
        ("natural", "scrub"),
        ("natural", "shrubbery"),
        ("natural", "tree"),
        ("natural", "tree_row"),
        ("natural", "tree_stump"),
        ("natural", "tundra"),
        ("natural", "wood"),
        ("amenity", "grave_yard"),
        ("landuse", "farmland"),
        ("landuse", "farmyard"),
        ("landuse", "forest"),
        ("landuse", "meadow"),
        ("landuse", "orchard"),
        ("landuse", "vineyard"),
        ("landuse", "cemetery"),
        ("landuse", "grass"),
        ("landuse", "recreation_ground"),
        ("landuse", "village_green"),
    ];
    let generic_tag_set: HashSet<(&str, &str)> = generic.into_iter().collect();
    if tag_set.contains(&("leisure", "garden")) {
        tag_set.contains(&("access", "yes")) || tag_set.contains(&("garden:type", "community"))
    } else {
        tag_set.intersection(&generic_tag_set).count() > 0
    }
}
