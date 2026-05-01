describe("Welcome Page", () => {
  it("should display the app window", async () => {
    const title = await browser.getTitle();
    expect(title).toBe("AI Cockpit");
  });

  it("should render the welcome page", async () => {
    const welcomePage = await $("[data-testid='welcome-page']");
    await welcomePage.waitForExist({ timeout: 10000 });
    expect(await welcomePage.isExisting()).toBe(true);
  });

  it("should have a start button that navigates to settings", async () => {
    const startBtn = await $("[data-testid='welcome-start-btn']");
    await startBtn.waitForClickable({ timeout: 5000 });
    await startBtn.click();
    await browser.pause(500);
    // Verify navigation away from welcome page
    const url = await browser.getUrl();
    expect(url).toContain("/settings");
  });
});