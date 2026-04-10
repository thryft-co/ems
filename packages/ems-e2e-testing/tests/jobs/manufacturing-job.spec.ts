import { test, expect } from '../../fixtures/auth.fixture';
import { DashboardPage } from '../../page-objects/DashboardPage';
import { JobsPage } from '../../page-objects/JobsPage';
import { createManufacturingJobData } from '../../data/test-jobs';

test.describe('Manufacturing Job CRUD @regression', () => {
  let dashboard: DashboardPage;
  let jobsPage: JobsPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    dashboard = new DashboardPage(authenticatedPage);
    jobsPage = new JobsPage(authenticatedPage);

    await dashboard.expectLoaded();

    // Navigate to Manufacturing section, then Jobs tab
    await dashboard.navigateToSection('Manufacturing');
    await dashboard.clickTab('Jobs');
  });

  test('should display the manufacturing jobs list', async ({ authenticatedPage }) => {
    await expect(
      authenticatedPage.getByText(/manufacturing|jobs/i).first(),
    ).toBeVisible();
  });

  test('should create a new manufacturing job', async () => {
    const jobData = createManufacturingJobData();

    await jobsPage.createJob(jobData);

    // Should return to list with the new job visible
    await jobsPage.expectJobInList(jobData.job_number);
  });

  test('should view manufacturing job details', async ({ authenticatedPage }) => {
    const jobData = createManufacturingJobData();
    await jobsPage.createJob(jobData);

    // View the job
    await jobsPage.viewJob(jobData.job_number);

    // Should see job details
    await expect(
      authenticatedPage.getByText(jobData.job_number).first(),
    ).toBeVisible();
  });

  test('should delete a manufacturing job', async () => {
    const jobData = createManufacturingJobData();
    await jobsPage.createJob(jobData);

    await jobsPage.expectJobInList(jobData.job_number);

    await jobsPage.deleteJob(jobData.job_number);
    await jobsPage.confirmDelete();

    await jobsPage.expectJobNotInList(jobData.job_number);
  });

  test('should create a job via API and verify it appears in UI', async ({
    authenticatedPage,
    apiClient,
    tenantAContext,
  }) => {
    // Create a job via API
    const jobData = createManufacturingJobData();
    await apiClient.createJob(
      tenantAContext.accessToken,
      tenantAContext.tenant.id,
      jobData,
    );

    // Refresh the page to see the new data
    await authenticatedPage.reload();
    await dashboard.expectLoaded();
    await dashboard.navigateToSection('Manufacturing');
    await dashboard.clickTab('Jobs');

    // The API-created job should be visible in the UI
    await jobsPage.expectJobInList(jobData.job_number);
  });
});
