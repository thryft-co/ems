import axios, { AxiosError } from "axios";
import {
  LoginRequest,
  RegisterRequest,
  AuthResponse,
  RefreshTokenRequest,
  RefreshTokenResponse,
  LogoutRequest,
  OAuthLoginRequest,
  OAuthUrlResponse,
  OAuthCallbackRequest,
  InternalPersonOAuthRegisterRequest,
  PersonOnlyAuthResponse,
  JoinTenantRequest,
  CreateAndJoinTenantRequest,
} from "@/features/auth/types/auth";

const API_URL = "/api/v1";

// Create an axios instance for auth requests
const authAxios = axios.create({
  baseURL: API_URL,
});

// Auth Service Class
export class AuthService {
  // Login with email and password
  static async login(credentials: LoginRequest): Promise<AuthResponse> {
    try {
      const response = await authAxios.post<AuthResponse>(
        "/auth/login",
        credentials,
      );
      return response.data;
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 401) {
        throw new Error("Invalid email or password");
      } else if (axiosError.response?.status === 404) {
        throw new Error("User or tenant not found");
      } else {
        throw new Error("Login failed. Please try again.");
      }
    }
  }

  // Register new user and tenant
  static async register(data: RegisterRequest): Promise<AuthResponse> {
    try {
      const response = await authAxios.post<AuthResponse>(
        "/auth/register",
        data,
      );
      return response.data;
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 409) {
        throw new Error("Email or subdomain already exists");
      } else if (axiosError.response?.status === 400) {
        throw new Error("Invalid registration data");
      } else {
        throw new Error("Registration failed. Please try again.");
      }
    }
  }

  // Register person without creating tenant
  static async personOnlyRegister(
    data: RegisterRequest,
  ): Promise<PersonOnlyAuthResponse> {
    try {
      const response = await authAxios.post<PersonOnlyAuthResponse>(
        "/auth/person-register",
        {
          email: data.email,
          first_name: data.first_name,
          last_name: data.last_name,
          password: data.password,
        },
      );
      return response.data;
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 409) {
        throw new Error("Email already exists");
      } else if (axiosError.response?.status === 400) {
        throw new Error("Invalid registration data");
      } else if (axiosError.response?.status === 500) {
        throw new Error("Server error. Please try again later.");
      } else {
        throw new Error("Registration failed. Please try again.");
      }
    }
  }

  // Join existing tenant
  static async joinTenant(
    data: JoinTenantRequest,
    accessToken: string,
  ): Promise<AuthResponse> {
    try {
      const response = await authAxios.post<AuthResponse>(
        "/auth/join-tenant",
        data,
        {
          headers: {
            Authorization: `Bearer ${accessToken}`,
          },
        },
      );
      return response.data;
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 404) {
        throw new Error("Organization not found");
      } else if (axiosError.response?.status === 403) {
        throw new Error("Organization is not active");
      } else if (axiosError.response?.status === 409) {
        throw new Error("You are already a member of this organization");
      } else if (axiosError.response?.status === 401) {
        throw new Error("Please log in again");
      } else {
        throw new Error("Failed to join organization. Please try again.");
      }
    }
  }

  // Create new tenant and join it
  static async createAndJoinTenant(
    data: CreateAndJoinTenantRequest,
    accessToken: string,
  ): Promise<AuthResponse> {
    try {
      const response = await authAxios.post<AuthResponse>(
        "/auth/create-tenant",
        data,
        {
          headers: {
            Authorization: `Bearer ${accessToken}`,
          },
        },
      );
      return response.data;
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 409) {
        throw new Error("Organization subdomain already exists");
      } else if (axiosError.response?.status === 400) {
        throw new Error("Invalid organization data");
      } else if (axiosError.response?.status === 401) {
        throw new Error("Please log in again");
      } else {
        throw new Error("Failed to create organization. Please try again.");
      }
    }
  }

  // Refresh access token
  static async refreshToken(
    refreshTokenData: RefreshTokenRequest,
  ): Promise<RefreshTokenResponse> {
    try {
      const response = await authAxios.post<RefreshTokenResponse>(
        "/auth/refresh",
        refreshTokenData,
      );
      return response.data;
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 401) {
        throw new Error("Invalid refresh token");
      } else {
        throw new Error("Token refresh failed");
      }
    }
  }

  // Logout user
  static async logout(
    logoutData: LogoutRequest,
    accessToken: string,
    tenantId?: string,
  ): Promise<void> {
    try {
      const headers: Record<string, string> = {
        Authorization: `Bearer ${accessToken}`,
      };

      // Add tenant ID header if provided
      if (tenantId) {
        headers["X-Tenant-ID"] = tenantId;
      }

      await authAxios.post("/auth/logout", logoutData, {
        headers,
      });
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 401) {
        throw new Error("Unauthorized logout request");
      } else if (axiosError.response?.status === 400) {
        throw new Error("Invalid logout request");
      } else {
        throw new Error("Logout failed");
      }
    }
  }

  // OAuth: Get authorization URL
  static async getOAuthUrl(
    request: OAuthLoginRequest,
  ): Promise<OAuthUrlResponse> {
    try {
      const response = await authAxios.post<OAuthUrlResponse>(
        "/auth/oauth/url",
        request,
      );
      return response.data;
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 404) {
        throw new Error("Tenant not found");
      } else if (axiosError.response?.status === 403) {
        throw new Error("Tenant not active");
      } else if (axiosError.response?.status === 503) {
        throw new Error("OAuth not configured for this tenant");
      } else {
        throw new Error("Failed to get OAuth URL");
      }
    }
  }

  // OAuth: Handle callback
  static async oauthCallback(
    request: OAuthCallbackRequest,
  ): Promise<AuthResponse> {
    try {
      const response = await authAxios.post<AuthResponse>(
        "/auth/oauth/callback",
        request,
      );
      return response.data;
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 404) {
        throw new Error("Tenant or user not found");
      } else if (axiosError.response?.status === 403) {
        throw new Error("Tenant not active or user not authorized");
      } else if (axiosError.response?.status === 400) {
        throw new Error("Invalid OAuth callback data");
      } else if (axiosError.response?.status === 503) {
        throw new Error("OAuth not configured");
      } else {
        throw new Error("OAuth authentication failed");
      }
    }
  }

  // OAuth: Register internal person
  static async oauthRegisterInternal(
    request: InternalPersonOAuthRegisterRequest,
  ): Promise<AuthResponse> {
    try {
      const response = await authAxios.post<AuthResponse>(
        "/auth/oauth/register/internal",
        request,
      );
      return response.data;
    } catch (error) {
      const axiosError = error as AxiosError;
      if (axiosError.response?.status === 409) {
        throw new Error("Email or subdomain already exists");
      } else if (axiosError.response?.status === 400) {
        throw new Error("Invalid OAuth registration data");
      } else if (axiosError.response?.status === 503) {
        throw new Error("OAuth not configured");
      } else {
        throw new Error("OAuth registration failed");
      }
    }
  }

  // Utility: Set authorization header for subsequent requests
  static setAuthHeader(token: string): void {
    authAxios.defaults.headers.common["Authorization"] = `Bearer ${token}`;
  }

  // Utility: Remove authorization header
  static removeAuthHeader(): void {
    delete authAxios.defaults.headers.common["Authorization"];
  }

  // Utility: Check if token is expired
  static isTokenExpired(token: string): boolean {
    try {
      const payload = JSON.parse(atob(token.split(".")[1]));
      const currentTime = Date.now() / 1000;
      return payload.exp < currentTime;
    } catch (error) {
      return true;
    }
  }
}

export default AuthService;
