import { test, expect } from '../../fixtures/auth.fixture';
import { DashboardPage } from '../../page-objects/DashboardPage';
import { PersonsPage } from '../../page-objects/PersonsPage';
import { createVendorPersonData } from '../../data/test-persons';

test.describe('Vendor CRUD @regression', () => {
  let dashboard: DashboardPage;
  let personsPage: PersonsPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    dashboard = new DashboardPage(authenticatedPage);
    personsPage = new PersonsPage(authenticatedPage);

    await dashboard.expectLoaded();

    // Navigate to Purchase section, then Vendors tab
    await dashboard.navigateToSection('Purchase');
    await dashboard.clickTab('Vendors');
  });

  test('should display the vendor list', async ({ authenticatedPage }) => {
    await expect(
      authenticatedPage.getByText(/vendors/i).first(),
    ).toBeVisible();
  });

  test('should create a new vendor', async () => {
    const vendorData = createVendorPersonData();

    await personsPage.createPerson('Vendor', vendorData);

    await personsPage.expectPersonInList(vendorData.name);
  });

  test('should validate vendor required fields', async ({ authenticatedPage }) => {
    // Open create form
    await personsPage.openCreateForm('Vendor');

    // Fill only name and email (missing company which is required for vendors)
    await personsPage.nameInput.fill('Test Vendor');
    await personsPage.emailInput.fill('vendor@test.ems.local');

    // Submit — should show validation error
    await personsPage.submitForm();

    // The form should still be visible (company is required)
    await expect(personsPage.nameInput).toBeVisible();
  });

  test('should edit a vendor', async () => {
    const vendorData = createVendorPersonData();
    await personsPage.createPerson('Vendor', vendorData);

    // Edit vendor
    await personsPage.editPerson(vendorData.name);

    const newServiceType = 'Cloud Services';
    await personsPage.serviceTypeInput.fill(newServiceType);
    await personsPage.submitForm();

    await personsPage.expectPersonInList(vendorData.name);
  });

  test('should delete a vendor', async () => {
    const vendorData = createVendorPersonData();
    await personsPage.createPerson('Vendor', vendorData);

    await personsPage.expectPersonInList(vendorData.name);

    await personsPage.deletePerson(vendorData.name);
    await personsPage.confirmDelete();

    await personsPage.expectPersonNotInList(vendorData.name);
  });
});
