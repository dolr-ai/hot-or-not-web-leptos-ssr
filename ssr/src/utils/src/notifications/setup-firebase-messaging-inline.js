import { initializeApp } from "https://www.gstatic.com/firebasejs/10.14.1/firebase-app.js";
// Import onMessage for client-side foreground message handling
import { getMessaging, onMessage } from "https://www.gstatic.com/firebasejs/10.14.1/firebase-messaging.js";

const app = initializeApp({
  apiKey: "AIzaSyCwo0EWTJz_w-J1lUf9w9NcEBdLNmGUaIo",
  authDomain: "hot-or-not-feed-intelligence.firebaseapp.com",
  projectId: "hot-or-not-feed-intelligence",
  storageBucket: "hot-or-not-feed-intelligence.appspot.com",
  messagingSenderId: "82502260393",
  appId: "1:82502260393:web:390e9d4e588cba65237bb8",
});

const messaging = getMessaging(app);

const vapidKey =
  "BOmsEya6dANYUoElzlUWv3Jekmw08_nqDEUFu06aTak-HQGd-G_Lsk8y4Bs9B4kcEjBM8FXF0IQ_oOpJDmU3zMs";

export async function getToken() {
  // Note: Firebase docs show getToken(messaging, {vapidKey: "..."}) for modular SDK.
  // This messaging.getToken() might be for compat mode. If issues, check Firebase docs.
  const currentToken = await messaging.getToken({ vapidKey: vapidKey });
  return currentToken;
}

export async function getDeviceFingerprint() {
  // Collect basic device info
  const userAgent = navigator.userAgent;
  const screenResolution = `${screen.width}x${screen.height}`;
  const language = navigator.language;
  const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone;

  // Generate a unique string
  let fingerprintString = `${userAgent}|${screenResolution}|${language}|${timezone}`;

  // Hash the string (using SHA-256 for example)
  const hash = await sha256(fingerprintString);
  return hash;
}

async function sha256(message) {
  const encoder = new TextEncoder();
  const data = encoder.encode(message);
  const hashBuffer = await crypto.subtle.digest("SHA-256", data);
  const hashArray = Array.from(new Uint8Array(hashBuffer)); // Convert buffer to byte array
  const hashHex = hashArray
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
  return hashHex;
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
  // You might want to remove this if Leptos handles the UI exclusively.
  const { notification: notificationData } = payload;
  if (notificationData) {
    const { title, body, image } = notificationData;
    const notificationOptions = {
      body: body,
      icon: image,
    };
    const notification = new Notification(title || "New Message", notificationOptions);
    notification.onerror = (err) => {
      console.error("Error displaying JS notification:", err);
    };
  }
});

// onBackgroundMessage logic should NOT be in this file.
// It belongs in your firebase-messaging-sw.js (service worker).