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

async function initialiseSource() {
  const source = {
    type: "geojson",
    data: null,
  };

  map.addSource("current", source);

  map.addLayer({
    id: "current",
    type: "fill",
    source: "current",
    layout: {},
    paint: {
      "fill-color": "#0080ff", // blue color fill
      "fill-opacity": 0.5,
    },
  });

  updateSourceOnViewChange();
}

function updateSourceOnViewChange() {
  console.log("view changed");
  const bounds = map.getBounds();
  console.log("bounds, ", bounds);
  console.log("triggering load of geojson");
  fetchLayers().then((geojson) => {
    console.log("geojson loaded");
    const source = map.getSource("current");
    source.setData(geojson);
    console.log("source updated");
  });
}

map.on("load", initialiseSource);
map.on("moveend", updateSourceOnViewChange);
