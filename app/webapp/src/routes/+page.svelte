<script>
	// @ts-nocheck

	import mapbox from 'mapbox-gl';
	const { Map } = mapbox;
	import { onMount, onDestroy } from 'svelte';

	import { PUBLIC_MAPBOX_TOKEN } from '$env/static/public';

	let map;
	let mapContainer;
	let lng, lat, zoom;

	lng = -71.224518;
	lat = 42.213995;
	zoom = 9;

	onMount(() => {
		const initialState = { lng: lng, lat: lat, zoom: zoom };

		map = new Map({
			container: mapContainer,
			accessToken: PUBLIC_MAPBOX_TOKEN,
			style: `mapbox://styles/mapbox/outdoors-v11`,
			center: [initialState.lng, initialState.lat],
			zoom: initialState.zoom
		});
	});

	onDestroy(() => {
		if (map) {
			map.remove();
		}
	});
</script>

<div class="map-wrap">
	<div class="map" bind:this={mapContainer} />
</div>

<style>
	.map {
		position: absolute;
		width: 100%;
		height: 100%;
	}
</style>
