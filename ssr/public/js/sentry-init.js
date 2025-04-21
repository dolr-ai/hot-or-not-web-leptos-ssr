import * as Sentry from 'https://cdn.jsdelivr.net/npm/@sentry/browser@9.13.0/+esm';

Sentry.init({
  dsn: "https://3f7d672f8461961bd7b6bec57acf7f18@sentry.yral.com/3",
  integrations: [
    Sentry.browserTracingIntegration(),
    Sentry.captureConsoleIntegration(),
    Sentry.httpClientIntegration(),
    Sentry.replayIntegration(),
  ],
  tracesSampleRate: 0.25, // 0.25
  replaysSessionSampleRate: 0.1, // 0.1
  replaysOnErrorSampleRate: 1.0,
  environment: "development",
  tracePropagationTargets: ["localhost", /^https:\/\/yral\.com\/api/],
});

(function () {
  const IMAGES = [];
  const origInstantiateStreaming = WebAssembly.instantiateStreaming;
  const origCompileStreaming = WebAssembly.compileStreaming;

  function getModuleInfo(module) {
    const buildIds = WebAssembly.Module.customSections(module, "build_id");
    let buildId = null;
    let debugFile = null;

    if (buildIds.length > 0) {
      const firstBuildId = new Uint8Array(buildIds[0]);
      buildId = Array.from(firstBuildId).reduce((acc, x, idx) => {
        return acc + (x & 0xff).toString(16).padStart(2, "0");
      }, "");
    }

    const externalDebugInfo = WebAssembly.Module.customSections(
      module,
      "external_debug_info"
    );
    if (externalDebugInfo.length > 0) {
      const firstExternalDebugInfo = new Uint8Array(externalDebugInfo[0]);
      const decoder = new TextDecoder("utf-8");
      debugFile = decoder.decode(firstExternalDebugInfo);
    }

    return { buildId, debugFile };
  }

  function recordModule(module, url) {
    const { buildId, debugFile } = getModuleInfo(module);
    if (buildId || debugFile) {
      const oldIdx = IMAGES.findIndex((img) => img.code_file === url);
      if (oldIdx >= 0) {
        IMAGES.splice(oldIdx, 1);
      }
      IMAGES.push({
        type: "wasm",
        code_id: buildId,
        code_file: url,
        debug_file: debugFile,
        debug_id: buildId.padEnd(32, "0").substr(0, 32) + "0",
      });
    }
  }

  function recordedInstanticateStreaming(promise, obj) {
    return Promise.resolve(promise).then((resp) => {
      return origInstantiateStreaming(resp, obj).then((rv) => {
        if (resp.url) {
          recordModule(rv.module, resp.url);
        }
        return rv;
      });
    });
  }

  function recordedCompileStreaming(promise, obj) {
    return Promise.resolve(promise).then((resp) => {
      return origCompileStreaming(resp, obj).then((module) => {
        if (resp.url) {
          recordModule(module, resp.url);
        }
        return module;
      });
    });
  }

  function getDebugMeta() {
    return {
      images: IMAGES,
    };
  }

  function getImageIndex(url) {
    return IMAGES.findIndex((img) => img.code_file === url);
  }

  Sentry.addEventProcessor((event) => {
    let haveWasm = false;
    if (event.exception && event.exception.values) {
      event.exception.values.forEach((exception) => {
        if (!exception.stacktrace) {
          return;
        }
        exception.stacktrace.frames.forEach((frame) => {
          let match;
          if (
            frame.filename &&
            (match = frame.filename.match(
              /^(.*?):wasm-function\[\d+\]:(0x[a-fA-F0-9]+)$/
            ))
          ) {
            const index = getImageIndex(match[1]);
            if (index >= 0) {
              frame.instruction_addr = match[2];
              frame.addr_mode = `rel:${index}`;
              frame.filename = match[1];
              frame.platform = "native";
              haveWasm = true;
            }
          }
        });
      });
    }

    if (haveWasm) {
      event.debug_meta = getDebugMeta();
    }

    return event;
  });

  WebAssembly.instantiateStreaming = recordedInstanticateStreaming;
  WebAssembly.compileStreaming = recordedCompileStreaming;
})();

window.Sentry = Sentry;