import { Page, Locator, expect } from '@playwright/test';

/**
 * Page Object for the Login screen.
 * Maps to: components/auth/Login.tsx
 */
export class LoginPage {
  readonly page: Page;
  readonly emailInput: Locator;
  readonly passwordInput: Locator;
  readonly submitButton: Locator;
  readonly switchToRegisterLink: Locator;
  readonly errorAlert: Locator;

  constructor(page: Page) {
    this.page = page;
    this.emailInput = page.locator('#email');
    this.passwordInput = page.locator('#password');
    this.submitButton = page.getByRole('button', { name: /sign in/i });
    this.switchToRegisterLink = page.getByRole('button', { name: /sign up/i });
    this.errorAlert = page.locator('.bg-red-100');
  }

  /** Navigate to the app root (which shows login for unauthenticated users) */
  async goto(): Promise<void> {
    await this.page.goto('/');
    await this.emailInput.waitFor({ state: 'visible', timeout: 15_000 });
  }

  /** Fill email and password fields */
  async fillCredentials(email: string, password: string): Promise<void> {
    await this.emailInput.fill(email);
    await this.passwordInput.fill(password);
  }

  /** Click the Sign In button */
  async submit(): Promise<void> {
    await this.submitButton.click();
  }

  /** Complete login: fill credentials and submit */
  async login(email: string, password: string): Promise<void> {
    await this.fillCredentials(email, password);
    await this.submit();
  }

  /** Assert that an error message is displayed */
  async expectError(message: string): Promise<void> {
    await expect(this.errorAlert).toBeVisible();
    await expect(this.errorAlert).toContainText(message);
  }

  /** Assert that a field validation error is displayed */
  async expectFieldError(fieldName: string, message: string): Promise<void> {
    const errorText = this.page.locator(`text=${message}`);
    await expect(errorText).toBeVisible();
  }

  /** Switch to the registration form */
  async switchToRegister(): Promise<void> {
    await this.switchToRegisterLink.click();
  }

  /** Assert the login form is visible */
  async expectVisible(): Promise<void> {
    await expect(this.emailInput).toBeVisible();
    await expect(this.passwordInput).toBeVisible();
    await expect(this.submitButton).toBeVisible();
  }
}

export default LoginPage;
