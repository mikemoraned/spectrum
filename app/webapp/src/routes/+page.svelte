<script>
	// @ts-nocheck

	import mapbox from 'mapbox-gl';
	const { Map } = mapbox;
	import { onMount, onDestroy } from 'svelte';

	import { PUBLIC_MAPBOX_TOKEN } from '$env/static/public';

	let map;
	let mapContainer;

	const edinburgh = [-3.188267, 55.953251];
	const starting_position = {
		center: edinburgh,
		zoom: 12
	};

	onMount(() => {
		map = new Map({
			container: mapContainer,
			accessToken: PUBLIC_MAPBOX_TOKEN,
			style: `mapbox://styles/mapbox/outdoors-v11`,
			...starting_position
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
