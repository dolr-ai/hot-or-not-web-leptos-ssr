describe("Menu page test", function () {
    before(function () {
        browser.url(`${browser.launchUrl}/menu`)
    });


    it('menu page has option to enable notification', async function (browser) {

        browser.element.findByText('Menu').waitUntil('visible', { timeout: 10000 })
        browser.element.findByText("Login").waitUntil("enabled", { timeout: 10000 })
        browser.percySnapshot('Menu Page');
        browser.element.findByText('Enable Notifications').waitUntil('visible', { timeout: 10000 })

    })
})