import "./style.css";

mapboxgl.accessToken = import.meta.env.VITE_MAPBOX_TOKEN;
const edinburgh = [-3.188267, 55.953251];
const map = new mapboxgl.Map({
  container: "map", // container id
  style: "mapbox://styles/mapbox/light-v11", // stylesheet location
  center: edinburgh, // starting position
  zoom: 12, // starting zoom
});
