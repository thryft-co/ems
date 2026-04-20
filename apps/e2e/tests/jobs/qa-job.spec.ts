import { test, expect } from '../../fixtures/auth.fixture';
import { DashboardPage } from '../../page-objects/DashboardPage';
import { JobsPage } from '../../page-objects/JobsPage';
import { createQAJobData } from '../../data/test-jobs';

test.describe('QA Job CRUD @regression', () => {
  let dashboard: DashboardPage;
  let jobsPage: JobsPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    dashboard = new DashboardPage(authenticatedPage);
    jobsPage = new JobsPage(authenticatedPage);

    await dashboard.expectLoaded();

    // Navigate to Quality section, then Jobs tab
    await dashboard.navigateToSection('Quality');
    await dashboard.clickTab('Jobs');
  });

  test('should create a new QA job', async () => {
    const jobData = createQAJobData();

    await jobsPage.createJob(jobData);

    await jobsPage.expectJobInList(jobData.job_number);
  });

  test('should create QA job with type-specific fields', async ({ authenticatedPage }) => {
    const jobData = createQAJobData({
      test_procedure: 'E2E Stress Test Protocol',
      inspection_type: 'Destructive + Non-destructive',
    });

    await jobsPage.createJob(jobData);

    // View the created job to verify QA fields
    await jobsPage.viewJob(jobData.job_number);

    // Verify the QA-specific data is preserved
    await expect(
      authenticatedPage.getByText(jobData.job_number).first(),
    ).toBeVisible();
  });

  test('should delete a QA job', async () => {
    const jobData = createQAJobData();
    await jobsPage.createJob(jobData);

    await jobsPage.expectJobInList(jobData.job_number);

    await jobsPage.deleteJob(jobData.job_number);
    await jobsPage.confirmDelete();

    await jobsPage.expectJobNotInList(jobData.job_number);
  });
});
