import { generateUniqueEmail, generateUniqueId } from '../utils/test-helpers';

export interface TestPersonData {
  name: string;
  email: string;
  phone?: string;
  person_type: 'internal' | 'customer' | 'vendor' | 'distributor';
  role: string;
  global_access?: string;
  is_active?: boolean;
  // Internal
  department?: string;
  position?: string;
  employee_id?: string;
  hire_date?: string;
  // Customer
  company?: string;
  industry?: string;
  customer_since?: string;
  // Vendor
  service_type?: string;
  contract_start?: string;
  contract_end?: string;
  // Distributor
  territory?: string;
  distribution_tier?: string;
  commission_rate?: string;
}

/** Create test data for an internal person */
export function createInternalPersonData(
  overrides?: Partial<TestPersonData>,
): TestPersonData {
  const id = generateUniqueId('emp');
  return {
    name: `Internal ${id}`,
    email: generateUniqueEmail('internal'),
    phone: '+1-555-0100',
    person_type: 'internal',
    role: 'employee',
    global_access: '["standard"]',
    is_active: true,
    department: 'Engineering',
    position: 'Software Engineer',
    employee_id: id,
    ...overrides,
  };
}

/** Create test data for a customer person */
export function createCustomerPersonData(
  overrides?: Partial<TestPersonData>,
): TestPersonData {
  const id = generateUniqueId('cust');
  return {
    name: `Customer ${id}`,
    email: generateUniqueEmail('customer'),
    phone: '+1-555-0200',
    person_type: 'customer',
    role: 'customer',
    global_access: '["standard"]',
    is_active: true,
    company: `${id} Corp`,
    industry: 'Technology',
    ...overrides,
  };
}

/** Create test data for a vendor person */
export function createVendorPersonData(
  overrides?: Partial<TestPersonData>,
): TestPersonData {
  const id = generateUniqueId('vend');
  return {
    name: `Vendor ${id}`,
    email: generateUniqueEmail('vendor'),
    phone: '+1-555-0300',
    person_type: 'vendor',
    role: 'vendor',
    global_access: '["standard"]',
    is_active: true,
    company: `${id} Supplies Inc`,
    service_type: 'Materials Supply',
    ...overrides,
  };
}

/** Create test data for a distributor person */
export function createDistributorPersonData(
  overrides?: Partial<TestPersonData>,
): TestPersonData {
  const id = generateUniqueId('dist');
  return {
    name: `Distributor ${id}`,
    email: generateUniqueEmail('distributor'),
    phone: '+1-555-0400',
    person_type: 'distributor',
    role: 'distributor',
    global_access: '["standard"]',
    is_active: true,
    company: `${id} Distribution LLC`,
    territory: 'North America',
    distribution_tier: 'tier1',
    commission_rate: '5.5',
    ...overrides,
  };
}
