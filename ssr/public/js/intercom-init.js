import { Intercom, show, hide, update, trackEvent } from "https://cdn.jsdelivr.net/npm/@intercom/messenger-js-sdk/+esm";

// Boot Intercom immediately when this module is imported
export function initIntercom() {
  Intercom({ app_id: "nxh9i6ww" });
}

initIntercom();

// Helper functions
export { show, hide, update, trackEvent };
