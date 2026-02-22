const CACHE_NAME = 'immer-in-bewegung-rs';

const urlsToCache = [
  './index.html',
  './version',
  './pkg/bewegung.js',
  './pkg/bewegung_bg.wasm',
  './static/script/bew-map.js',
  './static/script/bew-toolbox-input.js',
  './static/script/bew-filter.js',
  './static/script/bew-chart.js',
  './static/script/bew-tabulator.js',
  './static/script/bew-immich.js',
  './bundle/fonts/Righteous-Regular.ttf',
  './bundle/fonts/OFL.txt',
  './bundle/fonts/FrancoisOne-Regular.ttf',
  './bundle/fonts/Cairo-VariableFont_slnt,wght.ttf',
  './bundle/maplibre-gl/maplibre-gl.css',
  './bundle/maplibre-gl/maplibre-gl.js',
  './bundle/tabulator/tabulator.min.js',
  './bundle/tabulator/tabulator.min.css',
  './bundle/chartjs/chart.js',
  './bundle/codemirror/sql.min.js',
  './bundle/codemirror/source.txt',
  './static/images/photos.svg',
  './static/images/frog.svg',
  './static/images/house.svg',
  './static/images/database.svg',
  './static/images/diary.svg',
  './static/images/bag.svg',
  './static/images/console.svg',
  './static/images/travel.svg',
  './static/images/remove.svg',
  './static/images/ext.svg',
  './static/images/arrow-left.svg',
  './static/images/run.svg',
  './static/images/funnel.svg',
  './static/images/reload.svg',
  './static/images/arrow-right.svg',
  './static/images/persons.svg',
  './static/images/frog_g_72.webp',
  './static/images/layers.svg',
  './static/images/immich-logo-inline-dark-small.png',
  './static/images/passport.svg',
  './static/languages/swedish.json',
  './static/bewegung.css'
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    (async () => {
      const cache = await caches.open(CACHE_NAME);
      await Promise.allSettled(urlsToCache.map((url) => cache.add(url)));
      await self.skipWaiting();
    })()
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    (async () => {
      const cacheNames = await caches.keys();
      await Promise.all(cacheNames.filter((name) => name !== CACHE_NAME).map((name) => caches.delete(name)));
      await self.clients.claim();
    })()
  );
});

self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET') {
    return;
  }

  event.respondWith(
    (async () => {
      try {
        const reqUrl = new URL(event.request.url);

        if (
          reqUrl.href.includes('tile.openstreetmap.org') ||
          reqUrl.href === 'https://raw.githubusercontent.com/plans-coding/immer-in-bewegung/refs/heads/main/version' ||
          (reqUrl.origin === 'https://immich.karlaplan.dedyn.io' &&
            reqUrl.pathname.endsWith('/static/bew-video-player.html'))
        ) {
          return fetch(event.request);
        }

        if (reqUrl.origin !== self.location.origin) {
          return fetch(event.request);
        }

        const cacheKeyUrl = new URL(reqUrl.href);
        cacheKeyUrl.search = '';
        cacheKeyUrl.hash = '';
        const cacheKey = cacheKeyUrl.toString();

        const cache = await caches.open(CACHE_NAME);
        const cachedResponse = await cache.match(cacheKey);
        if (cachedResponse) {
          return cachedResponse;
        }

        const networkResponse = await fetch(event.request);
        if (networkResponse && networkResponse.ok && networkResponse.type === 'basic') {
          await cache.put(cacheKey, networkResponse.clone());
        }
        return networkResponse;
      } catch (error) {
        if (event.request.mode === 'navigate') {
          const fallback = await caches.match('./index.html');
          if (fallback) {
            return fallback;
          }
        }
        return new Response('Resource not found', { status: 404 });
      }
    })()
  );
});
