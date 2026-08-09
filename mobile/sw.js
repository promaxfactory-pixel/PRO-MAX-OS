const VERSION = 'v2';
const CACHE = 'promax-' + VERSION;
const CORE = [
  '/',
  '/index.html',
  '/styles.css',
  '/app.js',
  '/manifest.webmanifest',
  '/icons/icon-192.png',
  '/icons/icon-512.png'
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE).then((cache) =>
      Promise.all(CORE.map((u) => cache.add(u).catch(() => null)))
    ).then(() => self.skipWaiting())
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k)))
    ).then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);
  if (event.request.method !== 'GET' || url.pathname.startsWith('/api/')) {
    return;
  }
  if (url.origin !== self.location.origin) {
    return;
  }
  const isNav = event.request.mode === 'navigate';
  const networkFirst = isNav || url.pathname === '/app.js' || url.pathname === '/manifest.webmanifest';

  event.respondWith((async () => {
    if (networkFirst) {
      try {
        const res = await fetch(event.request);
        if (res && res.ok) {
          const copy = res.clone();
          caches.open(CACHE).then((cache) => cache.put(event.request, copy));
        }
        return res;
      } catch (e) {
        const cached = await caches.match(event.request);
        if (cached) return cached;
        const fallback = await caches.match('/index.html');
        if (fallback) return fallback;
        throw e;
      }
    }
    const cached = await caches.match(event.request);
    if (cached) {
      fetch(event.request)
        .then((res) => {
          if (res && res.ok) {
            const copy = res.clone();
            caches.open(CACHE).then((cache) => cache.put(event.request, copy));
          }
        })
        .catch(() => {});
      return cached;
    }
    try {
      const res = await fetch(event.request);
      if (res && res.ok) {
        const copy = res.clone();
        caches.open(CACHE).then((cache) => cache.put(event.request, copy));
      }
      return res;
    } catch (e) {
      const fallback = await caches.match('/index.html');
      if (fallback) return fallback;
      throw e;
    }
  })());
});
