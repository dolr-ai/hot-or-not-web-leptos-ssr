describe("Profile page tests", function () {
  before(function () {
    browser.url(`${browser.launchUrl}/profile/tokens`);
  });

  it("profile page", async function (browser) {
    browser.element
      .findByText("Login", { timeout: 10000 })
      .waitUntil("visible")
      .click();
    browser.element.findByText("Login with Google").waitUntil("visible");
    browser.percySnapshot("SignIn Modal");
  });
});
