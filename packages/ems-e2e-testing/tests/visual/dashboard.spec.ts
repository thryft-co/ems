import { test, expect } from '../../fixtures/auth.fixture';
import { DashboardPage } from '../../page-objects/DashboardPage';

test.describe('Visual Regression @visual', () => {
  test('dashboard should match screenshot after login', async ({ authenticatedPage }) => {
    const dashboard = new DashboardPage(authenticatedPage);
    await dashboard.expectLoaded();

    // Wait a bit for all async content to settle
    await authenticatedPage.waitForTimeout(2000);

    // Take a screenshot and compare against baseline
    await expect(authenticatedPage).toHaveScreenshot('dashboard-main.png', {
      maxDiffPixelRatio: 0.05,
      fullPage: true,
    });
  });

  test('sidebar navigation sections should match screenshot', async ({
    authenticatedPage,
  }) => {
    const dashboard = new DashboardPage(authenticatedPage);
    await dashboard.expectLoaded();

    // Capture the sidebar area
    const sidebar = authenticatedPage.locator('nav, aside').first();
    if (await sidebar.isVisible()) {
      await expect(sidebar).toHaveScreenshot('sidebar-navigation.png', {
        maxDiffPixelRatio: 0.05,
      });
    }
  });
});
