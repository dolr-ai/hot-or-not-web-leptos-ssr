import * as Sentry from 'https://cdn.jsdelivr.net/npm/@sentry/browser@10.5.0/+esm';
import { wasmIntegration } from 'https://cdn.jsdelivr.net/npm/@sentry/wasm@10.5.0/+esm';

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

// Check if we're in debug mode (local development)
const isDebugMode = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1';

// Get release version from meta tag or environment
const releaseVersion = document.querySelector('meta[name="sentry-release"]')?.content || 
                      window.SENTRY_RELEASE || 
                      'unknown';

Sentry.init({
  dsn: "https://3f7d672f8461961bd7b6bec57acf7f18@sentry.yral.com/3",
  
  // Set release version for source map matching
  release: releaseVersion,
  
  // Enable debug in local development
  debug: isDebugMode,
  
  // Set environment based on hostname
  environment: isDebugMode ? 'local-development' : 'production',
  
  integrations: [
    // WASM integration for better WebAssembly debugging
    wasmIntegration(),
    Sentry.browserTracingIntegration(),
    Sentry.captureConsoleIntegration({
      levels: isDebugMode ? ['error', 'warn'] : ['error']
    }),
    Sentry.contextLinesIntegration(),
    Sentry.extraErrorDataIntegration({
      depth: isDebugMode ? 10 : 5
    }),
    Sentry.httpClientIntegration(),
    Sentry.replayIntegration({
      networkDetailAllowUrls: [/^\//, 'yral.com', 'yral-ml-feed-server.fly.dev', 'icp-off-chain-agent.fly.dev', 'localhost'],
      maskAllText: false,
      blockAllMedia: false,
    }),
  ],
  
  // Better WASM error handling
  beforeSend(event, hint) {
    // Enhanced logging in debug mode
    if (isDebugMode) {
      console.log('Sentry Event:', event);
      console.log('Sentry Hint:', hint);
    }
    
    // Improve WASM stack traces
    if (event.exception && event.exception.values) {
      event.exception.values.forEach(exception => {
        if (exception.stacktrace && exception.stacktrace.frames) {
          exception.stacktrace.frames.forEach(frame => {
            // Mark WASM frames as in-app
            if (frame.filename && (frame.filename.includes('.wasm') || frame.filename.includes('wasm-function'))) {
              frame.in_app = true;
              if (!frame.context_line) {
                frame.context_line = '[WASM Module]';
              }
            }
          });
        }
      });
    }
    
    return event;
  },
  
  tracesSampler: tracesSampler,
  replaysSessionSampleRate: isDebugMode ? 1.0 : 0.5, // Capture all sessions in debug
  replaysOnErrorSampleRate: 1.0,
  tracePropagationTargets: [/^\//, 'yral.com', 'yral-ml-feed-server.fly.dev', 'icp-off-chain-agent.fly.dev', 'localhost'],
  
  // Include stack traces for better debugging
  attachStacktrace: true,
  
  // Include more context in errors
  maxValueLength: isDebugMode ? 1000 : 500,
});

// Enhanced global error handlers for WASM panics
window.addEventListener('error', (event) => {
  if (isDebugMode) {
    console.error('Global error caught:', event);
  }
  
  // Check if this is a WASM panic or Rust panic
  if (event.message && (event.message.includes('wasm') || event.message.includes('panic'))) {
    Sentry.captureException(new Error(`WASM/Rust Error: ${event.message}`), {
      tags: {
        source: 'wasm',
        type: 'panic'
      },
      extra: {
        filename: event.filename,
        lineno: event.lineno,
        colno: event.colno,
        error: event.error ? event.error.toString() : 'No error object'
      }
    });
  }
});

window.addEventListener('unhandledrejection', (event) => {
  if (isDebugMode) {
    console.error('Unhandled promise rejection:', event);
  }
  Sentry.captureException(event.reason, {
    tags: {
      source: 'promise',
      type: 'unhandled_rejection'
    }
  });
});

window.Sentry = Sentry;

if (isDebugMode) {
  console.log('Sentry initialized with @sentry/wasm integration and source map support');
  console.log('Sentry release:', releaseVersion);
}
