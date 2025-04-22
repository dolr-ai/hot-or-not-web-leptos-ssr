import * as Sentry from 'https://cdn.jsdelivr.net/npm/@sentry/browser@9.13.0/+esm';

// Function to determine the traces sample rate based on localStorage
function tracesSampler(samplingContext) {
  // Check if running in a browser environment where localStorage is available
  if (typeof window !== 'undefined' && window.localStorage) {
    const isInternalUser = window.localStorage.getItem('user-internal');
    // If 'user-internal' is explicitly set to 'true', sample all traces
    if (isInternalUser === 'true') {
      return 1.0;
    }
  }
  // Default sample rate for other users or if localStorage is unavailable/not set
  return 0.5; // 0.25 once stabilised
}

Sentry.init({
  dsn: "https://3f7d672f8461961bd7b6bec57acf7f18@sentry.yral.com/3",
  integrations: [
    Sentry.browserTracingIntegration(),
    Sentry.captureConsoleIntegration(),
    Sentry.contextLinesIntegration(),
    Sentry.extraErrorDataIntegration(),
    Sentry.httpClientIntegration(),
    Sentry.replayIntegration({
      networkDetailAllowUrls: ['localhost', /^\//, 'yral.com', 'yral-ml-feed-server.fly.dev', 'icp-off-chain-agent.fly.dev', 'prod-yral-icpumpsearch.fly.dev', 'prod-yral-nsfw-classification.fly.dev'],
      maskAllText: false,
      blockAllMedia: false,
    }),
  ],
  tracesSampler: tracesSampler,
  replaysSessionSampleRate: 0.5, // 0.1 once stailised
  replaysOnErrorSampleRate: 1.0,
  tracePropagationTargets: ['localhost', /^\//, 'yral.com', 'yral-ml-feed-server.fly.dev', 'icp-off-chain-agent.fly.dev', 'prod-yral-icpumpsearch.fly.dev', 'prod-yral-nsfw-classification.fly.dev'],
});

window.Sentry = Sentry;