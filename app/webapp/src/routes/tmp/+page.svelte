<script>
	// @ts-nocheck

	import mapbox from 'mapbox-gl';
	const { Map } = mapbox;
	import { onMount, onDestroy } from 'svelte';

	import { PUBLIC_MAPBOX_TOKEN } from '$env/static/public';
	import { PUBLIC_API_BASE_URL } from '$env/static/public';

	let map;
	let mapContainer;

	const edinburgh = [-3.188267, 55.953251];
	const starting_position = {
		center: edinburgh,
		zoom: 12
	};

	onMount(async () => {
		const { MapboxSearchBox } = await import('@mapbox/search-js-web');

		map = new Map({
			container: mapContainer,
			accessToken: PUBLIC_MAPBOX_TOKEN,
			style: `mapbox://styles/mapbox/outdoors-v11`,
			...starting_position
		});

		const search = new MapboxSearchBox();
		search.accessToken = PUBLIC_MAPBOX_TOKEN;
		map.addControl(search);

		map.on('load', initialiseSource);
		map.on('moveend', updateOnViewChange);
	});

	async function initialiseSource() {
		const source = {
			type: 'geojson',
			data: null
		};

		map.addSource('regions', source);

		map.addLayer({
			id: 'regions-borders',
			type: 'line',
			source: 'regions',
			layout: {},
			paint: {
				'line-color': 'black'
			}
		});
		map.addLayer({
			id: 'regions',
			type: 'fill',
			source: 'regions',
			layout: {},
			paint: {
				'fill-color': '#0080ff', // blue color fill
				'fill-opacity': 0.01
			}
		});
		map.addLayer({
			id: 'regions-highlighted',
			type: 'fill',
			source: 'regions',
			layout: {},
			paint: {
				'fill-color': '#0080ff', // blue color fill
				'fill-opacity': 0.5
			},
			filter: ['==', ['id'], null]
		});

		map.on('mousemove', 'regions', (e) => {
			map.getCanvas().style.cursor = 'pointer';

			const feature = e.features[0];
			console.dir(feature);
			map.setFilter('regions-highlighted', ['==', ['id'], feature.id]);
		});

		map.on('mouseleave', 'regions', () => {
			map.getCanvas().style.cursor = '';
			map.setFilter('regions-highlighted', ['==', ['id'], null]);
		});

		updateOnViewChange();
	}

	async function fetchRegions(bounds) {
		const sw = bounds.getSouthWest();
		const ne = bounds.getNorthEast();
		const q = `?sw_lat=${sw.lat}&sw_lon=${sw.lng}&ne_lat=${ne.lat}&ne_lon=${ne.lng}`;
		const service_url = `${PUBLIC_API_BASE_URL}overlaps${q}`;
		console.log('calling service ', service_url, ' ...');
		const response = await fetch(service_url);
		const geojson = response.json();
		console.log('called service');
		return geojson;
	}

	function updateOnViewChange() {
		console.log('view changed');
		const bounds = map.getBounds();
		console.log('bounds, ', bounds);
		console.log('triggering load of geojson');
		fetchRegions(bounds).then((geojson) => {
			console.log('geojson loaded');
			const source = map.getSource('regions');
			source.setData(geojson);
			console.log('source updated');
		});
	}

	onDestroy(() => {
		if (map) {
			map.remove();
		}
	});
</script>

<head>
	<link href="https://api.tiles.mapbox.com/mapbox-gl-js/v3.5.2/mapbox-gl.css" rel="stylesheet" />
</head>

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
