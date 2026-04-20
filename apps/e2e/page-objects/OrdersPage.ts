import { Page, Locator, expect } from '@playwright/test';
import { waitForLoadingComplete } from '../utils/test-helpers';
import type { TestOrderData } from '../data/test-orders';

/**
 * Page Object for Order CRUD operations.
 * Covers customer orders, purchase orders, and distributor orders.
 * Maps to: OrderFormView.tsx, OrderCard.tsx, and the list components.
 */
export class OrdersPage {
  readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  // ---------------------------------------------------------------------------
  // List view locators
  // ---------------------------------------------------------------------------

  get addOrderButton(): Locator {
    return this.page.getByRole('button', { name: /add.*order|create.*order|new.*order/i });
  }

  get refreshButton(): Locator {
    return this.page.getByRole('button', { name: /refresh/i });
  }

  // ---------------------------------------------------------------------------
  // Form view locators
  // ---------------------------------------------------------------------------

  get orderNumberInput(): Locator {
    return this.page.locator('#order_number');
  }

  get externalEntityIdInput(): Locator {
    return this.page.locator('#external_entity_id');
  }

  get createdByIdInput(): Locator {
    return this.page.locator('#created_by_id');
  }

  get orderDateInput(): Locator {
    return this.page.locator('#order_date');
  }

  get totalAmountInput(): Locator {
    return this.page.locator('#total_amount');
  }

  get notesInput(): Locator {
    return this.page.locator('#notes');
  }

  // Customer order specific
  get customerReferenceInput(): Locator {
    return this.page.locator('#customer_reference');
  }

  get shippingAddressInput(): Locator {
    return this.page.locator('#shipping_address');
  }

  get paymentMethodInput(): Locator {
    return this.page.locator('#payment_method');
  }

  get shippingMethodInput(): Locator {
    return this.page.locator('#shipping_method');
  }

  // Purchase order specific
  get vendorReferenceInput(): Locator {
    return this.page.locator('#vendor_reference');
  }

  get paymentTermsInput(): Locator {
    return this.page.locator('#payment_terms');
  }

  get shippingTermsInput(): Locator {
    return this.page.locator('#shipping_terms');
  }

  // Distributor order specific
  get territoryInput(): Locator {
    return this.page.locator('#territory');
  }

  get commissionRateInput(): Locator {
    return this.page.locator('#commission_rate');
  }

  get agreementReferenceInput(): Locator {
    return this.page.locator('#agreement_reference');
  }

  // Action buttons
  get submitButton(): Locator {
    return this.page.getByRole('button', { name: /create order|update order/i });
  }

  get backButton(): Locator {
    return this.page.getByRole('button', { name: /back/i }).first();
  }

  get cancelButton(): Locator {
    return this.page.getByRole('button', { name: /cancel/i });
  }

  // ---------------------------------------------------------------------------
  // Actions
  // ---------------------------------------------------------------------------

  /** Click the add order button to open the create form */
  async openCreateForm(): Promise<void> {
    await this.addOrderButton.click();
    await this.orderNumberInput.waitFor({ state: 'visible' });
  }

  /** Fill the order form with test data */
  async fillOrderForm(data: TestOrderData): Promise<void> {
    await this.orderNumberInput.fill(data.order_number);
    await this.externalEntityIdInput.fill(data.external_entity_id);
    await this.createdByIdInput.fill(data.created_by_id);
    await this.totalAmountInput.fill(String(data.total_amount));
    if (data.notes) await this.notesInput.fill(data.notes);

    // Type-specific fields
    switch (data.order_type) {
      case 'customer_order':
        if (data.customer_reference)
          await this.customerReferenceInput.fill(data.customer_reference);
        if (data.shipping_address)
          await this.shippingAddressInput.fill(data.shipping_address);
        if (data.payment_method) await this.paymentMethodInput.fill(data.payment_method);
        if (data.shipping_method) await this.shippingMethodInput.fill(data.shipping_method);
        break;
      case 'purchase_order':
        if (data.vendor_reference) await this.vendorReferenceInput.fill(data.vendor_reference);
        if (data.payment_terms) await this.paymentTermsInput.fill(data.payment_terms);
        if (data.shipping_terms) await this.shippingTermsInput.fill(data.shipping_terms);
        break;
      case 'distributor_order':
        if (data.territory) await this.territoryInput.fill(data.territory);
        if (data.commission_rate !== undefined)
          await this.commissionRateInput.fill(String(data.commission_rate));
        if (data.agreement_reference)
          await this.agreementReferenceInput.fill(data.agreement_reference);
        break;
    }
  }

  /** Submit the order form */
  async submitForm(): Promise<void> {
    await this.submitButton.click();
    await waitForLoadingComplete(this.page);
  }

  /** Create an order: open form, fill data, submit */
  async createOrder(data: TestOrderData): Promise<void> {
    await this.openCreateForm();
    await this.fillOrderForm(data);
    await this.submitForm();
  }

  /** Assert that an order with the given order number is visible */
  async expectOrderInList(orderNumber: string): Promise<void> {
    await expect(this.page.getByText(orderNumber).first()).toBeVisible({ timeout: 10_000 });
  }

  /** Assert that an order does NOT appear in the list */
  async expectOrderNotInList(orderNumber: string): Promise<void> {
    await expect(this.page.getByText(orderNumber)).not.toBeVisible({ timeout: 5_000 });
  }

  /** Click view on an order card */
  async viewOrder(orderNumber: string): Promise<void> {
    const card = this.page.locator(`text=${orderNumber}`).first().locator('..').locator('..');
    await card.getByRole('button', { name: /view/i }).click();
    await waitForLoadingComplete(this.page);
  }

  /** Click delete on an order card */
  async deleteOrder(orderNumber: string): Promise<void> {
    const card = this.page.locator(`text=${orderNumber}`).first().locator('..').locator('..');
    await card.getByRole('button', { name: /delete/i }).click();
  }

  /** Confirm the delete dialog */
  async confirmDelete(): Promise<void> {
    await this.page.getByRole('button', { name: /delete|confirm/i }).last().click();
    await waitForLoadingComplete(this.page);
  }

  /** Go back from form view */
  async goBack(): Promise<void> {
    await this.backButton.click();
    await waitForLoadingComplete(this.page);
  }
}

export default OrdersPage;
