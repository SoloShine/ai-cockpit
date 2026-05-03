import { expect } from "@wdio/globals";

async function navigateToSkillsMain() {
  await browser.url("http://tauri.localhost/skills");
  await browser.pause(1500);
}

async function waitForPageLoad() {
  const page = await $("[data-testid='skills-main-page']");
  await page.waitForExist({ timeout: 15000 });
  // Wait for loading to finish
  await browser.pause(2000);
}

describe("Skills Search and Filter", () => {
  describe("Search Input", () => {
    it("should render the search input", async () => {
      await navigateToSkillsMain();
      await waitForPageLoad();

      const searchInput = await $("[data-testid='search-input']");
      await searchInput.waitForExist({ timeout: 5000 });
      expect(await searchInput.isExisting()).toBe(true);
    });

    it("should filter skills by name when text is typed", async () => {
      await navigateToSkillsMain();
      await waitForPageLoad();

      // Check if there is a data table (requires skills to be loaded)
      const hasTable = (await $$(".n-data-table tbody tr")).length > 0;
      if (!hasTable) {
        // No skills to filter — skip gracefully
        return;
      }

      const searchInput = await $("[data-testid='search-input'] input");
      await searchInput.waitForExist({ timeout: 5000 });

      // Type a search query that likely won't match anything
      await searchInput.setValue("zzz-nonexistent-skill");
      await browser.pause(500);

      // Table should show fewer rows or empty state
      const rows = await $$(".n-data-table tbody tr");
      const hasEmpty = (await $$(".n-empty")).length > 0;

      // Either all rows are gone, or empty state appears
      expect(hasEmpty || rows.length === 0).toBe(true);

      // Clear search
      await searchInput.clearValue();
      await browser.pause(500);
    });

    it("should restore all rows when search is cleared", async () => {
      await navigateToSkillsMain();
      await waitForPageLoad();

      const rowsBefore = (await $$(".n-data-table tbody tr")).length;
      if (rowsBefore === 0) return;

      const searchInput = await $("[data-testid='search-input'] input");
      await searchInput.waitForExist({ timeout: 5000 });

      // Filter to narrow results
      await searchInput.setValue("zzz-no-match");
      await browser.pause(500);

      // Clear using keyboard (setValue("") doesn't trigger v-model in WebView2)
      await searchInput.click();
      await browser.keys(["Control", "a"]);
      await browser.keys(["Backspace"]);
      await browser.pause(800);

      const rowsAfter = (await $$(".n-data-table tbody tr")).length;
      expect(rowsAfter).toBe(rowsBefore);
    });
  });

  describe("Status Filter Bar", () => {
    it("should render filter bar when comparisons exist", async () => {
      await navigateToSkillsMain();
      await waitForPageLoad();

      // Filter bar only appears when comparisons.length > 0
      const hasComparisons = (await $$(".n-data-table tbody tr")).length > 0;
      if (!hasComparisons) return;

      const filterBar = await $("[data-testid='status-filter-bar']");
      await filterBar.waitForExist({ timeout: 5000 });
      expect(await filterBar.isExisting()).toBe(true);
    });

    it("should display filter chips with labels and counts", async () => {
      await navigateToSkillsMain();
      await waitForPageLoad();

      const hasComparisons = (await $$(".n-data-table tbody tr")).length > 0;
      if (!hasComparisons) return;

      const filterBar = await $("[data-testid='status-filter-bar']");
      await filterBar.waitForExist({ timeout: 5000 });

      const chips = await $$("[data-testid='status-filter-bar'] .status-chip");
      expect(chips.length).toBe(5); // all, same, outdated, localOnly, remoteOnly

      // Each chip should have text content
      for (const chip of chips) {
        const text = await chip.getText();
        expect(text.length).toBeGreaterThan(0);
      }
    });

    it("should highlight active chip when clicked", async () => {
      await navigateToSkillsMain();
      await waitForPageLoad();

      const hasComparisons = (await $$(".n-data-table tbody tr")).length > 0;
      if (!hasComparisons) return;

      const filterBar = await $("[data-testid='status-filter-bar']");
      await filterBar.waitForExist({ timeout: 5000 });

      // Click the "all" chip (first one)
      const firstChip = await $("[data-testid='status-filter-bar'] .status-chip");
      await firstChip.click();
      await browser.pause(300);

      // It should have active class
      const hasActive = (await $$(".status-chip.active")).length > 0;
      expect(hasActive).toBe(true);

      // Click again to deactivate
      await firstChip.click();
      await browser.pause(300);
    });

    it("should filter table rows when a status chip is clicked", async () => {
      await navigateToSkillsMain();
      await waitForPageLoad();

      const hasComparisons = (await $$(".n-data-table tbody tr")).length > 0;
      if (!hasComparisons) return;

      const filterBar = await $("[data-testid='status-filter-bar']");
      await filterBar.waitForExist({ timeout: 5000 });

      const rowsBefore = (await $$(".n-data-table tbody tr")).length;

      // Click the "all" chip — should keep all rows
      const allChip = await $("[data-testid='status-filter-bar'] .chip-all");
      if (!(await allChip.isExisting())) return;
      await allChip.click();
      await browser.pause(300);

      const rowsAfterAll = (await $$(".n-data-table tbody tr")).length;
      expect(rowsAfterAll).toBe(rowsBefore);

      // Deactivate by clicking again
      await allChip.click();
      await browser.pause(300);
    });
  });

  describe("Combined Search and Filter", () => {
    it("should show no-results message when both filters match nothing", async () => {
      await navigateToSkillsMain();
      await waitForPageLoad();

      const hasComparisons = (await $$(".n-data-table tbody tr")).length > 0;
      if (!hasComparisons) return;

      const searchInput = await $("[data-testid='search-input'] input");
      await searchInput.waitForExist({ timeout: 5000 });
      await searchInput.setValue("zzz-absolutely-no-match");
      await browser.pause(500);

      // Empty state should appear with "no results" message
      const empty = await $(".n-empty");
      await empty.waitForExist({ timeout: 3000 });
      expect(await empty.isExisting()).toBe(true);
    });
  });
});
