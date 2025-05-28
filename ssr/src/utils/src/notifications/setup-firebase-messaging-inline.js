import { initializeApp } from "https://www.gstatic.com/firebasejs/10.14.1/firebase-app.js";
import { getMessaging, onMessage, getToken as firebaseGetToken, deleteToken as firebaseDeleteToken } from "https://www.gstatic.com/firebasejs/10.14.1/firebase-messaging.js";

let isInitialized = false;
let app = null;
let messaging = null;

const vapidKey =
  "BHVXxI5mw_QCsR148ZO4CwxYrsi0EwqJ691arpO4zxa-EMxmrO7odRdX43vpoVQgRcalWVr7Y7sKH_DlWZbpcEI";

function initializeFirebase() {
  if (!isInitialized) {
    app = initializeApp({
      apiKey: "AIzaSyCc_3-30sOgNhpPprV-YDMSTebf4EAPNIo",
      authDomain: "client-device-notification.firebaseapp.com",
      projectId: "client-device-notification",
      storageBucket: "client-device-notification.firebasestorage.app",
      messagingSenderId: "257800168511",
      appId: "1:257800168511:web:ce7840178c24f97e09048a",
      measurementId: "G-WLPMS55C10"
    });
    messaging = getMessaging(app);
    isInitialized = true;
    console.log("Firebase initialized successfully");
  }
  return { app, messaging };
}

export async function getToken() {
  try {
    if (!isInitialized) {
      initializeFirebase();
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    
    console.log("Requesting FCM token...");
    const currentToken = await firebaseGetToken(messaging, { vapidKey: vapidKey });
    if (currentToken) {
      console.log("FCM Token received");
    } else {
      console.log('No registration token available. Request permission to generate one.');
    }
    return currentToken;
  } catch (err) {
    console.error('An error occurred while retrieving token. ', err);
    throw err;
  }
}

export async function deleteFcmToken() {
  try {
    if (!isInitialized) {
      initializeFirebase();
    }
    
    const deleted = await firebaseDeleteToken(messaging);
    if (deleted) {
      console.log("FCM token deleted successfully.");
    } else {
      console.warn("No FCM token found to delete.");
    }
    return deleted;
  } catch (err) {
    console.error("Failed to delete FCM token:", err);
    throw err;
  }
}

export async function getNotificationPermission() {
  try {
    const permission = await Notification.requestPermission();
    const granted = (permission === "granted");
    console.log(`Notification permission ${granted ? 'granted' : 'denied'}`);
    return granted;
  } catch (err) {
    console.error("Error requesting notification permission:", err);
    return false;
  }
}

initializeFirebase();

onMessage(messaging, (payload) => {
  console.log("Message received in JS (foreground), dispatching event.", payload);

  const event = new CustomEvent("firebaseForegroundMessage", { detail: payload });
  window.dispatchEvent(event);

  const data = payload.notification;
  if (data) { 
    const title = data.title || "New Message"; 
    const body = data.body || "You have a new message."; 

    const notificationOptions = {
      body: body,
    };
  }
});