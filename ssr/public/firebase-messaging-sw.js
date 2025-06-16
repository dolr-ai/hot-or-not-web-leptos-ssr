importScripts("https://www.gstatic.com/firebasejs/10.14.1/firebase-app-compat.js");
importScripts("https://www.gstatic.com/firebasejs/10.14.1/firebase-messaging-compat.js");

const firebaseConfig =  {
  apiKey: "AIzaSyBi3lnHvvPyPlhTk9kG1GZubc477V3q40M",
  authDomain: "yral-mobile.firebaseapp.com",
  projectId: "yral-mobile",
  storageBucket: "yral-mobile.firebasestorage.app",
  messagingSenderId: "1035262663512",
  appId: "1:1035262663512:web:c995288e332798c51c232d",
  measurementId: "G-V2WDJLV7E5"
};

if (!firebase.apps.length) {
  firebase.initializeApp(firebaseConfig);
  console.log("[firebase-messaging-sw.js] Firebase initialized.");
} else {
  firebase.app();
  console.log("[firebase-messaging-sw.js] Firebase already initialized.");
}

const messaging = firebase.messaging();

self.addEventListener('notificationclick', function(event) {
  console.log('[firebase-messaging-sw.js] Notification click Received.', event.notification.data);
  event.notification.close();

  const FOCUSED_CLIENT_URL = "/";
  event.waitUntil(
    clients.matchAll({ type: 'window', includeUncontrolled: true }).then(function(clientList) {
      for (let i = 0; i < clientList.length; i++) {
        const client = clientList[i];
        if (client.url === FOCUSED_CLIENT_URL && 'focus' in client) {
          return client.focus();
        }
      }
      if (clients.openWindow) {
        return clients.openWindow(FOCUSED_CLIENT_URL);
      }
    })
  );
});

self.addEventListener('install', (event) => {
  console.log('[firebase-messaging-sw.js] Installing service worker (compat version)...');
  event.waitUntil(self.skipWaiting());
});

self.addEventListener('activate', (event) => {
  console.log('[firebase-messaging-sw.js] Activating service worker (compat version)...');
  event.waitUntil(clients.claim());
}); 