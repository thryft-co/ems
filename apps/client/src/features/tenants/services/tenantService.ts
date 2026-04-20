import axios, { AxiosError } from "axios";
import {
  Tenant,
  CreateTenantRequest,
  UpdateTenantRequest,
  TenantListQuery,
} from "@/features/tenants/types/tenant";

const API_URL = "/api/v1";

// Create an axios instance for tenant requests
const tenantAxios = axios.create({
  baseURL: API_URL,
});

// Set up axios interceptor to include authorization token
tenantAxios.interceptors.request.use((config) => {
  const token = localStorage.getItem("accessToken");
  if (token) {
    config.headers["Authorization"] = `Bearer ${token}`;
  }
  return config;
});

// Tenant Service Class
export class TenantService {
  // Get all tenants (with optional pagination)
  static async getAllTenants(query?: TenantListQuery): Promise<Tenant[]> {
    try {
      const params = new URLSearchParams();
      if (query?.limit) params.append("limit", query.limit.toString());
      if (query?.offset) params.append("offset", query.offset.toString());

      const response = await tenantAxios.get<Tenant[]>(
        `/tenants?${params.toString()}`,
      );
      return response.data;
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 401) {
        throw new Error("Unauthorized access");
      } else {
        throw new Error("Failed to fetch tenants");
      }
    }
  }

  // Get a specific tenant by ID
  static async getTenantById(tenantId: string): Promise<Tenant> {
    try {
      const response = await tenantAxios.get<Tenant>(`/tenants/${tenantId}`);
      return response.data;
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 404) {
        throw new Error("Tenant not found");
      } else if (axiosError.response?.status === 401) {
        throw new Error("Unauthorized access");
      } else {
        throw new Error("Failed to fetch tenant");
      }
    }
  }

  // Create a new tenant
  static async createTenant(tenantData: CreateTenantRequest): Promise<Tenant> {
    try {
      const response = await tenantAxios.post<Tenant>("/tenants", tenantData);
      return response.data;
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 400) {
        throw new Error("Invalid tenant data");
      } else if (axiosError.response?.status === 401) {
        throw new Error("Unauthorized access");
      } else if (axiosError.response?.status === 409) {
        throw new Error("Tenant subdomain already exists");
      } else {
        throw new Error("Failed to create tenant");
      }
    }
  }

  // Update an existing tenant
  static async updateTenant(
    tenantId: string,
    tenantData: UpdateTenantRequest,
  ): Promise<Tenant> {
    try {
      const response = await tenantAxios.put<Tenant>(
        `/tenants/${tenantId}`,
        tenantData,
      );
      return response.data;
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 404) {
        throw new Error("Tenant not found");
      } else if (axiosError.response?.status === 400) {
        throw new Error("Invalid tenant data");
      } else if (axiosError.response?.status === 401) {
        throw new Error("Unauthorized access");
      } else {
        throw new Error("Failed to update tenant");
      }
    }
  }

  // Delete a tenant
  static async deleteTenant(tenantId: string): Promise<void> {
    try {
      await tenantAxios.delete(`/tenants/${tenantId}`);
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 404) {
        throw new Error("Tenant not found");
      } else if (axiosError.response?.status === 401) {
        throw new Error("Unauthorized access");
      } else {
        throw new Error("Failed to delete tenant");
      }
    }
  }

  // Get tenants accessible to the current user
  static async getAccessibleTenants(): Promise<Tenant[]> {
    try {
      const response = await tenantAxios.get<Tenant[]>("/tenants/accessible");
      return response.data;
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 401) {
        throw new Error("Unauthorized access");
      } else {
        throw new Error("Failed to fetch accessible tenants");
      }
    }
  }
}

export default TenantService;
