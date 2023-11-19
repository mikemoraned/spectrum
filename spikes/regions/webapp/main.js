import "./style.css";

mapboxgl.accessToken = import.meta.env.VITE_MAPBOX_TOKEN;
const edinburgh = [-3.188267, 55.953251];
const maine = [-68.137343, 45.137451];
const starting_position = {
  // center: edinburgh,
  center: maine,
  // zoom: 12,
  zoom: 5,
};
const style = "mapbox://styles/mapbox/streets-v12";
const map = new mapboxgl.Map({
  container: "map", // container id
  style: style,
  ...starting_position,
});

async function fetchFeature() {
  console.log("calling service ...");
  const service_url = import.meta.env.VITE_SERVICE_BASE_URL;
  const response = await fetch(service_url);
  console.log("called service");
  const json = {
    type: "geojson",
    data: {
      type: "Feature",
      geometry: {
        type: "Polygon",
        // These coordinates outline Maine.
        coordinates: [
          [
            [-67.13734, 45.13745],
            [-66.96466, 44.8097],
            [-68.03252, 44.3252],
            [-69.06, 43.98],
            [-70.11617, 43.68405],
            [-70.64573, 43.09008],
            [-70.75102, 43.08003],
            [-70.79761, 43.21973],
            [-70.98176, 43.36789],
            [-70.94416, 43.46633],
            [-71.08482, 45.30524],
            [-70.66002, 45.46022],
            [-70.30495, 45.91479],
            [-70.00014, 46.69317],
            [-69.23708, 47.44777],
            [-68.90478, 47.18479],
            [-68.2343, 47.35462],
            [-67.79035, 47.06624],
            [-67.79141, 45.70258],
            [-67.13734, 45.13745],
          ],
        ],
      },
    },
  };
  return json;
}

map.on("load", () => {
  console.log("loading ...");

  fetchFeature().then((json) => {
    map.addSource("maine", json);

    map.addLayer({
      id: "maine",
      type: "fill",
      source: "maine", // reference the data source
      layout: {},
      paint: {
        "fill-color": "#0080ff", // blue color fill
        "fill-opacity": 0.5,
      },
    });

    // Add a black outline around the polygon.
    map.addLayer({
      id: "outline",
      type: "line",
      source: "maine",
      layout: {},
      paint: {
        "line-color": "#000",
        "line-width": 3,
      },
    });
  });

  console.log("loading done");
});
