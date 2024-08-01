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

- (x) v0.1: show regions

  - (x) webapp

    - (/) get example sveltekit app working on netlify
      - (/) example sveltekit app working locally
      - (/) deployed on netlify; https://deploy-preview-8--spectrum-green.netlify.app
    - (/) show map, initially focussed on edinburgh
      - (/) switch to default setup for sveltekit
      - (/) add basic mapbox setup
      - (/) switch to centered on edinburgh
    - (x) call `regions` endpoint whenever boundbox box changes
    - (x) maps returned geojson to regions displayed on the map, which is cleared whenever the bounding box changes
    - (x) hosted under spectrum.houseofmoran.io
      - (x) create domain name mapped to netlify
      - (x) configure netlify

  - (x) build
    - (x) ingest openstreetmaps extract covering edinburgh
    - (x) find regions (incomplete, I think I don't know yet how to cover ways)
    - (x) save as flatgeobuf
  - (x) data
    - (x) just check in flatgeobuf file directly
  - (x) service
    - (/) create basic axum webapp, running on fly.io, showing "hello world"
      - (/) basic axum service running locally
      - (/) create fly.io `spectrum` app
      - (/) add tracing setup, sending traces to honeycomb
    - (x) `regions` endpoint that:
      - takes a bounding box
      - finds the shapes in the flatgeobuf that are in that bb
      - converts to geojson and returns it

- (x) ...
