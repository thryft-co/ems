import React, {
  createContext,
  useContext,
  useReducer,
  useEffect,
  ReactNode,
} from "react";
import {
  AuthContextType,
  AuthState,
  LoginRequest,
  RegisterRequest,
  AuthUser,
  AuthTenant,
  PersonOnlyAuthResponse,
  AuthPersonWithoutTenant,
  JoinTenantRequest,
  CreateAndJoinTenantRequest,
} from "@/features/auth/types/auth";
import { AuthService } from "@/features/auth/services/authService";

// Auth action types
type AuthAction =
  | { type: "SET_LOADING"; payload: boolean }
  | {
      type: "LOGIN_SUCCESS";
      payload: {
        user: AuthUser;
        tenant: AuthTenant;
        accessToken: string;
        refreshToken: string;
      };
    }
  | {
      type: "PERSON_ONLY_REGISTER_SUCCESS";
      payload: {
        person: AuthPersonWithoutTenant;
        accessToken: string;
        refreshToken: string;
      };
    }
  | { type: "LOGOUT" }
  | { type: "SELECT_TENANT"; payload: AuthTenant }
  | {
      type: "REFRESH_TOKEN_SUCCESS";
      payload: { accessToken: string; refreshToken: string };
    }
  | { type: "CLEAR_SESSION" };

// Initial authentication state
const initialState: AuthState = {
  isAuthenticated: false,
  user: null,
  currentTenant: null,
  accessToken: null,
  refreshToken: null,
  isLoading: true,
};

// Auth reducer
const authReducer = (state: AuthState, action: AuthAction): AuthState => {
  switch (action.type) {
    case "SET_LOADING":
      return { ...state, isLoading: action.payload };

    case "LOGIN_SUCCESS":
      return {
        ...state,
        isAuthenticated: true,
        user: action.payload.user,
        currentTenant: action.payload.tenant,
        accessToken: action.payload.accessToken,
        refreshToken: action.payload.refreshToken,
        isLoading: false,
      };

    case "PERSON_ONLY_REGISTER_SUCCESS":
      return {
        ...state,
        isAuthenticated: true,
        user: {
          id: action.payload.person.id,
          email: action.payload.person.email,
          first_name: action.payload.person.first_name,
          last_name: action.payload.person.last_name,
          role: "pending" as any, // Temporary role for users without tenants
        },
        currentTenant: null, // No tenant initially
        accessToken: action.payload.accessToken,
        refreshToken: action.payload.refreshToken,
        isLoading: false,
      };

    case "LOGOUT":
      return {
        ...initialState,
        isLoading: false,
      };

    case "SELECT_TENANT":
      return {
        ...state,
        currentTenant: action.payload,
      };

    case "REFRESH_TOKEN_SUCCESS":
      return {
        ...state,
        accessToken: action.payload.accessToken,
        refreshToken: action.payload.refreshToken,
      };

    case "CLEAR_SESSION":
      return {
        ...initialState,
        isLoading: false,
      };

    default:
      return state;
  }
};

// Create Auth Context
const AuthContext = createContext<AuthContextType | null>(null);

// Auth Provider Props
interface AuthProviderProps {
  children: ReactNode;
}

// Auth Provider Component
export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [state, dispatch] = useReducer(authReducer, initialState);

  // Load session from localStorage on mount
  useEffect(() => {
    const loadSession = () => {
      try {
        const accessToken = localStorage.getItem("accessToken");
        const refreshToken = localStorage.getItem("refreshToken");
        const user = localStorage.getItem("user");
        const currentTenant = localStorage.getItem("currentTenant");

        if (accessToken && refreshToken && user && currentTenant) {
          const userData = JSON.parse(user);
          const tenantData = JSON.parse(currentTenant);

          // Check if token is expired
          if (!AuthService.isTokenExpired(accessToken)) {
            dispatch({
              type: "LOGIN_SUCCESS",
              payload: {
                user: userData,
                tenant: tenantData,
                accessToken,
                refreshToken,
              },
            });
            AuthService.setAuthHeader(accessToken);
          } else {
            // Try to refresh the token
            refreshAccessToken();
          }
        } else {
          dispatch({ type: "SET_LOADING", payload: false });
        }
      } catch (error) {
        console.error("Error loading session:", error);
        clearSession();
      }
    };

    loadSession();
  }, []);

  // Save session to localStorage
  const saveSession = (
    user: AuthUser,
    tenant: AuthTenant,
    accessToken: string,
    refreshToken: string,
  ) => {
    localStorage.setItem("accessToken", accessToken);
    localStorage.setItem("refreshToken", refreshToken);
    localStorage.setItem("user", JSON.stringify(user));
    localStorage.setItem("currentTenant", JSON.stringify(tenant));
  };

  // Clear session from localStorage
  const clearSession = () => {
    localStorage.removeItem("accessToken");
    localStorage.removeItem("refreshToken");
    localStorage.removeItem("user");
    localStorage.removeItem("currentTenant");
    AuthService.removeAuthHeader();
    dispatch({ type: "CLEAR_SESSION" });
  };

  // Login function
  const login = async (credentials: LoginRequest): Promise<void> => {
    try {
      dispatch({ type: "SET_LOADING", payload: true });

      const response = await AuthService.login(credentials);

      dispatch({
        type: "LOGIN_SUCCESS",
        payload: {
          user: response.user,
          tenant: response.tenant,
          accessToken: response.access_token,
          refreshToken: response.refresh_token,
        },
      });

      saveSession(
        response.user,
        response.tenant,
        response.access_token,
        response.refresh_token,
      );
      AuthService.setAuthHeader(response.access_token);
    } catch (error) {
      dispatch({ type: "SET_LOADING", payload: false });
      throw error;
    }
  };

  // Register function
  const register = async (data: RegisterRequest): Promise<void> => {
    try {
      dispatch({ type: "SET_LOADING", payload: true });

      const response = await AuthService.personOnlyRegister(data);

      dispatch({
        type: "PERSON_ONLY_REGISTER_SUCCESS",
        payload: {
          person: response.person,
          accessToken: response.access_token,
          refreshToken: response.refresh_token,
        },
      });

      // Save person-only session (without tenant)
      localStorage.setItem("accessToken", response.access_token);
      localStorage.setItem("refreshToken", response.refresh_token);
      localStorage.setItem(
        "user",
        JSON.stringify({
          id: response.person.id,
          email: response.person.email,
          first_name: response.person.first_name,
          last_name: response.person.last_name,
          role: "pending",
        }),
      );
      // Don't save currentTenant since there isn't one yet

      AuthService.setAuthHeader(response.access_token);
    } catch (error) {
      dispatch({ type: "SET_LOADING", payload: false });
      throw error;
    }
  };

  // Join existing tenant function
  const joinTenant = async (tenantSubdomain: string): Promise<void> => {
    if (!state.accessToken) {
      throw new Error("No access token available");
    }

    try {
      dispatch({ type: "SET_LOADING", payload: true });

      const response = await AuthService.joinTenant(
        { tenant_subdomain: tenantSubdomain },
        state.accessToken,
      );

      dispatch({
        type: "LOGIN_SUCCESS",
        payload: {
          user: response.user,
          tenant: response.tenant,
          accessToken: response.access_token,
          refreshToken: response.refresh_token,
        },
      });

      saveSession(
        response.user,
        response.tenant,
        response.access_token,
        response.refresh_token,
      );
      AuthService.setAuthHeader(response.access_token);
    } catch (error) {
      dispatch({ type: "SET_LOADING", payload: false });
      throw error;
    }
  };

  // Create and join tenant function
  const createAndJoinTenant = async (
    tenantName: string,
    tenantSubdomain: string,
  ): Promise<void> => {
    if (!state.accessToken) {
      throw new Error("No access token available");
    }

    try {
      dispatch({ type: "SET_LOADING", payload: true });

      const response = await AuthService.createAndJoinTenant(
        { tenant_name: tenantName, tenant_subdomain: tenantSubdomain },
        state.accessToken,
      );

      dispatch({
        type: "LOGIN_SUCCESS",
        payload: {
          user: response.user,
          tenant: response.tenant,
          accessToken: response.access_token,
          refreshToken: response.refresh_token,
        },
      });

      saveSession(
        response.user,
        response.tenant,
        response.access_token,
        response.refresh_token,
      );
      AuthService.setAuthHeader(response.access_token);
    } catch (error) {
      dispatch({ type: "SET_LOADING", payload: false });
      throw error;
    }
  };

  // Logout function
  const logout = async (): Promise<void> => {
    try {
      if (state.refreshToken && state.accessToken && state.currentTenant) {
        await AuthService.logout(
          { refresh_token: state.refreshToken },
          state.accessToken,
          state.currentTenant.id,
        );
      }
    } catch (error) {
      console.error("Logout error:", error);
    } finally {
      clearSession();
    }
  };

  // Select tenant function
  const selectTenant = (tenant: AuthTenant | null): void => {
    dispatch({ type: "SELECT_TENANT", payload: tenant as any });
    if (tenant) {
      localStorage.setItem("currentTenant", JSON.stringify(tenant));
    } else {
      localStorage.removeItem("currentTenant");
    }
  };

  // Refresh access token function
  const refreshAccessToken = async (): Promise<void> => {
    try {
      const refreshToken = localStorage.getItem("refreshToken");

      if (!refreshToken) {
        throw new Error("No refresh token available");
      }

      const response = await AuthService.refreshToken({
        refresh_token: refreshToken,
      });

      dispatch({
        type: "REFRESH_TOKEN_SUCCESS",
        payload: {
          accessToken: response.access_token,
          refreshToken: response.refresh_token,
        },
      });

      localStorage.setItem("accessToken", response.access_token);
      localStorage.setItem("refreshToken", response.refresh_token);
      AuthService.setAuthHeader(response.access_token);
    } catch (error) {
      console.error("Token refresh failed:", error);
      clearSession();
      throw error;
    }
  };

  // Context value
  const contextValue: AuthContextType = {
    ...state,
    login,
    register,
    joinTenant,
    createAndJoinTenant,
    logout,
    selectTenant,
    refreshAccessToken,
    clearSession,
  };

  return (
    <AuthContext.Provider value={contextValue}>{children}</AuthContext.Provider>
  );
};

// useAuth hook
export const useAuth = (): AuthContextType => {
  const context = useContext(AuthContext);

  if (!context) {
    throw new Error("useAuth must be used within an AuthProvider");
  }

  return context;
};

export default AuthContext;
