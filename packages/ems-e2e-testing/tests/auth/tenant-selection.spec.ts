import { test, expect } from '@playwright/test';
import { LoginPage } from '../../page-objects/LoginPage';
import { RegisterPage } from '../../page-objects/RegisterPage';
import { TenantSelectionPage } from '../../page-objects/TenantSelectionPage';
import { DashboardPage } from '../../page-objects/DashboardPage';
import { createTestUserData, createTestTenantData } from '../../data/test-users';

test.describe('Tenant Selection @smoke', () => {
  test('should create a new organization and land on dashboard', async ({ page }) => {
    const loginPage = new LoginPage(page);
    const registerPage = new RegisterPage(page);
    const tenantPage = new TenantSelectionPage(page);
    const dashboardPage = new DashboardPage(page);

    const userData = createTestUserData();
    const tenantData = createTestTenantData();

    // Register a new user
    await loginPage.goto();
    await loginPage.switchToRegister();
    await registerPage.register(userData);

    // Wait for tenant selection
    await tenantPage.expectVisible();

    // Create a new organization
    await tenantPage.createOrganization(tenantData.name, tenantData.subdomain);

    // Should land on the dashboard
    await dashboardPage.expectLoaded();
  });

  test('should show error for duplicate subdomain', async ({ page }) => {
    const loginPage = new LoginPage(page);
    const registerPage = new RegisterPage(page);
    const tenantPage = new TenantSelectionPage(page);

    const user1 = createTestUserData();
    const user2 = createTestUserData();
    const tenantData = createTestTenantData();

    // User 1: register and create org
    await loginPage.goto();
    await loginPage.switchToRegister();
    await registerPage.register(user1);
    await tenantPage.expectVisible();
    await tenantPage.createOrganization(tenantData.name, tenantData.subdomain);

    // Clear state and register user 2
    await page.context().clearCookies();
    await page.evaluate(() => localStorage.clear());

    await loginPage.goto();
    await loginPage.switchToRegister();
    await registerPage.register(user2);
    await tenantPage.expectVisible();

    // Try to create org with same subdomain
    await tenantPage.createOrganization('Another Org', tenantData.subdomain);

    // Should show error about duplicate subdomain
    await expect(tenantPage.errorAlert).toBeVisible({ timeout: 10_000 });
  });

  test('should validate organization name and subdomain', async ({ page }) => {
    const loginPage = new LoginPage(page);
    const registerPage = new RegisterPage(page);
    const tenantPage = new TenantSelectionPage(page);

    const userData = createTestUserData();

    await loginPage.goto();
    await loginPage.switchToRegister();
    await registerPage.register(userData);
    await tenantPage.expectVisible();

    // Open create form but try to submit with empty fields
    await tenantPage.openCreateForm();
    await tenantPage.createSubmitButton.click();

    // Form should still be visible (not submitted)
    await expect(tenantPage.orgNameInput).toBeVisible();
  });
});
