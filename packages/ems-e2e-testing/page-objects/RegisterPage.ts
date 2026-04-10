import { Page, Locator, expect } from '@playwright/test';

/**
 * Page Object for the Registration screen.
 * Maps to: components/auth/Register.tsx
 */
export class RegisterPage {
  readonly page: Page;
  readonly firstNameInput: Locator;
  readonly lastNameInput: Locator;
  readonly emailInput: Locator;
  readonly passwordInput: Locator;
  readonly confirmPasswordInput: Locator;
  readonly submitButton: Locator;
  readonly switchToLoginLink: Locator;
  readonly errorAlert: Locator;

  constructor(page: Page) {
    this.page = page;
    this.firstNameInput = page.locator('#first_name');
    this.lastNameInput = page.locator('#last_name');
    this.emailInput = page.locator('#email');
    this.passwordInput = page.locator('#password');
    this.confirmPasswordInput = page.locator('#confirmPassword');
    this.submitButton = page.getByRole('button', { name: /create account/i });
    this.switchToLoginLink = page.getByRole('button', { name: /sign in/i });
    this.errorAlert = page.locator('.bg-red-100');
  }

  /** Fill all registration form fields */
  async fillForm(data: {
    first_name: string;
    last_name: string;
    email: string;
    password: string;
    confirmPassword?: string;
  }): Promise<void> {
    await this.firstNameInput.fill(data.first_name);
    await this.lastNameInput.fill(data.last_name);
    await this.emailInput.fill(data.email);
    await this.passwordInput.fill(data.password);
    await this.confirmPasswordInput.fill(
      data.confirmPassword ?? data.password,
    );
  }

  /** Click the Create Account button */
  async submit(): Promise<void> {
    await this.submitButton.click();
  }

  /** Fill the form and submit in one call */
  async register(data: {
    first_name: string;
    last_name: string;
    email: string;
    password: string;
  }): Promise<void> {
    await this.fillForm(data);
    await this.submit();
  }

  /** Assert that an error message is displayed */
  async expectError(message: string): Promise<void> {
    await expect(this.errorAlert).toBeVisible();
    await expect(this.errorAlert).toContainText(message);
  }

  /** Assert that a specific field validation error is visible */
  async expectFieldError(message: string): Promise<void> {
    const errorText = this.page.locator(`text=${message}`);
    await expect(errorText).toBeVisible();
  }

  /** Switch to the login form */
  async switchToLogin(): Promise<void> {
    await this.switchToLoginLink.click();
  }

  /** Assert the register form is visible */
  async expectVisible(): Promise<void> {
    await expect(this.firstNameInput).toBeVisible();
    await expect(this.lastNameInput).toBeVisible();
    await expect(this.emailInput).toBeVisible();
    await expect(this.submitButton).toBeVisible();
  }
}

export default RegisterPage;
