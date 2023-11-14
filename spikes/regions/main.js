import "./style.css";

mapboxgl.accessToken = import.meta.env.VITE_MAPBOX_TOKEN;
const map = new mapboxgl.Map({
  container: "map", // container id
  style: "mapbox://styles/mapbox/light-v11", // stylesheet location
  center: [-84.5, 38.05], // starting position
  zoom: 12, // starting zoom
});
