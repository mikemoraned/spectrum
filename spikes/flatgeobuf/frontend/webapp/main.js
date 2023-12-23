import "./style.css";

mapboxgl.accessToken = import.meta.env.VITE_MAPBOX_TOKEN;
const edinburgh = [-3.188267, 55.953251];
const maine = [-68.137343, 45.137451];
const starting_position = {
  center: edinburgh,
  // center: maine,
  zoom: 12,
  // zoom: 5,
};
const style = "mapbox://styles/mapbox/streets-v12";
const map = new mapboxgl.Map({
  container: "map", // container id
  style: style,
  ...starting_position,
});

async function fetchLayers() {
  const service_url = `${import.meta.env.VITE_SERVICE_BASE_URL}/layers`;
  console.log("calling service ", service_url, " ...");
  const response = await fetch(service_url);
  const geojson = response.json();
  console.log("called service");
  return geojson;
}

map.on("load", () => {
  console.log("loading ...");

  fetchLayers().then((geojson) => {
    const source = {
      type: "geojson",
      data: geojson,
    };

    map.addSource("maine", source);

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
  });

  console.log("loading done");
});
