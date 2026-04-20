// Person types
export type PersonType = "internal" | "customer" | "vendor" | "distributor";

// Base Person Interface
export interface Person {
  id: string;
  uuid?: string;
  name: string;
  email: string;
  phone?: string;
  person_type: PersonType;
  global_access?: string[];
  is_active?: boolean;
  last_login?: string;
  created_at?: string;
  updated_at?: string;
}

// Internal Person Interface
export interface InternalPersonDetails {
  department: string;
  position: string;
  employee_id: string;
  hire_date?: string;
}

export interface InternalPerson extends Person {
  internal: InternalPersonDetails;
}

// Customer Person Interface
export interface CustomerPersonDetails {
  company?: string;
  industry?: string;
  customer_since?: string;
  account_manager_id?: string;
}

export interface CustomerPerson extends Person {
  customer: CustomerPersonDetails;
}

// Vendor Person Interface
export interface VendorPersonDetails {
  company: string;
  service_type?: string;
  contract_start?: string;
  contract_end?: string;
}

export interface VendorPerson extends Person {
  vendor: VendorPersonDetails;
}

// Distributor Person Interface
export interface DistributorPersonDetails {
  company: string;
  territory?: string;
  distribution_tier?: string;
  commission_rate?: string;
}

export interface DistributorPerson extends Person {
  distributor: DistributorPersonDetails;
}

// Person Form Interface for creating/updating persons
export interface PersonFormData {
  name: string;
  email: string;
  phone?: string;
  person_type: PersonType;
  role: string;
  global_access?: string; // JSON string
  is_active?: boolean;

  // Internal specific
  department?: string;
  position?: string;
  employee_id?: string;
  hire_date?: string;

  // Customer specific
  company?: string;
  industry?: string;
  customer_since?: string;
  account_manager_id?: string;

  // Vendor specific
  service_type?: string;
  contract_start?: string;
  contract_end?: string;

  // Distributor specific
  territory?: string;
  distribution_tier?: string;
  commission_rate?: string;
}

// API Response Interfaces
export interface PersonListResponse extends Array<Person> {}

export interface PersonDetailResponse extends Person {
  internal?: InternalPersonDetails;
  customer?: CustomerPersonDetails;
  vendor?: VendorPersonDetails;
  distributor?: DistributorPersonDetails;
}

export interface PersonCreateResponse {
  id: string;
}

export interface PersonUpdateResponse {
  id: string;
  message: string;
}
