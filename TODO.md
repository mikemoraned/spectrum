```mermaid
flowchart TB
    subgraph webapp
        subgraph stack
        direction TB
        layer1-->layer2-->layer3
        end
        basemap
        basemap <--> layer1
        basemap <--> layer2
        basemap <--> layer3
    end
    subgraph service
    direction LR
    endpoint1
    endpoint2
    end
    subgraph data
    flatgeobuf
    end
    subgraph build
    openstreetmap-->regions
    routes_provider-->routes
    routes-->overlapped
    regions-->overlapped
    end

    layer1-..->endpoint1
    layer2-..->endpoint1
    layer3-..->endpoint2

    overlapped-..->flatgeobuf

    endpoint1-..->flatgeobuf
    endpoint2-..->flatgeobuf
```

- (/) v0.1: show map bounding box as regions

  - (/) webapp

    - (/) get example sveltekit app working on netlify
      - (/) example sveltekit app working locally
      - (/) deployed on netlify; https://deploy-preview-8--spectrum-green.netlify.app
    - (/) show map, initially focussed on edinburgh
      - (/) switch to default setup for sveltekit
      - (/) add basic mapbox setup
      - (/) switch to centered on edinburgh
    - (/) call `regions` endpoint whenever bounding box changes
    - (/) maps returned geojson to regions displayed on the map, which is cleared whenever the bounding box changes
    - (/) hosted under spectrum.houseofmoran.io
      - (/) create domain name mapped to netlify
      - (/) configure netlify

  - (/) service
    - (/) create basic axum webapp, running on fly.io, showing "hello world"
      - (/) basic axum service running locally
      - (/) create fly.io `spectrum` app
      - (/) add tracing setup, sending traces to honeycomb
    - (/) `regions` endpoint that takes a bounding box and returns the bounding box as a polygon

- (x) v0.2: show openstreetmap regions
  - (/) build
    - (/) ingest openstreetmaps extract covering edinburgh
    - (/) find regions (incomplete, as I don't know yet how to handle relations)
    - (/) save as geojson and verify looks correct in geojson.io
    - (/) save as flatgeobuf
  - (/) data
    - (/) just check in flatgeobuf file directly
  - (/) service
    - (/) find all polygons that are in the flatgeobuf within the bounding box and convert to geojson
      - (/) locally
      - (/) on fly.io
- (x) v0.3: add basic search + cleanup pass
  - (/) add mapbox location search
  - (x) union regions
  - (x) clippy hints
- (x) ...
- (x) support relations
