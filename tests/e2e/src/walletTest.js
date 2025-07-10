describe("wallet page tests", function () {
  before(function () {
    browser.url(`${browser.launchUrl}/wallet`);
  });

  it("wallet page contains login button", async function (browser) {
    browser.waitForElementVisible("body", 25000);
    browser.pause(10000);

    browser.element
      .findByText("Login to claim", { timeout: 50000, exact: false })
      .waitUntil("enabled");
  });

  it("default wallet page contains SATS", function (browser) {
    browser.waitForElementVisible("body", 25000);
    browser.element
        .findByText("SATS", { timeout: 10000 })
        .waitUntil("visible", { timeout: 10000 })
        .assert.enabled();
  });

  it("wallet page snapshot test", function (browser) {
    browser.percySnapshot("Wallet Page");
  });

  it("check usdc  loading", async function (browser) {
    browser.waitForElementVisible("body", 25000);
    browser.pause(10000);
    browser.url(
      `${browser.launchUrl}/wallet/34yzw-zrmgu-vg6ms-2uj2a-czql2-7y4bu-mt5so-ckrtz-znelw-yyvr4-2ae`
    );
    browser.element
      .findByText("USDC", { timeout: 20000 })
      .waitUntil("visible", { timeout: 20000 })
      .assert.enabled();
  });
});
