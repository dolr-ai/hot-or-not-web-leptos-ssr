(function () {
  const currentUrl = window.location.href;
  const hasUTM = currentUrl.toLowerCase().includes("utm");
  const alreadyStored = localStorage.getItem("initial_url");

  // Store only if not already stored and URL contains UTM parameters
  if (!alreadyStored && hasUTM) {
    localStorage.setItem("initial_url", currentUrl);
  }
})();
