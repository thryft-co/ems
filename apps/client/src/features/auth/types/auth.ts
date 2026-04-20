// PersonRole enum - matching backend PersonRole
export enum PersonRole {
  Pending = "pending",
  Admin = "admin",
  Manager = "manager",
  Employee = "employee",
  Customer = "customer",
  Vendor = "vendor",
  Distributor = "distributor",
}

// Login Request Interface
export interface LoginRequest {
  email: string;
  password: string;
}

// Register Request Interface
export interface RegisterRequest {
  email: string;
  first_name: string;
  last_name: string;
  password: string;
}

// OAuth Provider enum
export enum OAuthProvider {
  Google = "google",
  Microsoft = "microsoft",
  Apple = "apple",
}

// OAuth Login Request Interface
export interface OAuthLoginRequest {
  provider: OAuthProvider;
  tenant_subdomain: string;
  redirect_url?: string;
}

// OAuth URL Response Interface
export interface OAuthUrlResponse {
  auth_url: string;
  state: string;
}

// OAuth Callback Request Interface
export interface OAuthCallbackRequest {
  provider: OAuthProvider;
  code: string;
  state: string;
  tenant_subdomain: string;
}

// Internal Person OAuth Register Request Interface
export interface InternalPersonOAuthRegisterRequest {
  provider: OAuthProvider;
  code: string;
  state: string;
  tenant_subdomain: string;
  tenant_name: string;
  department?: string;
  position?: string;
  employee_id?: string;
}

// Auth User Interface
export interface AuthUser {
  id: string;
  email: string;
  first_name: string;
  last_name: string;
  role: PersonRole;
}

// Auth Tenant Interface
export interface AuthTenant {
  id: string;
  name: string;
  subdomain: string;
}

// Auth Response Interface
export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  user: AuthUser;
  tenant: AuthTenant;
}

// Refresh Token Request Interface
export interface RefreshTokenRequest {
  refresh_token: string;
}

// Refresh Token Response Interface
export interface RefreshTokenResponse {
  access_token: string;
  refresh_token: string;
}

// Logout Request Interface
export interface LogoutRequest {
  refresh_token: string;
}

// JWT Claims Interface
export interface Claims {
  sub: string; // User ID
  tenant_id: string;
  role: string;
  exp: number; // Expiration time
  iat: number; // Issued at
}

// Auth Context State Interface
export interface AuthState {
  isAuthenticated: boolean;
  user: AuthUser | null;
  currentTenant: AuthTenant | null;
  accessToken: string | null;
  refreshToken: string | null;
  isLoading: boolean;
}

// Auth Context Actions Interface
export interface AuthContextType extends AuthState {
  login: (credentials: LoginRequest) => Promise<void>;
  register: (data: RegisterRequest) => Promise<void>;
  joinTenant: (tenantSubdomain: string) => Promise<void>;
  createAndJoinTenant: (
    tenantName: string,
    tenantSubdomain: string,
  ) => Promise<void>;
  logout: () => Promise<void>;
  selectTenant: (tenant: AuthTenant) => void;
  refreshAccessToken: () => Promise<void>;
  clearSession: () => void;
}

// Person-only registration (without tenant creation)
export interface PersonOnlyAuthResponse {
  access_token: string;
  refresh_token: string;
  person: AuthPersonWithoutTenant;
}

// Person info without tenant context
export interface AuthPersonWithoutTenant {
  id: string;
  email: string;
  first_name: string;
  last_name: string;
}

// Request to join existing tenant
export interface JoinTenantRequest {
  tenant_subdomain: string;
}

// Request to create new tenant and join it
export interface CreateAndJoinTenantRequest {
  tenant_name: string;
  tenant_subdomain: string;
}
