describe("Menu page test", function () {
    before(function () {
        browser.url(`${browser.launchUrl}/menu`)
    });


    it('menu page has option to enable notification', async function (browser) {

        browser.element.findByText('Menu').waitUntil('visible', { timeout: 10000 })
        browser.element.findByText("Login").waitUntil("enabled", { timeout: 10000 })
        browser.percySnapshot('Menu Page');
        let settingsRow = browser.element.findByText("Settings");
        let scrollY = (await settingsRow.getRect()).y;

        browser.perform(function () {
            const actions = this.actions({ async: true })

            return actions.scroll(0, 0, 0, Math.ceil(scrollY))

        })



        let settingsOptions = locateWith(settingsRow).toRightOf(browser.element("svg"));
        browser.click(settingsOptions)

        browser.element.findByText("Enable Notifications").assert.visible()

    })
})