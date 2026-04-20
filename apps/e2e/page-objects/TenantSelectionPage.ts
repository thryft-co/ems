import { Page, Locator, expect } from '@playwright/test';

/**
 * Page Object for the Tenant Selection / Organization screen.
 * Maps to: components/auth/TenantSelection.tsx
 */
export class TenantSelectionPage {
  readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  // ---------------------------------------------------------------------------
  // Locators (dynamic because the page has two views: list vs create form)
  // ---------------------------------------------------------------------------

  /** "Create Organization" button on the tenant list view */
  get createOrgButton(): Locator {
    return this.page.getByRole('button', { name: /create.*organization/i });
  }

  /** Organization name input on the create form */
  get orgNameInput(): Locator {
    return this.page.locator('#name');
  }

  /** Subdomain input on the create form */
  get subdomainInput(): Locator {
    return this.page.locator('#subdomain');
  }

  /** Submit button on the create form (same text) */
  get createSubmitButton(): Locator {
    return this.page.getByRole('button', { name: /^create organization$/i });
  }

  /** "Continue to {tenantName}" button on the list view */
  get continueButton(): Locator {
    return this.page.getByRole('button', { name: /continue to|select an organization/i });
  }

  /** Sign Out button */
  get signOutButton(): Locator {
    return this.page.getByRole('button', { name: /sign out/i });
  }

  /** Back to org list button on the create form */
  get backToListButton(): Locator {
    return this.page.getByText('Back to organization list');
  }

  /** Error alert */
  get errorAlert(): Locator {
    return this.page.locator('.bg-red-100');
  }

  // ---------------------------------------------------------------------------
  // Actions
  // ---------------------------------------------------------------------------

  /** Wait for the tenant selection page to be visible */
  async expectVisible(): Promise<void> {
    // Either we see "Select Organization" or "Get Started"
    await this.page
      .getByText(/select organization|get started|no organizations/i)
      .first()
      .waitFor({ state: 'visible', timeout: 15_000 });
  }

  /** Click the "Create Organization" button to show the create form */
  async openCreateForm(): Promise<void> {
    await this.createOrgButton.first().click();
    await this.orgNameInput.waitFor({ state: 'visible' });
  }

  /** Fill and submit the create organization form */
  async createOrganization(name: string, subdomain: string): Promise<void> {
    await this.openCreateForm();
    await this.orgNameInput.fill(name);
    await this.subdomainInput.fill(subdomain);
    await this.createSubmitButton.click();
  }

  /** Select a tenant from the existing tenant list by name */
  async selectTenant(tenantName: string): Promise<void> {
    const tenantCard = this.page.locator(`text=${tenantName}`).first();
    await tenantCard.click();
  }

  /** Confirm the selected tenant and proceed */
  async confirmSelection(): Promise<void> {
    await this.continueButton.click();
  }

  /** Select a tenant and confirm in one call */
  async selectAndConfirmTenant(tenantName: string): Promise<void> {
    await this.selectTenant(tenantName);
    await this.continueButton.click();
  }

  /** Assert that the welcome message is visible with the user's first name */
  async expectWelcomeMessage(firstName: string): Promise<void> {
    await expect(
      this.page.getByText(`Welcome back, ${firstName}!`),
    ).toBeVisible();
  }

  /** Assert error message is displayed */
  async expectError(message: string): Promise<void> {
    await expect(this.errorAlert).toBeVisible();
    await expect(this.errorAlert).toContainText(message);
  }

  /** Sign out from the tenant selection screen */
  async signOut(): Promise<void> {
    await this.signOutButton.first().click();
  }
}

export default TenantSelectionPage;
