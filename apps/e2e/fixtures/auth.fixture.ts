import { test as base, Page, BrowserContext } from '@playwright/test';
import { ApiClient } from '../utils/api-client';
import { createTestUserData, createTestTenantData } from '../data/test-users';
import { setAuthState } from '../utils/test-helpers';
import type { TestAuthContext } from './types';

/**
 * Extended Playwright fixtures that provide:
 * - `apiClient`: Pre-configured API client
 * - `authenticatedPage`: A page already logged in (via localStorage injection)
 * - `tenantAContext` / `tenantBContext`: Two isolated auth contexts for tenant isolation tests
 */

// Define custom fixture types
type AuthFixtures = {
  /** Pre-configured API client for backend calls */
  apiClient: ApiClient;

  /** A page that is pre-authenticated with a fresh user+tenant */
  authenticatedPage: Page;

  /** Auth context for Tenant A (used in isolation tests) */
  tenantAContext: TestAuthContext;

  /** Auth context for Tenant B (used in isolation tests) */
  tenantBContext: TestAuthContext;
};

/**
 * Helper: Register a user, create a tenant, and return the full auth context.
 */
async function createAuthContext(apiClient: ApiClient): Promise<TestAuthContext> {
  const userData = createTestUserData();
  const tenantData = createTestTenantData();

  const authResponse = await apiClient.setupUserWithTenant(
    userData.email,
    userData.password,
    userData.first_name,
    userData.last_name,
    tenantData.name,
    tenantData.subdomain,
  );

  return {
    user: {
      id: authResponse.user.id,
      email: userData.email,
      password: userData.password,
      first_name: authResponse.user.first_name || userData.first_name,
      last_name: authResponse.user.last_name || userData.last_name,
    },
    tenant: {
      id: authResponse.tenant.id,
      name: authResponse.tenant.name,
      subdomain: authResponse.tenant.subdomain,
    },
    accessToken: authResponse.access_token,
    refreshToken: authResponse.refresh_token,
  };
}

export const test = base.extend<AuthFixtures>({
  // -------------------------------------------------------------------------
  // apiClient: shared API client instance
  // -------------------------------------------------------------------------
  apiClient: async ({}, use) => {
    const client = new ApiClient();
    await use(client);
  },

  // -------------------------------------------------------------------------
  // authenticatedPage: a page pre-loaded with auth tokens in localStorage
  // -------------------------------------------------------------------------
  authenticatedPage: async ({ page, apiClient }, use) => {
    // Create a fresh user+tenant for this test
    const authContext = await createAuthContext(apiClient);

    // Inject tokens into localStorage before navigating
    await setAuthState(page, {
      accessToken: authContext.accessToken,
      refreshToken: authContext.refreshToken,
      user: {
        id: authContext.user.id,
        email: authContext.user.email,
        first_name: authContext.user.first_name,
        last_name: authContext.user.last_name,
        role: 'admin',
      },
      tenant: {
        id: authContext.tenant.id,
        name: authContext.tenant.name,
        subdomain: authContext.tenant.subdomain,
      },
    });

    // Navigate to the app — should go directly to dashboard
    await page.goto('/');

    await use(page);
  },

  // -------------------------------------------------------------------------
  // tenantAContext: auth context for tenant A (for isolation tests)
  // -------------------------------------------------------------------------
  tenantAContext: async ({ apiClient }, use) => {
    const context = await createAuthContext(apiClient);
    await use(context);
  },

  // -------------------------------------------------------------------------
  // tenantBContext: auth context for tenant B (for isolation tests)
  // -------------------------------------------------------------------------
  tenantBContext: async ({ apiClient }, use) => {
    const context = await createAuthContext(apiClient);
    await use(context);
  },
});

export { expect } from '@playwright/test';
