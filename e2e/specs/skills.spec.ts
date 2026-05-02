import { expect } from "@wdio/globals";

describe("Skills Plugin", () => {
  // Helper: navigate to skills page by finding and clicking the correct sub-menu item
  async function navigateToSkillsMain() {
    // The sidebar has "Skill 管理" as a group with children.
    // First click the group to expand it, then click "全局 Skills"
    // Use NMenu's internal structure: group items have .n-menu-item-content-header

    // Use http://tauri.localhost/ format (WebView2 compatible)
    await browser.url("http://tauri.localhost/skills");
    await browser.pause(1000);
  }

  describe("Skills Main Page", () => {
    it("should display the skills main page", async () => {
      await navigateToSkillsMain();
      const page = await $("[data-testid='skills-main-page']");
      await page.waitForExist({ timeout: 15000 });
      expect(await page.isExisting()).toBe(true);
    });

    it("should display the page title and action buttons", async () => {
      await navigateToSkillsMain();
      const page = await $("[data-testid='skills-main-page']");
      await page.waitForExist({ timeout: 15000 });

      // Title text should contain "Skill"
      const titleText = await $("[data-testid='skills-main-page'] .n-text");
      const text = await titleText.getText();
      expect(text).toContain("Skill");

      // History and Migrate buttons should exist
      expect(await $("[data-testid='btn-history']").isExisting()).toBe(true);
      expect(await $("[data-testid='btn-migrate']").isExisting()).toBe(true);
    });

    it("should show agent selector dropdown", async () => {
      await navigateToSkillsMain();
      const page = await $("[data-testid='skills-main-page']");
      await page.waitForExist({ timeout: 15000 });

      // Agent select component should be visible
      const selects = await $$("[data-testid='skills-main-page'] .n-select");
      expect(selects.length).toBeGreaterThanOrEqual(1);
    });

    it("should show comparison table or empty state", async () => {
      await navigateToSkillsMain();
      const page = await $("[data-testid='skills-main-page']");
      await page.waitForExist({ timeout: 15000 });

      // Wait for loading to finish — either a data table or empty state appears
      await browser.pause(2000);

      const hasTable = (await $$(".n-data-table")).length > 0;
      const hasEmpty = (await $$(".n-empty")).length > 0;
      const hasAlert = (await $$(".n-alert")).length > 0;
      expect(hasTable || hasEmpty || hasAlert).toBe(true);
    });
  });

  describe("Operation History Panel", () => {
    it("should open history drawer when button clicked", async () => {
      await navigateToSkillsMain();
      const page = await $("[data-testid='skills-main-page']");
      await page.waitForExist({ timeout: 15000 });

      const btnHistory = await $("[data-testid='btn-history']");
      await btnHistory.waitForClickable({ timeout: 5000 });
      await btnHistory.click();
      await browser.pause(800);

      // Drawer should appear
      const drawer = await $("[data-testid='history-drawer']");
      await drawer.waitForExist({ timeout: 5000 });
      expect(await drawer.isDisplayed()).toBe(true);
    });

    it("should show empty state or history list in drawer", async () => {
      // Drawer is already open from previous test
      const drawer = await $("[data-testid='history-drawer']");
      await drawer.waitForExist({ timeout: 5000 });

      // Should show either empty state or a list of history items
      const hasEmpty = (await $$(".n-empty")).length > 0;
      const hasItems = (await $$(".history-item")).length > 0;
      expect(hasEmpty || hasItems).toBe(true);
    });

    it("should show clear history button", async () => {
      const drawer = await $("[data-testid='history-drawer']");
      await drawer.waitForExist({ timeout: 5000 });

      // "清空历史" button should be in the drawer header
      const buttons = await $$("[data-testid='history-drawer'] button");
      let hasClearBtn = false;
      for (const btn of buttons) {
        const text = await btn.getText();
        if (text.includes("清空") || text.includes("Clear")) {
          hasClearBtn = true;
          break;
        }
      }
      expect(hasClearBtn).toBe(true);
    });
  });

  describe("Migration Dialog", () => {
    it("should open migration dialog when button clicked", async () => {
      await navigateToSkillsMain();
      const page = await $("[data-testid='skills-main-page']");
      await page.waitForExist({ timeout: 15000 });

      const btnMigrate = await $("[data-testid='btn-migrate']");
      await btnMigrate.waitForClickable({ timeout: 5000 });
      await btnMigrate.click();
      await browser.pause(800);

      const dialog = await $("[data-testid='migrate-dialog']");
      await dialog.waitForExist({ timeout: 5000 });
      expect(await dialog.isDisplayed()).toBe(true);
    });

    it("should show step indicator and agent selector", async () => {
      const dialog = await $("[data-testid='migrate-dialog']");
      await dialog.waitForExist({ timeout: 5000 });

      // Step indicator should be present
      const stepIndicator = await $("[data-testid='migrate-dialog'] .step-indicator");
      expect(await stepIndicator.isExisting()).toBe(true);

      // Agent select should be present
      const selects = await $("[data-testid='migrate-dialog'] .n-select");
      expect(await selects.isExisting()).toBe(true);
    });

    it("should display step circles (1, 2, 3)", async () => {
      const steps = await $$("[data-testid='migrate-dialog'] .step-circle");
      expect(steps.length).toBe(3);
    });
  });

  describe("Guide Page", () => {
    it("should navigate to guide page", async () => {
      await browser.url("http://tauri.localhost/skills/guide");
      await browser.pause(1500);

      // Guide page should render — look for guide-related elements
      // The page has a two-column layout with NLayout
      const guideContent = await $(".n-layout");
      await guideContent.waitForExist({ timeout: 10000 });
      expect(await guideContent.isExisting()).toBe(true);
    });

    it("should display guide sections with code blocks", async () => {
      // Guide should have sections — look for headings or code blocks
      const hasHeadings = (await $$("h2, h3, h4")).length > 0;
      const hasCode = (await $$("code, pre, .hljs")).length > 0;
      expect(hasHeadings || hasCode).toBe(true);
    });
  });
});
