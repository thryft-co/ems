import { Page, Locator, expect } from '@playwright/test';
import { waitForLoadingComplete } from '../utils/test-helpers';
import type { TestJobData } from '../data/test-jobs';

/**
 * Page Object for Job CRUD operations.
 * Covers manufacturing, QA, and service jobs.
 * Maps to: JobFormView.tsx, JobCard.tsx, and the list components.
 */
export class JobsPage {
  readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  // ---------------------------------------------------------------------------
  // List view locators
  // ---------------------------------------------------------------------------

  get addJobButton(): Locator {
    return this.page.getByRole('button', { name: /add.*job|create.*job|new.*job/i });
  }

  get refreshButton(): Locator {
    return this.page.getByRole('button', { name: /refresh/i });
  }

  // ---------------------------------------------------------------------------
  // Form view locators
  // ---------------------------------------------------------------------------

  get jobNumberInput(): Locator {
    return this.page.locator('#job_number');
  }

  get itemIdInput(): Locator {
    return this.page.locator('#item_id');
  }

  get quantityInput(): Locator {
    return this.page.locator('#quantity');
  }

  get assignedUserIdInput(): Locator {
    return this.page.locator('#assigned_user_id');
  }

  get supervisorIdInput(): Locator {
    return this.page.locator('#supervisor_id');
  }

  get customerIdInput(): Locator {
    return this.page.locator('#customer_id');
  }

  get dueDateInput(): Locator {
    return this.page.locator('#due_date');
  }

  get commentsInput(): Locator {
    return this.page.locator('#comments');
  }

  // Manufacturing-specific
  get productionLineInput(): Locator {
    return this.page.locator('#production_line');
  }

  get batchNumberInput(): Locator {
    return this.page.locator('#batch_number');
  }

  // QA-specific
  get testProcedureInput(): Locator {
    return this.page.locator('#test_procedure');
  }

  get inspectionTypeInput(): Locator {
    return this.page.locator('#inspection_type');
  }

  // Service-specific
  get serviceTypeInput(): Locator {
    return this.page.locator('#service_type');
  }

  get problemDescriptionInput(): Locator {
    return this.page.locator('#problem_description');
  }

  // Action buttons
  get submitButton(): Locator {
    return this.page.getByRole('button', { name: /^create$|save/i });
  }

  get backButton(): Locator {
    return this.page.getByRole('button', { name: /back/i }).first();
  }

  // ---------------------------------------------------------------------------
  // Actions
  // ---------------------------------------------------------------------------

  /** Click the add/create job button to open the form */
  async openCreateForm(): Promise<void> {
    await this.addJobButton.click();
    await this.jobNumberInput.waitFor({ state: 'visible' });
  }

  /** Fill the job form with test data */
  async fillJobForm(data: TestJobData): Promise<void> {
    await this.jobNumberInput.fill(data.job_number);
    await this.itemIdInput.fill(data.item_id);
    await this.quantityInput.fill(String(data.quantity));

    if (data.due_date) await this.dueDateInput.fill(data.due_date);
    if (data.comments) await this.commentsInput.fill(data.comments);

    // Type-specific fields
    switch (data.job_type) {
      case 'manufacturing':
        if (data.production_line) await this.productionLineInput.fill(data.production_line);
        if (data.batch_number) await this.batchNumberInput.fill(data.batch_number);
        break;
      case 'qa':
        if (data.test_procedure) await this.testProcedureInput.fill(data.test_procedure);
        if (data.inspection_type) await this.inspectionTypeInput.fill(data.inspection_type);
        break;
      case 'service':
        if (data.service_type) await this.serviceTypeInput.fill(data.service_type);
        if (data.problem_description)
          await this.problemDescriptionInput.fill(data.problem_description);
        break;
    }
  }

  /** Submit the job form */
  async submitForm(): Promise<void> {
    await this.submitButton.click();
    await waitForLoadingComplete(this.page);
  }

  /** Create a job: open form, fill data, submit */
  async createJob(data: TestJobData): Promise<void> {
    await this.openCreateForm();
    await this.fillJobForm(data);
    await this.submitForm();
  }

  /** Assert that a job with the given job number exists in the list */
  async expectJobInList(jobNumber: string): Promise<void> {
    await expect(this.page.getByText(jobNumber).first()).toBeVisible({ timeout: 10_000 });
  }

  /** Assert that a job does NOT appear in the list */
  async expectJobNotInList(jobNumber: string): Promise<void> {
    await expect(this.page.getByText(jobNumber)).not.toBeVisible({ timeout: 5_000 });
  }

  /** Click the view button on a job card */
  async viewJob(jobNumber: string): Promise<void> {
    const card = this.page.locator(`text=${jobNumber}`).first().locator('..').locator('..');
    await card.getByRole('button', { name: /view/i }).click();
    await waitForLoadingComplete(this.page);
  }

  /** Click the delete button on a job card */
  async deleteJob(jobNumber: string): Promise<void> {
    const card = this.page.locator(`text=${jobNumber}`).first().locator('..').locator('..');
    await card.getByRole('button', { name: /delete/i }).click();
  }

  /** Confirm deletion in the confirmation dialog */
  async confirmDelete(): Promise<void> {
    await this.page.getByRole('button', { name: /delete|confirm/i }).last().click();
    await waitForLoadingComplete(this.page);
  }

  /** Go back from form view to list view */
  async goBack(): Promise<void> {
    await this.backButton.click();
    await waitForLoadingComplete(this.page);
  }
}

export default JobsPage;
