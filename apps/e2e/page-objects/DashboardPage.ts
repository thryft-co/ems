import { Page, Locator, expect } from '@playwright/test';
import { waitForLoadingComplete } from '../utils/test-helpers';

/**
 * Page Object for the main Dashboard view.
 * Maps to: the <Dashboard /> component in App.tsx.
 *
 * The EMS app is a single-page app without URL routing —
 * all navigation is state-based via sidebar clicks and tab switches.
 */
export class DashboardPage {
  readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  // ---------------------------------------------------------------------------
  // Header locators
  // ---------------------------------------------------------------------------

  /** The main header title */
  get headerTitle(): Locator {
    return this.page.getByText('Enterprise Management Suite');
  }

  /** The current tenant name displayed in the header */
  get tenantBadge(): Locator {
    // The tenant name is inside a button with Building2 icon
    return this.page.locator('header button').filter({ has: this.page.locator('svg') }).first();
  }

  /** User name display in the header */
  get userBadge(): Locator {
    return this.page.locator('header button').filter({ has: this.page.locator('svg') }).nth(1);
  }

  /** Sign Out button in the header */
  get signOutButton(): Locator {
    return this.page.getByRole('button', { name: /sign out/i });
  }

  // ---------------------------------------------------------------------------
  // Sidebar navigation
  // ---------------------------------------------------------------------------

  /** Click a sidebar navigation item by its label text */
  async navigateToSection(label: string): Promise<void> {
    await this.page.getByRole('button', { name: label, exact: false }).click();
    await waitForLoadingComplete(this.page);
  }

  /** Click a tab button within the current section */
  async clickTab(tabLabel: string): Promise<void> {
    // Tab buttons are plain <button> elements with text matching the tab label
    await this.page
      .locator('button')
      .filter({ hasText: tabLabel })
      .first()
      .click();
    await waitForLoadingComplete(this.page);
  }

  // ---------------------------------------------------------------------------
  // Assertions
  // ---------------------------------------------------------------------------

  /** Wait for the dashboard to be fully loaded */
  async expectLoaded(): Promise<void> {
    await expect(this.headerTitle).toBeVisible({ timeout: 15_000 });
    await waitForLoadingComplete(this.page);
  }

  /** Assert the tenant name is displayed in the header */
  async expectTenantName(name: string): Promise<void> {
    await expect(this.page.getByText(name).first()).toBeVisible();
  }

  /** Assert the user name is displayed in the header */
  async expectUserName(firstName: string, lastName: string): Promise<void> {
    await expect(
      this.page.getByText(`${firstName} ${lastName}`).first(),
    ).toBeVisible();
  }

  /** Assert that a specific section title is visible in the content area */
  async expectSectionTitle(title: string): Promise<void> {
    await expect(this.page.getByText(title).first()).toBeVisible();
  }

  // ---------------------------------------------------------------------------
  // Actions
  // ---------------------------------------------------------------------------

  /** Logout from the dashboard */
  async logout(): Promise<void> {
    await this.signOutButton.click();
  }

  /** Navigate to the app root and wait for dashboard to load */
  async goto(): Promise<void> {
    await this.page.goto('/');
    await this.expectLoaded();
  }
}

export default DashboardPage;
