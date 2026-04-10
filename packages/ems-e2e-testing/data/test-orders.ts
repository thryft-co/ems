import {
  generateUniqueId,
  generateUUID,
  getTodayDate,
  getFutureDate,
} from '../utils/test-helpers';

export interface TestOrderData {
  order_number: string;
  order_type: 'customer_order' | 'distributor_order' | 'purchase_order';
  external_entity_id: string;
  external_entity_type: 'customer' | 'distributor' | 'vendor';
  order_date: string;
  total_amount: number;
  status: 'draft' | 'submitted' | 'approved' | 'fulfilled' | 'cancelled' | 'paid';
  created_by_id: string;
  notes?: string;
  // Customer Order
  customer_reference?: string;
  shipping_address?: string;
  billing_address?: string;
  promised_delivery_date?: string;
  payment_method?: string;
  shipping_method?: string;
  discount_amount?: number;
  // Purchase Order
  vendor_reference?: string;
  expected_delivery_date?: string;
  payment_terms?: string;
  shipping_terms?: string;
  // Distributor Order
  territory?: string;
  commission_rate?: number;
  target_resale_amount?: number;
  agreement_reference?: string;
}

/** Create test data for a customer order */
export function createCustomerOrderData(
  createdById: string,
  externalEntityId?: string,
  overrides?: Partial<TestOrderData>,
): TestOrderData {
  return {
    order_number: generateUniqueId('CO'),
    order_type: 'customer_order',
    external_entity_id: externalEntityId || generateUUID(),
    external_entity_type: 'customer',
    order_date: getTodayDate(),
    total_amount: 2500.0,
    status: 'draft',
    created_by_id: createdById,
    notes: 'E2E test customer order',
    customer_reference: generateUniqueId('CREF'),
    shipping_address: '123 Test Street, Test City, TC 12345',
    billing_address: '123 Test Street, Test City, TC 12345',
    promised_delivery_date: getFutureDate(14),
    payment_method: 'Credit Card',
    shipping_method: 'Standard',
    discount_amount: 0,
    ...overrides,
  };
}

/** Create test data for a purchase order */
export function createPurchaseOrderData(
  createdById: string,
  externalEntityId?: string,
  overrides?: Partial<TestOrderData>,
): TestOrderData {
  return {
    order_number: generateUniqueId('PO'),
    order_type: 'purchase_order',
    external_entity_id: externalEntityId || generateUUID(),
    external_entity_type: 'vendor',
    order_date: getTodayDate(),
    total_amount: 15000.0,
    status: 'draft',
    created_by_id: createdById,
    notes: 'E2E test purchase order',
    vendor_reference: generateUniqueId('VREF'),
    expected_delivery_date: getFutureDate(21),
    payment_terms: 'Net 30',
    shipping_terms: 'FOB Origin',
    ...overrides,
  };
}

/** Create test data for a distributor order */
export function createDistributorOrderData(
  createdById: string,
  externalEntityId?: string,
  overrides?: Partial<TestOrderData>,
): TestOrderData {
  return {
    order_number: generateUniqueId('DO'),
    order_type: 'distributor_order',
    external_entity_id: externalEntityId || generateUUID(),
    external_entity_type: 'distributor',
    order_date: getTodayDate(),
    total_amount: 50000.0,
    status: 'draft',
    created_by_id: createdById,
    notes: 'E2E test distributor order',
    territory: 'North America',
    commission_rate: 8.5,
    target_resale_amount: 75000.0,
    agreement_reference: generateUniqueId('AGMT'),
    ...overrides,
  };
}
