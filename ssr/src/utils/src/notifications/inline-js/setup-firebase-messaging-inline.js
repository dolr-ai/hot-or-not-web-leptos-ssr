import { initializeApp } from "https://www.gstatic.com/firebasejs/10.14.1/firebase-app.js";
import { getMessaging, onMessage, getToken as firebaseGetToken, deleteToken as firebaseDeleteToken } from "https://www.gstatic.com/firebasejs/10.14.1/firebase-messaging.js";

let isInitialized = false;
let app = null;
let messaging = null;

const vapidKey =
  "BGVNVR0JA9emg1nqMp7-yUv9FRWqqqI5aaoIr5vuScgIZBNqxiaNKHbqJLoelPrwHby9i8193Z_l4UpsliPDHQI";

function initializeFirebase() {
  if (!isInitialized) {
    app = initializeApp({
      apiKey: "AIzaSyBi3lnHvvPyPlhTk9kG1GZubc477V3q40M",
      authDomain: "yral-mobile.firebaseapp.com",
      projectId: "yral-mobile",
      storageBucket: "yral-mobile.firebasestorage.app",
      messagingSenderId: "1035262663512",
      appId: "1:1035262663512:web:c995288e332798c51c232d",
      measurementId: "G-V2WDJLV7E5"
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