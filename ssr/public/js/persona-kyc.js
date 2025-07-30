// ssr/public/js/kyc.js

let personaClient = null;

async function loadPersonaClient() {
  if (window.Persona) return window.Persona;

  return new Promise((resolve, reject) => {
    const script = document.createElement("script");
    script.src = "https://cdn.withpersona.com/dist/persona-v4.0.0.js";
    script.onload = () => resolve(window.Persona);
    script.onerror = reject;
    document.head.appendChild(script);
  });
}

async function launchPersonaFlow(config, kyc_on_status_change, kyc_on_complete) {
  const Persona = await loadPersonaClient();

  personaClient = new Persona.Client({
    ...config,
    onReady: () => {
      personaClient.open();
    },
    onCancel: () => {
      kyc_on_status_change?.("Pending");
    },
    onComplete: ({ inquiryId, status, fields }) => {
          // Inquiry completed. Optionally tell your server about it.
          console.log(`Sending finished inquiry ${inquiryId} to backend ${status}`);
          console.log("Persona flow completed successfully.");
          kyc_on_complete?.({ inquiryId, status, fields });
    },
    onError: () => {
      console.error("Persona flow encountered an error.");
      kyc_on_status_change?.("Pending");
    },
  });
}

// ✅ Attach to global scope for Rust to access
window.launchPersonaFlow = launchPersonaFlow;
