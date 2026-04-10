import { test, expect } from '../../fixtures/auth.fixture';
import { DashboardPage } from '../../page-objects/DashboardPage';
import { OrdersPage } from '../../page-objects/OrdersPage';
import { createCustomerOrderData } from '../../data/test-orders';
import { generateUUID } from '../../utils/test-helpers';

test.describe('Customer Order CRUD @regression', () => {
  let dashboard: DashboardPage;
  let ordersPage: OrdersPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    dashboard = new DashboardPage(authenticatedPage);
    ordersPage = new OrdersPage(authenticatedPage);

    await dashboard.expectLoaded();

    // Navigate to Customers section, then Orders tab
    await dashboard.navigateToSection('Customers');
    await dashboard.clickTab('Orders');
  });

  test('should display the customer orders list', async ({ authenticatedPage }) => {
    await expect(
      authenticatedPage.getByText(/orders|customer/i).first(),
    ).toBeVisible();
  });

  test('should create a new customer order', async () => {
    const createdById = generateUUID();
    const orderData = createCustomerOrderData(createdById);

    await ordersPage.createOrder(orderData);

    await ordersPage.expectOrderInList(orderData.order_number);
  });

  test('should view customer order details', async ({ authenticatedPage }) => {
    const createdById = generateUUID();
    const orderData = createCustomerOrderData(createdById);
    await ordersPage.createOrder(orderData);

    // View the order
    await ordersPage.viewOrder(orderData.order_number);

    await expect(
      authenticatedPage.getByText(orderData.order_number).first(),
    ).toBeVisible();
  });

  test('should delete a customer order', async () => {
    const createdById = generateUUID();
    const orderData = createCustomerOrderData(createdById);
    await ordersPage.createOrder(orderData);

    await ordersPage.expectOrderInList(orderData.order_number);

    await ordersPage.deleteOrder(orderData.order_number);
    await ordersPage.confirmDelete();

    await ordersPage.expectOrderNotInList(orderData.order_number);
  });
});
