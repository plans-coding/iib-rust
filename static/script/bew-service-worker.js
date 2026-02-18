const CACHE_NAME = 'immer-in-bewegung-rs';


const urlsToCache = [

    './index.html',
    './version',

    '../pkg/bewegung.js',
    '../pkg/bewegung_bg.wasm',

    '../static/script/bew-map.js',
    '../static/script/bew-toolbox-input.js',
    '../static/script/bew-filter.js',
    '../static/script/bew-chart.js',
    '../static/script/bew-tabulator.js',
    '../static/script/bew-immich.js',

    '../bundle/fonts/Righteous-Regular.ttf',
    '../bundle/fonts/OFL.txt',
    '../bundle/fonts/FrancoisOne-Regular.ttf',
    '../bundle/fonts/Cairo-VariableFont_slnt,wght.ttf',
    '../bundle/maplibre-gl/maplibre-gl.css',
    '../bundle/maplibre-gl/maplibre-gl.js',
    '../bundle/tabulator/tabulator.min.js',
    '../bundle/tabulator/tabulator.min.css',
    '../bundle/chartjs/chart.js',
    '../bundle/codemirror/sql.min.js',
    '../bundle/codemirror/source.txt',

    '../static/images/photos.svg',
    '../static/images/frog.svg',
    '../static/images/house.svg',
    '../static/images/database.svg',
    '../static/images/diary.svg',
    '../static/images/bag.svg',
    '../static/images/console.svg',
    '../static/images/travel.svg',
    '../static/images/remove.svg',
    '../static/images/ext.svg',
    '../static/images/arrow-left.svg',
    '../static/images/run.svg',
    '../static/images/funnel.svg',
    '../static/images/reload.svg',
    '../static/images/arrow-right.svg',
    '../static/images/persons.svg',
    '../static/images/frog_g_72.webp',
    '../static/images/layers.svg',
    '../static/images/immich-logo-inline-dark-small.png',
    '../static/images/passport.svg',

    '../static/languages/swedish.json',
    '../static/bewegung.css'
];

self.addEventListener('install', event => {
    event.waitUntil(
        caches.open(CACHE_NAME)
            .then(cache => {
                console.log('Opened cache');
                return cache.addAll(urlsToCache)
                    .catch(err => {
                        console.error('Caching failed:', err);
                        urlsToCache.forEach(async url => {
                            try {
                                await cache.add(url);
                            } catch (error) {
                                console.error(`Failed to cache ${url}:`, error);
                            }
                        });
                    });
            })
    );
});

// Activate event - Cleanup old caches
self.addEventListener('activate', event => {
    event.waitUntil(
        caches.keys().then(cacheNames => {
            return Promise.all(
                cacheNames.filter(name => name !== CACHE_NAME)
                    .map(name => caches.delete(name))
            );
        })
    );
});

/*
self.addEventListener('fetch', (event) => {
  event.respondWith(
    (async () => {
      try {
        // Skip caching for Leaflet tile requests
        if (event.request.url.includes('tile.openstreetmap.org')) {
          return fetch(event.request);
        }

        // Check if the requested resource is available in cache
        const response = await caches.match(event.request);

        // If the resource is available in cache, return it
        if (response) {
          return response;
        }

        // If not available in cache, try fetching from the network
        const networkResponse = await fetch(event.request);

        // If the network request is successful, cache the response and return it
        const cache = await caches.open(CACHE_NAME);
        cache.put(event.request, networkResponse.clone());

        return networkResponse;
      } catch (error) {
        // If the resource is not available (both in cache and network), drop the request
        console.warn(`Resource not available: ${event.request.url}`);
        return new Response('Resource not found', { status: 404 });
      }
    })()
  );
});*/

self.addEventListener('fetch', (event) => {
  event.respondWith(
    (async () => {
      try {

        /*if (event.request.url === 'https://raw.githubusercontent.com/plans-coding/immer-in-bewegung/refs/heads/main/version') {
          return fetch(event.request);
        }

        if (event.request.url.includes('tile.openstreetmap.org')) {
          return fetch(event.request);
        }*/

        const reqUrl = event.request.url;

        // ---- EXCLUDES (network only, never cache) ----
        if (
          reqUrl.includes('tile.openstreetmap.org') ||
          reqUrl === 'https://raw.githubusercontent.com/plans-coding/immer-in-bewegung/refs/heads/main/version' ||
          reqUrl.startsWith('https://immich.karlaplan.dedyn.io/bewrust/static/bew-video-player.html')
        ) {
          return fetch(event.request);
        }

        // Normalize URL by stripping query parameters (optional, depends on needs) to handle eg ?p=map&country=Sweden
        const url = new URL(event.request.url);
        url.search = ''; // Remove all query parameters

        const cache = await caches.open(CACHE_NAME);
        const response = await cache.match(url.toString());

        if (response) {
          return response;
        }

        // Fetch from network
        const networkResponse = await fetch(event.request);
        cache.put(event.request, networkResponse.clone());

        return networkResponse;
      } catch (error) {
        console.warn(`Resource not available: ${event.request.url}`);
        return new Response('Resource not found', { status: 404 });
      }
    })()
  );
});



