import axios from "axios";
import {
  Person,
  PersonDetailResponse,
  PersonCreateResponse,
  PersonUpdateResponse,
  PersonFormData,
} from "@/features/persons/types/person";

const API_URL = "/api/v1";

// Set up axios interceptor to include tenant ID and authorization token in all requests
axios.interceptors.request.use((config) => {
  // Get tenant ID from local storage
  const tenantId = localStorage.getItem("tenantId");
  if (tenantId) {
    config.headers["X-Tenant-ID"] = tenantId;
  }

  // Get authorization token from local storage
  const accessToken = localStorage.getItem("accessToken");
  if (accessToken) {
    config.headers["Authorization"] = `Bearer ${accessToken}`;
  }

  return config;
});

// Generic Person API methods
export const getAllPersons = async (): Promise<Person[]> => {
  try {
    const response = await axios.get(`${API_URL}/person`);
    return response.data;
  } catch (error) {
    console.error("Error fetching persons:", error);
    throw error;
  }
};

export const getPersonById = async (
  personId: string,
): Promise<PersonDetailResponse> => {
  try {
    const response = await axios.get(`${API_URL}/person/${personId}`);
    return response.data;
  } catch (error) {
    console.error(`Error fetching person ${personId}:`, error);
    throw error;
  }
};

export const createPerson = async (
  personData: PersonFormData,
): Promise<PersonCreateResponse> => {
  try {
    const response = await axios.post(`${API_URL}/person`, personData);
    return response.data;
  } catch (error) {
    console.error("Error creating person:", error);
    throw error;
  }
};

export const updatePerson = async (
  personId: string,
  personData: Partial<PersonFormData>,
): Promise<PersonUpdateResponse> => {
  try {
    const response = await axios.put(
      `${API_URL}/person/${personId}`,
      personData,
    );
    return response.data;
  } catch (error) {
    console.error(`Error updating person ${personId}:`, error);
    throw error;
  }
};

export const deletePerson = async (personId: string): Promise<boolean> => {
  try {
    await axios.delete(`${API_URL}/person/${personId}`);
    return true;
  } catch (error) {
    console.error(`Error deleting person ${personId}:`, error);
    throw error;
  }
};

// Specialized Person API methods
export const getInternalPersons = async (): Promise<Person[]> => {
  try {
    const response = await axios.get<Person[]>(`${API_URL}/person/internal`);
    return response.data;
  } catch (error) {
    console.error("Error fetching internal persons:", error);
    throw error;
  }
};

export const getInternalPersonById = async (
  personId: string,
): Promise<PersonDetailResponse> => {
  try {
    const response = await axios.get<PersonDetailResponse>(
      `${API_URL}/person/internal/${personId}`,
    );
    return response.data;
  } catch (error) {
    console.error(`Error fetching internal person ${personId}:`, error);
    throw error;
  }
};

export const getCustomerPersons = async (): Promise<Person[]> => {
  try {
    const response = await axios.get<Person[]>(`${API_URL}/person/customer`);
    return response.data;
  } catch (error) {
    console.error("Error fetching customer persons:", error);
    throw error;
  }
};

export const getCustomerPersonById = async (
  personId: string,
): Promise<PersonDetailResponse> => {
  try {
    const response = await axios.get<PersonDetailResponse>(
      `${API_URL}/person/customer/${personId}`,
    );
    return response.data;
  } catch (error) {
    console.error(`Error fetching customer person ${personId}:`, error);
    throw error;
  }
};

export const getVendorPersons = async (): Promise<Person[]> => {
  try {
    const response = await axios.get<Person[]>(`${API_URL}/person/vendor`);
    return response.data;
  } catch (error) {
    console.error("Error fetching vendor persons:", error);
    throw error;
  }
};

export const getVendorPersonById = async (
  personId: string,
): Promise<PersonDetailResponse> => {
  try {
    const response = await axios.get<PersonDetailResponse>(
      `${API_URL}/person/vendor/${personId}`,
    );
    return response.data;
  } catch (error) {
    console.error(`Error fetching vendor person ${personId}:`, error);
    throw error;
  }
};

export const getDistributorPersons = async (): Promise<Person[]> => {
  try {
    const response = await axios.get<Person[]>(`${API_URL}/person/distributor`);
    return response.data;
  } catch (error) {
    console.error("Error fetching distributor persons:", error);
    throw error;
  }
};

export const getDistributorPersonById = async (
  personId: string,
): Promise<PersonDetailResponse> => {
  try {
    const response = await axios.get<PersonDetailResponse>(
      `${API_URL}/person/distributor/${personId}`,
    );
    return response.data;
  } catch (error) {
    console.error(`Error fetching distributor person ${personId}:`, error);
    throw error;
  }
};
