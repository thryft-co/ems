import { test, expect } from '../../fixtures/auth.fixture';
import { DashboardPage } from '../../page-objects/DashboardPage';
import { OrdersPage } from '../../page-objects/OrdersPage';
import { createPurchaseOrderData } from '../../data/test-orders';
import { generateUUID } from '../../utils/test-helpers';

test.describe('Purchase Order CRUD @regression', () => {
  let dashboard: DashboardPage;
  let ordersPage: OrdersPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    dashboard = new DashboardPage(authenticatedPage);
    ordersPage = new OrdersPage(authenticatedPage);

    await dashboard.expectLoaded();

    // Navigate to Purchase section, then Orders tab
    await dashboard.navigateToSection('Purchase');
    await dashboard.clickTab('Orders');
  });

  test('should display the purchase orders list', async ({ authenticatedPage }) => {
    await expect(
      authenticatedPage.getByText(/orders|purchase/i).first(),
    ).toBeVisible();
  });

  test('should create a new purchase order', async () => {
    const createdById = generateUUID();
    const orderData = createPurchaseOrderData(createdById);

    await ordersPage.createOrder(orderData);

    await ordersPage.expectOrderInList(orderData.order_number);
  });

  test('should create purchase order with type-specific fields', async ({
    authenticatedPage,
  }) => {
    const createdById = generateUUID();
    const orderData = createPurchaseOrderData(createdById, undefined, {
      payment_terms: 'Net 60',
      shipping_terms: 'FOB Destination',
    });

    await ordersPage.createOrder(orderData);

    // View to verify purchase-specific fields
    await ordersPage.viewOrder(orderData.order_number);

    await expect(
      authenticatedPage.getByText(orderData.order_number).first(),
    ).toBeVisible();
  });

  test('should delete a purchase order', async () => {
    const createdById = generateUUID();
    const orderData = createPurchaseOrderData(createdById);
    await ordersPage.createOrder(orderData);

    await ordersPage.expectOrderInList(orderData.order_number);

    await ordersPage.deleteOrder(orderData.order_number);
    await ordersPage.confirmDelete();

    await ordersPage.expectOrderNotInList(orderData.order_number);
  });
});
