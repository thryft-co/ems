import { Page } from '@playwright/test';
import { randomUUID } from 'crypto';

/**
 * Generate a unique email address for test isolation.
 * Each test run gets a unique email to avoid conflicts.
 */
export function generateUniqueEmail(prefix: string = 'e2e'): string {
  const id = randomUUID().slice(0, 8);
  const timestamp = Date.now();
  return `${prefix}-${id}-${timestamp}@test.ems.local`;
}

/**
 * Generate a unique identifier with a given prefix.
 * Useful for tenant subdomains, job numbers, order numbers, etc.
 */
export function generateUniqueId(prefix: string = 'test'): string {
  const id = randomUUID().slice(0, 8);
  return `${prefix}-${id}`;
}

/**
 * Generate a valid UUID v4 string for use in form fields
 * that require UUID inputs (item_id, customer_id, etc.)
 */
export function generateUUID(): string {
  return randomUUID();
}

/**
 * Wait for all loading spinners to disappear from the page.
 * The EMS app uses `.animate-spin` class for loading indicators.
 */
export async function waitForLoadingComplete(page: Page): Promise<void> {
  // Wait for any spinning loaders to disappear
  await page.waitForSelector('.animate-spin', {
    state: 'hidden',
    timeout: 15_000,
  }).catch(() => {
    // No spinner found, which is fine
  });

  // Also wait for the main loading indicator with text "Loading..."
  await page.waitForSelector('text=Loading...', {
    state: 'hidden',
    timeout: 5_000,
  }).catch(() => {
    // No loading text found, which is fine
  });
}

/**
 * Wait for a success or error toast/alert to appear.
 */
export async function waitForAlert(
  page: Page,
  text: string,
  timeout: number = 10_000,
): Promise<void> {
  await page.getByText(text).waitFor({ state: 'visible', timeout });
}

/**
 * Set localStorage values to simulate an authenticated session.
 * This is used by fixtures to skip the login UI and go directly to the dashboard.
 */
export async function setAuthState(
  page: Page,
  data: {
    accessToken: string;
    refreshToken: string;
    user: {
      id: string;
      email: string;
      first_name: string;
      last_name: string;
      role: string;
    };
    tenant: {
      id: string;
      name: string;
      subdomain: string;
    };
  },
): Promise<void> {
  await page.addInitScript((authData) => {
    localStorage.setItem('accessToken', authData.accessToken);
    localStorage.setItem('refreshToken', authData.refreshToken);
    localStorage.setItem('user', JSON.stringify(authData.user));
    localStorage.setItem('currentTenant', JSON.stringify(authData.tenant));
    localStorage.setItem('tenantId', authData.tenant.id);
  }, data);
}

/**
 * Clear all auth-related localStorage entries.
 */
export async function clearAuthState(page: Page): Promise<void> {
  await page.addInitScript(() => {
    localStorage.removeItem('accessToken');
    localStorage.removeItem('refreshToken');
    localStorage.removeItem('user');
    localStorage.removeItem('currentTenant');
    localStorage.removeItem('tenantId');
  });
}

/**
 * Get a today's date string in YYYY-MM-DD format for form inputs.
 */
export function getTodayDate(): string {
  return new Date().toISOString().split('T')[0];
}

/**
 * Get a future date string in YYYY-MM-DD format.
 */
export function getFutureDate(daysFromNow: number = 30): string {
  const date = new Date();
  date.setDate(date.getDate() + daysFromNow);
  return date.toISOString().split('T')[0];
}
