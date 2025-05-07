import { initializeApp } from "https://www.gstatic.com/firebasejs/10.14.1/firebase-app.js";
import { getMessaging } from "https://www.gstatic.com/firebasejs/10.14.1/firebase-messaging.js";

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

// This is called when a message is received while the app is in the foreground
messaging.onMessage((payload) => {
  console.log("Message received. ", payload);

  const { notification: notificationData } = payload;

  // We can modify the notification here

  const { title, body, image } = notificationData;
  const notificationOptions = {
    body,
    icon: image,
  };

  // Send a notification to the user
  const notification = new Notification(title, notificationOptions);
  notification.onerror = (event) => {
    console.log("Could not send notification");
    console.log(event);
  };
});

messaging.onBackgroundMessage((payload) => {
  const { notification } = payload;

  // We can modify the notification here

  const { title, body, image } = notification;
  const notificationOptions = {
    body,
    icon: image,
  };

  self.registration.showNotification(title, notificationOptions);
});