/**
 * Type definitions used across fixtures and tests.
 */

/** Represents a test user that has been registered via the API */
export interface TestUser {
  id: string;
  email: string;
  password: string;
  first_name: string;
  last_name: string;
}

/** Represents a test tenant that has been created via the API */
export interface TestTenant {
  id: string;
  name: string;
  subdomain: string;
}

/**
 * Represents a fully authenticated test context:
 * a user who is logged in and associated with a specific tenant.
 */
export interface TestAuthContext {
  user: TestUser;
  tenant: TestTenant;
  accessToken: string;
  refreshToken: string;
}
