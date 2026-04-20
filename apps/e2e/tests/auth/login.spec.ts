import { test, expect } from '@playwright/test';
import { LoginPage } from '../../page-objects/LoginPage';
import { ApiClient } from '../../utils/api-client';
import { createTestUserData, createTestTenantData } from '../../data/test-users';

test.describe('Login @smoke', () => {
  let loginPage: LoginPage;

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page);
    await loginPage.goto();
  });

  test('should display the login form', async () => {
    await loginPage.expectVisible();
  });

  test('should login successfully with valid credentials', async ({ page }) => {
    // Seed a user via API first
    const apiClient = new ApiClient();
    const userData = createTestUserData();
    const tenantData = createTestTenantData();

    await apiClient.setupUserWithTenant(
      userData.email,
      userData.password,
      userData.first_name,
      userData.last_name,
      tenantData.name,
      tenantData.subdomain,
    );

    // Now login via UI
    await loginPage.login(userData.email, userData.password);

    // Should navigate away from login (to tenant selection or dashboard)
    await expect(page.locator('#email')).not.toBeVisible({ timeout: 15_000 });
  });

  test('should show error for invalid email', async () => {
    await loginPage.login('nonexistent@test.ems.local', 'WrongPassword123!');

    // Wait for the error to appear (API returns 404 or 401)
    await expect(loginPage.errorAlert).toBeVisible({ timeout: 10_000 });
  });

  test('should show error for wrong password', async () => {
    // Seed a user
    const apiClient = new ApiClient();
    const userData = createTestUserData();
    const tenantData = createTestTenantData();

    await apiClient.setupUserWithTenant(
      userData.email,
      userData.password,
      userData.first_name,
      userData.last_name,
      tenantData.name,
      tenantData.subdomain,
    );

    // Login with wrong password
    await loginPage.login(userData.email, 'CompletelyWrongPassword!');

    await expect(loginPage.errorAlert).toBeVisible({ timeout: 10_000 });
  });

  test('should validate empty email field', async () => {
    await loginPage.fillCredentials('', 'SomePassword123!');
    await loginPage.submit();

    // The HTML5 required validation should prevent submission
    // or the app shows an error
    const emailInput = loginPage.emailInput;
    await expect(emailInput).toBeVisible();
  });

  test('should switch to register page', async () => {
    await loginPage.switchToRegister();

    // The register form should now be visible
    const firstNameInput = loginPage.page.locator('#first_name');
    await expect(firstNameInput).toBeVisible({ timeout: 5_000 });
  });
});
