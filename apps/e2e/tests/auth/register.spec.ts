import { test, expect } from '@playwright/test';
import { LoginPage } from '../../page-objects/LoginPage';
import { RegisterPage } from '../../page-objects/RegisterPage';
import { TenantSelectionPage } from '../../page-objects/TenantSelectionPage';
import { ApiClient } from '../../utils/api-client';
import { createTestUserData } from '../../data/test-users';

test.describe('Registration @smoke', () => {
  let loginPage: LoginPage;
  let registerPage: RegisterPage;

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page);
    registerPage = new RegisterPage(page);

    await loginPage.goto();
    await loginPage.switchToRegister();
    await registerPage.expectVisible();
  });

  test('should display the registration form', async () => {
    await registerPage.expectVisible();
  });

  test('should register successfully and redirect to tenant selection', async ({ page }) => {
    const userData = createTestUserData();

    await registerPage.register(userData);

    // After registration, should see the tenant selection screen
    const tenantPage = new TenantSelectionPage(page);
    await tenantPage.expectVisible();
  });

  test('should show error for mismatched passwords', async () => {
    const userData = createTestUserData();

    await registerPage.fillForm({
      first_name: userData.first_name,
      last_name: userData.last_name,
      email: userData.email,
      password: userData.password,
      confirmPassword: 'DifferentPassword123!',
    });

    await registerPage.submit();

    // Should show password mismatch error
    await registerPage.expectFieldError('Passwords do not match');
  });

  test('should show error for duplicate email', async ({ page }) => {
    // First, register a user via API
    const apiClient = new ApiClient();
    const existingUser = createTestUserData();

    await apiClient.registerPerson({
      email: existingUser.email,
      first_name: existingUser.first_name,
      last_name: existingUser.last_name,
      password: existingUser.password,
    });

    // Try to register again with the same email via UI
    await registerPage.register(existingUser);

    // Should show duplicate email error
    await expect(registerPage.errorAlert).toBeVisible({ timeout: 10_000 });
  });

  test('should validate required fields', async () => {
    // Try to submit with empty fields
    await registerPage.submit();

    // The form should still be visible (not submitted)
    await registerPage.expectVisible();
  });

  test('should switch to login page', async ({ page }) => {
    await registerPage.switchToLogin();

    // Login form should now be visible
    const loginEmailInput = page.locator('#email');
    const loginPasswordInput = page.locator('#password');
    await expect(loginEmailInput).toBeVisible();
    await expect(loginPasswordInput).toBeVisible();
  });
});
