import { generateUniqueEmail, generateUniqueId } from '../utils/test-helpers';

const DEFAULT_PASSWORD = process.env.TEST_USER_PASSWORD || 'TestPassword123!';

export interface TestUserData {
  email: string;
  password: string;
  first_name: string;
  last_name: string;
}

export interface TestTenantData {
  name: string;
  subdomain: string;
}

/**
 * Create a unique test user data object.
 * Each call generates a fresh email to ensure test isolation.
 */
export function createTestUserData(overrides?: Partial<TestUserData>): TestUserData {
  return {
    email: generateUniqueEmail('user'),
    password: DEFAULT_PASSWORD,
    first_name: 'Test',
    last_name: 'User',
    ...overrides,
  };
}

/**
 * Create test tenant data with a unique subdomain.
 */
export function createTestTenantData(overrides?: Partial<TestTenantData>): TestTenantData {
  return {
    name: `Test Org ${generateUniqueId('org')}`,
    subdomain: generateUniqueId('org'),
    ...overrides,
  };
}

/**
 * Create paired user + tenant data for a complete test context.
 */
export function createTestContext(label: string = 'a') {
  return {
    user: createTestUserData({
      first_name: `Tenant${label.toUpperCase()}`,
      last_name: 'Admin',
    }),
    tenant: createTestTenantData({
      name: `Org ${label.toUpperCase()} ${generateUniqueId('')}`,
    }),
  };
}
