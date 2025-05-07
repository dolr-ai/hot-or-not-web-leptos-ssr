import { initializeApp } from "https://www.gstatic.com/firebasejs/10.14.1/firebase-app.js";
import { getMessaging, onBackgroundMessage } from "https://www.gstatic.com/firebasejs/10.14.1/firebase-messaging-sw.js";

// TODO: Ensure these details are kept secure and are appropriate for your build process.
// Consider using environment variables or a build step to inject this configuration if needed.
const firebaseConfig = {
  apiKey: "AIzaSyCwo0EWTJz_w-J1lUf9w9NcEBdLNmGUaIo",
  authDomain: "hot-or-not-feed-intelligence.firebaseapp.com",
  projectId: "hot-or-not-feed-intelligence",
  storageBucket: "hot-or-not-feed-intelligence.appspot.com",
  messagingSenderId: "82502260393",
  appId: "1:82502260393:web:390e9d4e588cba65237bb8",
};

const app = initializeApp(firebaseConfig);
const messaging = getMessaging(app);

onBackgroundMessage(messaging, (payload) => {
  console.log("[firebase-messaging-sw.js] Received background message ", payload);

  // Customize notification here
  const notificationTitle = payload.notification?.title || "New Message";
  const notificationOptions = {
    body: payload.notification?.body || "You have a new message.",
    icon: payload.notification?.image || "/default-icon.png", // TODO: Replace with your actual default icon path
    data: payload.data // Pass along any data payload for notification click handling
  };

  self.registration.showNotification(notificationTitle, notificationOptions);
});

// Optional: Handle notification clicks
self.addEventListener('notificationclick', function(event) {
  console.log('[firebase-messaging-sw.js] Notification click Received.', event.notification.data);
  event.notification.close();

  // Example: Focus or open a window
  // This is a basic example, you might want to tailor it based on event.notification.data
  // or the action clicked if you've added action buttons to your notification.
  const FOCUSED_CLIENT_URL = "/"; // TODO: Change to your app's root URL or a specific path from data
  event.waitUntil(
    clients.matchAll({ type: 'window', includeUncontrolled: true }).then(function(clientList) {
      // Check if there's already a window open.
      for (let i = 0; i < clientList.length; i++) {
        const client = clientList[i];
        // If the client's URL matches and it's visible, focus it.
        if (client.url === FOCUSED_CLIENT_URL && 'focus' in client) {
          return client.focus();
        }
      }
      // If no window is open or focused, open a new one.
      if (clients.openWindow) {
        return clients.openWindow(FOCUSED_CLIENT_URL);
      }
    })
  );
});

// This is important to ensure the service worker takes control as soon as possible.
self.addEventListener('install', (event) => {
  console.log('[firebase-messaging-sw.js] Installing service worker...');
  event.waitUntil(self.skipWaiting()); // Forces the waiting service worker to become the active service worker.
});

self.addEventListener('activate', (event) => {
  console.log('[firebase-messaging-sw.js] Activating service worker...');
  // Ensures that the service worker takes control of the page immediately.
  event.waitUntil(clients.claim());
}); 