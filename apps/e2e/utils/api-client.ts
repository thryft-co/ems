import axios, { AxiosInstance, AxiosError } from 'axios';

/** Auth response from the EMS API */
interface AuthResponse {
  access_token: string;
  refresh_token: string;
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
}

/** Person-only auth response (no tenant) */
interface PersonOnlyAuthResponse {
  access_token: string;
  refresh_token: string;
  person: {
    id: string;
    email: string;
    first_name: string;
    last_name: string;
  };
}

/** Generic API response for resource creation */
interface CreateResponse {
  id: string;
}

/**
 * API client for E2E test data setup and teardown.
 * Uses the backend REST API directly (bypasses the UI) for speed and reliability.
 */
export class ApiClient {
  private client: AxiosInstance;

  constructor(baseURL?: string) {
    this.client = axios.create({
      baseURL: baseURL || process.env.API_URL || 'http://localhost:5002',
      timeout: 30_000,
      headers: {
        'Content-Type': 'application/json',
      },
    });
  }

  // ---------------------------------------------------------------------------
  // Auth endpoints
  // ---------------------------------------------------------------------------

  /** Register a new person (without creating a tenant) */
  async registerPerson(data: {
    email: string;
    first_name: string;
    last_name: string;
    password: string;
  }): Promise<PersonOnlyAuthResponse> {
    const response = await this.client.post<PersonOnlyAuthResponse>(
      '/api/v1/auth/person-register',
      data,
    );
    return response.data;
  }

  /** Login with email and password (requires existing tenant association) */
  async login(email: string, password: string): Promise<AuthResponse> {
    const response = await this.client.post<AuthResponse>(
      '/api/v1/auth/login',
      { email, password },
    );
    return response.data;
  }

  /** Create a new tenant and join it (requires a valid access token) */
  async createAndJoinTenant(
    accessToken: string,
    tenantName: string,
    tenantSubdomain: string,
  ): Promise<AuthResponse> {
    const response = await this.client.post<AuthResponse>(
      '/api/v1/auth/create-tenant',
      {
        tenant_name: tenantName,
        tenant_subdomain: tenantSubdomain,
      },
      {
        headers: { Authorization: `Bearer ${accessToken}` },
      },
    );
    return response.data;
  }

  /** Join an existing tenant by subdomain */
  async joinTenant(
    accessToken: string,
    tenantSubdomain: string,
  ): Promise<AuthResponse> {
    const response = await this.client.post<AuthResponse>(
      '/api/v1/auth/join-tenant',
      { tenant_subdomain: tenantSubdomain },
      {
        headers: { Authorization: `Bearer ${accessToken}` },
      },
    );
    return response.data;
  }

  /** Logout (blacklist tokens) */
  async logout(
    accessToken: string,
    refreshToken: string,
    tenantId: string,
  ): Promise<void> {
    await this.client.post(
      '/api/v1/auth/logout',
      { refresh_token: refreshToken },
      {
        headers: {
          Authorization: `Bearer ${accessToken}`,
          'X-Tenant-ID': tenantId,
        },
      },
    );
  }

  // ---------------------------------------------------------------------------
  // Person endpoints
  // ---------------------------------------------------------------------------

  /** Create a person within a tenant */
  async createPerson(
    accessToken: string,
    tenantId: string,
    data: Record<string, unknown>,
  ): Promise<CreateResponse> {
    const response = await this.client.post<CreateResponse>(
      '/api/v1/person',
      data,
      {
        headers: {
          Authorization: `Bearer ${accessToken}`,
          'X-Tenant-ID': tenantId,
        },
      },
    );
    return response.data;
  }

  /** Get all persons for a given type (internal, customer, vendor, distributor) */
  async getPersonsByType(
    accessToken: string,
    tenantId: string,
    personType: string,
  ): Promise<unknown[]> {
    const response = await this.client.get(`/api/v1/person/${personType}`, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'X-Tenant-ID': tenantId,
      },
    });
    return response.data;
  }

  /** Get all persons */
  async getAllPersons(
    accessToken: string,
    tenantId: string,
  ): Promise<unknown[]> {
    const response = await this.client.get('/api/v1/person', {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'X-Tenant-ID': tenantId,
      },
    });
    return response.data;
  }

  /** Delete a person by ID */
  async deletePerson(
    accessToken: string,
    tenantId: string,
    personId: string,
  ): Promise<void> {
    await this.client.delete(`/api/v1/person/${personId}`, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'X-Tenant-ID': tenantId,
      },
    });
  }

  // ---------------------------------------------------------------------------
  // Job endpoints
  // ---------------------------------------------------------------------------

  /** Create a job within a tenant */
  async createJob(
    accessToken: string,
    tenantId: string,
    data: Record<string, unknown>,
  ): Promise<CreateResponse> {
    const response = await this.client.post<CreateResponse>(
      '/api/v1/job',
      data,
      {
        headers: {
          Authorization: `Bearer ${accessToken}`,
          'X-Tenant-ID': tenantId,
        },
      },
    );
    return response.data;
  }

  /** Get all jobs (optionally filtered by type) */
  async getJobs(
    accessToken: string,
    tenantId: string,
    jobType?: string,
  ): Promise<unknown[]> {
    const url = jobType ? `/api/v1/job?type=${jobType}` : '/api/v1/job';
    const response = await this.client.get(url, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'X-Tenant-ID': tenantId,
      },
    });
    return response.data;
  }

  /** Delete a job by ID */
  async deleteJob(
    accessToken: string,
    tenantId: string,
    jobId: string,
  ): Promise<void> {
    await this.client.delete(`/api/v1/job/${jobId}`, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'X-Tenant-ID': tenantId,
      },
    });
  }

  // ---------------------------------------------------------------------------
  // Order endpoints
  // ---------------------------------------------------------------------------

  /** Create an order within a tenant */
  async createOrder(
    accessToken: string,
    tenantId: string,
    data: Record<string, unknown>,
  ): Promise<CreateResponse> {
    const response = await this.client.post<CreateResponse>(
      '/api/v1/order',
      data,
      {
        headers: {
          Authorization: `Bearer ${accessToken}`,
          'X-Tenant-ID': tenantId,
        },
      },
    );
    return response.data;
  }

  /** Get all orders (optionally filtered by type) */
  async getOrders(
    accessToken: string,
    tenantId: string,
    orderType?: string,
  ): Promise<unknown[]> {
    const url = orderType
      ? `/api/v1/order/${orderType}`
      : '/api/v1/order';
    const response = await this.client.get(url, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'X-Tenant-ID': tenantId,
      },
    });
    return response.data;
  }

  /** Delete an order by ID */
  async deleteOrder(
    accessToken: string,
    tenantId: string,
    orderId: string,
  ): Promise<void> {
    await this.client.delete(`/api/v1/order/${orderId}`, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'X-Tenant-ID': tenantId,
      },
    });
  }

  // ---------------------------------------------------------------------------
  // Health check
  // ---------------------------------------------------------------------------

  /** Check if the backend is healthy */
  async healthCheck(): Promise<boolean> {
    try {
      const response = await this.client.get('/health', { timeout: 5000 });
      return response.status === 200;
    } catch {
      return false;
    }
  }

  /**
   * Wait for the backend to become healthy, with retries.
   * @param maxAttempts Maximum number of retry attempts
   * @param delayMs Delay between retries in milliseconds
   */
  async waitForHealthy(
    maxAttempts: number = 30,
    delayMs: number = 2000,
  ): Promise<void> {
    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      const healthy = await this.healthCheck();
      if (healthy) {
        console.log(`✓ Backend is healthy (attempt ${attempt})`);
        return;
      }
      console.log(
        `⏳ Waiting for backend... (attempt ${attempt}/${maxAttempts})`,
      );
      await new Promise((resolve) => setTimeout(resolve, delayMs));
    }
    throw new Error(
      `Backend did not become healthy after ${maxAttempts} attempts`,
    );
  }

  // ---------------------------------------------------------------------------
  // Convenience: Full user+tenant setup
  // ---------------------------------------------------------------------------

  /**
   * Register a user and create a tenant in one call.
   * Returns the full auth response with tokens, user, and tenant info.
   */
  async setupUserWithTenant(
    email: string,
    password: string,
    firstName: string,
    lastName: string,
    tenantName: string,
    tenantSubdomain: string,
  ): Promise<AuthResponse> {
    // Step 1: Register person
    const registerResponse = await this.registerPerson({
      email,
      first_name: firstName,
      last_name: lastName,
      password,
    });

    // Step 2: Create and join tenant
    const authResponse = await this.createAndJoinTenant(
      registerResponse.access_token,
      tenantName,
      tenantSubdomain,
    );

    return authResponse;
  }

  /**
   * Extract a user-friendly error message from an Axios error.
   */
  static getErrorMessage(error: unknown): string {
    if (error instanceof AxiosError) {
      const status = error.response?.status;
      const data = error.response?.data;
      return `HTTP ${status}: ${JSON.stringify(data)}`;
    }
    return String(error);
  }
}

export default ApiClient;
