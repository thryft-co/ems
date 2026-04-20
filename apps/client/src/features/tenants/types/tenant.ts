// Tenant Interface
export interface Tenant {
  id: string;
  name: string;
  subdomain: string;
  database_url?: string;
  settings?: Record<string, any>;
  is_active?: boolean;
  created_at?: string;
  updated_at?: string;
}

// Create Tenant Request Interface
export interface CreateTenantRequest {
  name: string;
  subdomain: string;
  settings?: Record<string, any>;
}

// Update Tenant Request Interface
export interface UpdateTenantRequest {
  name?: string;
  settings?: Record<string, any>;
  is_active?: boolean;
}

// Tenant Context Interface
export interface TenantContext {
  tenant_id: string;
  tenant: Tenant;
}

// Tenant List Query Parameters
export interface TenantListQuery {
  limit?: number;
  offset?: number;
}
