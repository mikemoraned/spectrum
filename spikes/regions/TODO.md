- (/) show map of edinburgh in webapp
- (/) load and show a GeoJSON set of features on the Map
- (/) load GeoJSON dynamically from a service
- (/) define those features in the service by loading from an openstreetmap file and showing all polygons of interest
  - (/) show meadows
    - (/) find meadows in the `edinburgh_scotland.osm.pbf` file
    - (/) convert nodes into geojson and save to file (for separate validation)
    - (/) fix bug: re-order positions based on order in way
    - (/) integrate into service and load into webapp
  - (/) find all ways tagged `leisure=park`
  - (/) integrate into service and load into webapp
- (/) filter polygons to those of a particular label or type
