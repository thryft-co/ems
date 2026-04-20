import { test, expect } from '../../fixtures/auth.fixture';
import { DashboardPage } from '../../page-objects/DashboardPage';
import { PersonsPage } from '../../page-objects/PersonsPage';
import { createCustomerPersonData } from '../../data/test-persons';

test.describe('Customer CRUD @regression', () => {
  let dashboard: DashboardPage;
  let personsPage: PersonsPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    dashboard = new DashboardPage(authenticatedPage);
    personsPage = new PersonsPage(authenticatedPage);

    // Wait for dashboard to load
    await dashboard.expectLoaded();

    // Navigate to Customers section via sidebar
    await dashboard.navigateToSection('Customers');
  });

  test('should display the customer list', async ({ authenticatedPage }) => {
    // The customer list view should be visible
    await expect(
      authenticatedPage.getByText(/customers/i).first(),
    ).toBeVisible();
  });

  test('should create a new customer', async () => {
    const customerData = createCustomerPersonData();

    await personsPage.createPerson('Customer', customerData);

    // Should return to list and the new customer should be visible
    await personsPage.expectPersonInList(customerData.name);
  });

  test('should view customer details', async ({ authenticatedPage }) => {
    // First, create a customer
    const customerData = createCustomerPersonData();
    await personsPage.createPerson('Customer', customerData);

    // Now click view on the customer
    await personsPage.viewPerson(customerData.name);

    // Should see the person details (View mode — form fields are disabled)
    await expect(authenticatedPage.getByText('View Customer').first()).toBeVisible({
      timeout: 10_000,
    });
  });

  test('should edit an existing customer', async ({ authenticatedPage }) => {
    // Create a customer first
    const customerData = createCustomerPersonData();
    await personsPage.createPerson('Customer', customerData);

    // Edit the customer
    await personsPage.editPerson(customerData.name);

    // Change the company name
    const newCompany = 'Updated Corp E2E';
    await personsPage.companyInput.fill(newCompany);
    await personsPage.submitForm();

    // Verify the update
    await personsPage.expectPersonInList(customerData.name);
  });

  test('should delete a customer', async () => {
    // Create a customer
    const customerData = createCustomerPersonData();
    await personsPage.createPerson('Customer', customerData);

    // Verify it exists
    await personsPage.expectPersonInList(customerData.name);

    // Delete the customer
    await personsPage.deletePerson(customerData.name);
    await personsPage.confirmDelete();

    // Verify it's gone
    await personsPage.expectPersonNotInList(customerData.name);
  });
});
