import mixpanel from "https://cdn.mxpnl.com/libs/mixpanel-js/dist/mixpanel.module.js";

mixpanel.init("609abef3172b5fc64554f5ac6c77414d", {autocapture: true,  track_pageview: true,  debug: false, persistence: 'localStorage'});

window.mixpanel = mixpanel;

export function trackEvent(name, props = {}) {
  mixpanel.track(name, props);
}

export function identifyUser(userId) {
  mixpanel.identify(userId);
}
