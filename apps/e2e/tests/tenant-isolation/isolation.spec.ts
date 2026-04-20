import { test, expect } from '../../fixtures/auth.fixture';
import { createCustomerPersonData } from '../../data/test-persons';
import { createManufacturingJobData } from '../../data/test-jobs';
import { createCustomerOrderData } from '../../data/test-orders';

test.describe('Tenant Isolation @critical', () => {
  test('Tenant A person should not be visible to Tenant B (API)', async ({
    apiClient,
    tenantAContext,
    tenantBContext,
  }) => {
    // Create a person in Tenant A
    const personData = createCustomerPersonData();
    const createdPerson = await apiClient.createPerson(
      tenantAContext.accessToken,
      tenantAContext.tenant.id,
      personData,
    );
    expect(createdPerson.id).toBeTruthy();

    // Tenant A should see it
    const tenantAPersons = await apiClient.getPersonsByType(
      tenantAContext.accessToken,
      tenantAContext.tenant.id,
      'customer',
    );
    const foundInA = (tenantAPersons as { id: string }[]).some(
      (p) => p.id === createdPerson.id,
    );
    expect(foundInA).toBe(true);

    // Tenant B should NOT see it
    const tenantBPersons = await apiClient.getPersonsByType(
      tenantBContext.accessToken,
      tenantBContext.tenant.id,
      'customer',
    );
    const foundInB = (tenantBPersons as { id: string }[]).some(
      (p) => p.id === createdPerson.id,
    );
    expect(foundInB).toBe(false);
  });

  test('Tenant A job should not be visible to Tenant B (API)', async ({
    apiClient,
    tenantAContext,
    tenantBContext,
  }) => {
    // Create a job in Tenant A
    const jobData = createManufacturingJobData();
    const createdJob = await apiClient.createJob(
      tenantAContext.accessToken,
      tenantAContext.tenant.id,
      jobData,
    );
    expect(createdJob.id).toBeTruthy();

    // Tenant A should see it
    const tenantAJobs = await apiClient.getJobs(
      tenantAContext.accessToken,
      tenantAContext.tenant.id,
    );
    const foundInA = (tenantAJobs as { id: string }[]).some(
      (j) => j.id === createdJob.id,
    );
    expect(foundInA).toBe(true);

    // Tenant B should NOT see it
    const tenantBJobs = await apiClient.getJobs(
      tenantBContext.accessToken,
      tenantBContext.tenant.id,
    );
    const foundInB = (tenantBJobs as { id: string }[]).some(
      (j) => j.id === createdJob.id,
    );
    expect(foundInB).toBe(false);
  });

  test('Tenant A order should not be visible to Tenant B (API)', async ({
    apiClient,
    tenantAContext,
    tenantBContext,
  }) => {
    // Create an order in Tenant A
    const orderData = createCustomerOrderData(tenantAContext.user.id);
    const createdOrder = await apiClient.createOrder(
      tenantAContext.accessToken,
      tenantAContext.tenant.id,
      orderData,
    );
    expect(createdOrder.id).toBeTruthy();

    // Tenant A should see it
    const tenantAOrders = await apiClient.getOrders(
      tenantAContext.accessToken,
      tenantAContext.tenant.id,
      'customer_order',
    );
    const foundInA = (tenantAOrders as { id: string }[]).some(
      (o) => o.id === createdOrder.id,
    );
    expect(foundInA).toBe(true);

    // Tenant B should NOT see it
    const tenantBOrders = await apiClient.getOrders(
      tenantBContext.accessToken,
      tenantBContext.tenant.id,
      'customer_order',
    );
    const foundInB = (tenantBOrders as { id: string }[]).some(
      (o) => o.id === createdOrder.id,
    );
    expect(foundInB).toBe(false);
  });

  test('Cross-tenant API call with wrong tenant ID returns no data', async ({
    apiClient,
    tenantAContext,
    tenantBContext,
  }) => {
    // Create a person in Tenant A
    const personData = createCustomerPersonData();
    await apiClient.createPerson(
      tenantAContext.accessToken,
      tenantAContext.tenant.id,
      personData,
    );

    // Try to access Tenant A data with Tenant A's token but Tenant B's tenant ID
    // This should return empty results or 403 (depending on RLS implementation)
    try {
      const crossTenantResults = await apiClient.getPersonsByType(
        tenantAContext.accessToken,
        tenantBContext.tenant.id,
        'customer',
      );
      // If we get results, they should not contain Tenant A data
      const contaminated = (crossTenantResults as { name: string }[]).some(
        (p) => p.name === personData.name,
      );
      expect(contaminated).toBe(false);
    } catch (error: unknown) {
      // A 403 or 401 is also acceptable — it means the server correctly blocked
      const axiosError = error as { response?: { status: number } };
      expect([401, 403]).toContain(axiosError.response?.status);
    }
  });
});
