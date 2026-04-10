import { Page, Locator, expect } from '@playwright/test';
import { waitForLoadingComplete } from '../utils/test-helpers';
import type { TestPersonData } from '../data/test-persons';

/**
 * Page Object for person CRUD operations.
 * Covers all person types: internal, customer, vendor, distributor.
 * Maps to: PersonFormView.tsx, PersonCard.tsx, and the various list components.
 */
export class PersonsPage {
  readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  // ---------------------------------------------------------------------------
  // List view locators
  // ---------------------------------------------------------------------------

  /** The "Add {type}" button (e.g., "Add Customer", "Add Vendor") */
  addButton(type: string): Locator {
    return this.page.getByRole('button', { name: new RegExp(`add ${type}`, 'i') });
  }

  /** Refresh button */
  get refreshButton(): Locator {
    return this.page.getByRole('button', { name: /refresh/i });
  }

  /** Empty state message */
  get emptyState(): Locator {
    return this.page.getByText(/no .* found/i);
  }

  // ---------------------------------------------------------------------------
  // Form view locators
  // ---------------------------------------------------------------------------

  get nameInput(): Locator {
    return this.page.locator('#name');
  }

  get emailInput(): Locator {
    return this.page.locator('#email');
  }

  get phoneInput(): Locator {
    return this.page.locator('#phone');
  }

  get companyInput(): Locator {
    return this.page.locator('#company');
  }

  get industryInput(): Locator {
    return this.page.locator('#industry');
  }

  get departmentInput(): Locator {
    return this.page.locator('#department');
  }

  get positionInput(): Locator {
    return this.page.locator('#position');
  }

  get employeeIdInput(): Locator {
    return this.page.locator('#employee_id');
  }

  get serviceTypeInput(): Locator {
    return this.page.locator('#service_type');
  }

  get territoryInput(): Locator {
    return this.page.locator('#territory');
  }

  get commissionRateInput(): Locator {
    return this.page.locator('#commission_rate');
  }

  /** The "Create" or "Save Changes" submit button */
  get submitButton(): Locator {
    return this.page.getByRole('button', { name: /^create$|save changes/i });
  }

  /** Back button from form view */
  get backButton(): Locator {
    return this.page.getByRole('button', { name: /back/i }).first();
  }

  /** Cancel button in form view */
  get cancelButton(): Locator {
    return this.page.getByRole('button', { name: /cancel/i });
  }

  // ---------------------------------------------------------------------------
  // Actions
  // ---------------------------------------------------------------------------

  /** Click "Add {type}" to open the create form */
  async openCreateForm(personType: string): Promise<void> {
    await this.addButton(personType).click();
    await this.nameInput.waitFor({ state: 'visible' });
  }

  /** Fill the person form with the provided data */
  async fillPersonForm(data: TestPersonData): Promise<void> {
    await this.nameInput.fill(data.name);
    await this.emailInput.fill(data.email);
    if (data.phone) await this.phoneInput.fill(data.phone);

    // Type-specific fields
    switch (data.person_type) {
      case 'internal':
        if (data.department) await this.departmentInput.fill(data.department);
        if (data.position) await this.positionInput.fill(data.position);
        if (data.employee_id) await this.employeeIdInput.fill(data.employee_id);
        break;
      case 'customer':
        if (data.company) await this.companyInput.fill(data.company);
        if (data.industry) await this.industryInput.fill(data.industry);
        break;
      case 'vendor':
        if (data.company) await this.companyInput.fill(data.company);
        if (data.service_type) await this.serviceTypeInput.fill(data.service_type);
        break;
      case 'distributor':
        if (data.company) await this.companyInput.fill(data.company);
        if (data.territory) await this.territoryInput.fill(data.territory);
        if (data.commission_rate) await this.commissionRateInput.fill(data.commission_rate);
        break;
    }
  }

  /** Submit the person form */
  async submitForm(): Promise<void> {
    await this.submitButton.click();
    await waitForLoadingComplete(this.page);
  }

  /** Create a person: open form, fill data, submit */
  async createPerson(personType: string, data: TestPersonData): Promise<void> {
    await this.openCreateForm(personType);
    await this.fillPersonForm(data);
    await this.submitForm();
  }

  /** Assert that a person with the given name exists in the list */
  async expectPersonInList(name: string): Promise<void> {
    await expect(this.page.getByText(name).first()).toBeVisible({ timeout: 10_000 });
  }

  /** Assert that a person with the given name does NOT exist in the list */
  async expectPersonNotInList(name: string): Promise<void> {
    await expect(this.page.getByText(name)).not.toBeVisible({ timeout: 5_000 });
  }

  /** Click the view button on a person card */
  async viewPerson(name: string): Promise<void> {
    const card = this.page.locator(`text=${name}`).first().locator('..');
    await card.locator('button', { hasText: /view/i }).or(card.locator('[title="View"]')).click();
    await waitForLoadingComplete(this.page);
  }

  /** Click the edit button on a person card */
  async editPerson(name: string): Promise<void> {
    const card = this.page.locator(`text=${name}`).first().locator('..').locator('..');
    await card.getByRole('button', { name: /edit/i }).click();
    await waitForLoadingComplete(this.page);
  }

  /** Click the delete button on a person card */
  async deletePerson(name: string): Promise<void> {
    const card = this.page.locator(`text=${name}`).first().locator('..').locator('..');
    await card.getByRole('button', { name: /delete/i }).click();
  }

  /** Confirm deletion in the delete confirmation dialog */
  async confirmDelete(): Promise<void> {
    await this.page.getByRole('button', { name: /delete|confirm/i }).last().click();
    await waitForLoadingComplete(this.page);
  }

  /** Go back from the form view to the list view */
  async goBack(): Promise<void> {
    await this.backButton.click();
    await waitForLoadingComplete(this.page);
  }
}

export default PersonsPage;
