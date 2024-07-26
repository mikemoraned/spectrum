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

- (x) v0.1
  - (x) webapp
    - (x) show map, initially focussed on edinburgh
    - (x) call `routes` endpoint whenever boundbox box changes
    - (x) maps returned geojson to a route displayed on the map, which is cleared whenever the bounding box changes
  - (x) service
    - (x) `routes` endpoint that takes a bounding box and returns a random set of routes that overlap that bounding box, represented as geojson
  - build: none
  - data: none
- (x) ...
