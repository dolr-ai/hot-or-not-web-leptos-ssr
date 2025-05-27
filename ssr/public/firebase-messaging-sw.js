importScripts("https://www.gstatic.com/firebasejs/10.14.1/firebase-app-compat.js");
importScripts("https://www.gstatic.com/firebasejs/10.14.1/firebase-messaging-compat.js");

const firebaseConfig = {

  apiKey: "AIzaSyCc_3-30sOgNhpPprV-YDMSTebf4EAPNIo",

  authDomain: "client-device-notification.firebaseapp.com",

  projectId: "client-device-notification",

  storageBucket: "client-device-notification.firebasestorage.app",

  messagingSenderId: "257800168511",

  appId: "1:257800168511:web:ce7840178c24f97e09048a",

  measurementId: "G-WLPMS55C10"

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

  const FOCUSED_CLIENT_URL = "/"; // TODO: Change to your app's root URL
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