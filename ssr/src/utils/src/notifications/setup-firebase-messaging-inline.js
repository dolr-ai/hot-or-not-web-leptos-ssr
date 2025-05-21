import { initializeApp } from "https://www.gstatic.com/firebasejs/10.14.1/firebase-app.js";
// Import onMessage and getToken for client-side foreground message handling
import { getMessaging, onMessage, getToken as firebaseGetToken } from "https://www.gstatic.com/firebasejs/10.14.1/firebase-messaging.js";

const app = initializeApp({

  apiKey: "AIzaSyCc_3-30sOgNhpPprV-YDMSTebf4EAPNIo",

  authDomain: "client-device-notification.firebaseapp.com",

  projectId: "client-device-notification",

  storageBucket: "client-device-notification.firebasestorage.app",

  messagingSenderId: "257800168511",

  appId: "1:257800168511:web:ce7840178c24f97e09048a",

  measurementId: "G-WLPMS55C10"

});

const messaging = getMessaging(app);

const vapidKey =
  "BHVXxI5mw_QCsR148ZO4CwxYrsi0EwqJ691arpO4zxa-EMxmrO7odRdX43vpoVQgRcalWVr7Y7sKH_DlWZbpcEI";

// Renamed the imported getToken to avoid conflict if there was a local getToken variable elsewhere
export async function getToken() {
  try {
    console.log("Requesting FCM token...");
    const currentToken = await firebaseGetToken(messaging, { vapidKey: vapidKey });
    if (currentToken) {
      console.log("FCM Token received: ", currentToken);
    } else {
      console.log('No registration token available. Request permission to generate one.');
    }
    return currentToken;
  } catch (err) {
    console.error('An error occurred while retrieving token. ', err);
    throw err; // Re-throw the error so wasm_bindgen can catch it
  }
}

export async function getNotificationPermission() {
  const permission = await Notification.requestPermission();
  return permission === "granted";
}

// Handles messages when the web app is in the foreground
onMessage(messaging, (payload) => {
  console.log("Message received in JS (foreground), dispatching event.", payload);

  // Dispatch a custom event for Leptos to handle
  const event = new CustomEvent("firebaseForegroundMessage", { detail: payload });
  window.dispatchEvent(event);

  // Optionally, still show a default browser notification from JS
  const data = payload.notification;
  if (data) { 
    const title = data.title || "New Message"; 
    const body = data.body || "You have a new message."; 

    const notificationOptions = {
      body: body,
    };
    const notification = new Notification(title, notificationOptions);
    notification.onerror = (err) => {
      console.error("Error displaying JS notification:", err);
    };
  }
});

// onBackgroundMessage logic should NOT be in this file.
// It belongs in your firebase-messaging-sw.js (service worker).