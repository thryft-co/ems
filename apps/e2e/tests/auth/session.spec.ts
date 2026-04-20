import { test, expect } from '../../fixtures/auth.fixture';
import { DashboardPage } from '../../page-objects/DashboardPage';
import { LoginPage } from '../../page-objects/LoginPage';

test.describe('Session Management @regression', () => {
  test('should persist session across page refresh', async ({ authenticatedPage }) => {
    const dashboardPage = new DashboardPage(authenticatedPage);

    // Dashboard should be loaded
    await dashboardPage.expectLoaded();

    // Refresh the page
    await authenticatedPage.reload();

    // Dashboard should still be loaded (tokens persist in localStorage)
    await dashboardPage.expectLoaded();
  });

  test('should clear session and redirect to login on sign out', async ({
    authenticatedPage,
  }) => {
    const dashboardPage = new DashboardPage(authenticatedPage);
    const loginPage = new LoginPage(authenticatedPage);

    await dashboardPage.expectLoaded();

    // Click sign out
    await dashboardPage.logout();

    // Should redirect to login page
    await loginPage.expectVisible();

    // LocalStorage should be cleared
    const accessToken = await authenticatedPage.evaluate(() =>
      localStorage.getItem('accessToken'),
    );
    expect(accessToken).toBeNull();
  });

  test('should redirect to login when tokens are removed', async ({ page }) => {
    const loginPage = new LoginPage(page);

    // Navigate without auth (no tokens in localStorage)
    await page.goto('/');

    // Should show login page
    await loginPage.expectVisible();
  });
});
