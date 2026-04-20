import axios from "axios";
import {
  Job,
  JobDetailResponse,
  JobCreateResponse,
  JobUpdateResponse,
  JobHistory,
  JobHistoryCreateResponse,
  JobFormData,
} from "@/features/jobs/types/job";

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

// Generic Job API calls
export const getAllJobs = async (): Promise<Job[]> => {
  try {
    const response = await axios.get<Job[]>(`${API_URL}/job`);
    return response.data;
  } catch (error) {
    console.error("Error fetching jobs:", error);
    throw error;
  }
};

export const getJobById = async (jobId: string): Promise<JobDetailResponse> => {
  try {
    console.log(`Fetching job with ID: ${jobId}`);
    // Ensure we have a proper string format for the ID
    const formattedJobId = String(jobId).trim();
    if (!formattedJobId) {
      throw new Error("Invalid job ID: empty or undefined");
    }

    console.log(`Sending get request to: ${API_URL}/job/${formattedJobId}`);
    const response = await axios.get<JobDetailResponse>(
      `${API_URL}/job/${formattedJobId}`,
    );
    console.log("Job details received:", response.data);
    return response.data;
  } catch (error) {
    console.error(`Error fetching job ${jobId}:`, error);
    if (axios.isAxiosError(error)) {
      console.error("API error details:", error.response?.data);
    }
    throw error;
  }
};

export const createJob = async (
  jobData: JobFormData,
): Promise<JobCreateResponse> => {
  try {
    const response = await axios.post<JobCreateResponse>(
      `${API_URL}/job`,
      jobData,
    );
    return response.data;
  } catch (error) {
    console.error("Error creating job:", error);
    throw error;
  }
};

export const updateJob = async (
  jobId: string,
  jobData: Partial<JobFormData>,
): Promise<JobUpdateResponse> => {
  try {
    console.log(`Updating job with ID: ${jobId}`);
    // Ensure we have a proper string format for the ID
    const formattedJobId = String(jobId).trim();
    if (!formattedJobId) {
      throw new Error("Invalid job ID: empty or undefined");
    }

    console.log(
      `Sending update request to: ${API_URL}/job/${formattedJobId}`,
      jobData,
    );
    const response = await axios.put<JobUpdateResponse>(
      `${API_URL}/job/${formattedJobId}`,
      jobData,
    );
    console.log("Update job response:", response.data);
    return response.data;
  } catch (error) {
    console.error(`Error updating job ${jobId}:`, error);
    if (axios.isAxiosError(error)) {
      console.error("API error details:", error.response?.data);
      console.error("API error status:", error.response?.status);
      console.error("API error headers:", error.response?.headers);
    }
    throw error;
  }
};

export const deleteJob = async (jobId: string): Promise<boolean> => {
  try {
    console.log(`Attempting to delete job with ID: ${jobId}`);
    // Ensure we have a proper string format for the ID
    const formattedJobId = String(jobId).trim();
    if (!formattedJobId) {
      throw new Error("Invalid job ID: empty or undefined");
    }

    console.log(`Sending delete request to: ${API_URL}/job/${formattedJobId}`);
    const response = await axios.delete(`${API_URL}/job/${formattedJobId}`);
    console.log("Delete job response:", response);
    return true;
  } catch (error) {
    console.error(`Error deleting job ${jobId}:`, error);
    if (axios.isAxiosError(error)) {
      console.error("API error details:", error.response?.data);
    }
    throw error;
  }
};

// Manufacturing Job API calls
export const getManufacturingJobs = async (): Promise<Job[]> => {
  try {
    const response = await axios.get<Job[]>(
      `${API_URL}/job?type=manufacturing`,
    );
    return response.data;
  } catch (error) {
    console.error("Error fetching manufacturing jobs:", error);
    throw error;
  }
};

export const getManufacturingJobById = async (
  jobId: string,
): Promise<JobDetailResponse> => {
  try {
    const response = await axios.get<JobDetailResponse>(
      `${API_URL}/job/${jobId}`,
    );
    return response.data;
  } catch (error) {
    console.error(`Error fetching manufacturing job ${jobId}:`, error);
    throw error;
  }
};

// QA Job API calls
export const getQAJobs = async (): Promise<Job[]> => {
  try {
    const response = await axios.get<Job[]>(`${API_URL}/job?type=qa`);
    return response.data;
  } catch (error) {
    console.error("Error fetching QA jobs:", error);
    throw error;
  }
};

export const getQAJobById = async (
  jobId: string,
): Promise<JobDetailResponse> => {
  try {
    const response = await axios.get<JobDetailResponse>(
      `${API_URL}/job/${jobId}`,
    );
    return response.data;
  } catch (error) {
    console.error(`Error fetching QA job ${jobId}:`, error);
    throw error;
  }
};

// Service Job API calls
export const getServiceJobs = async (): Promise<Job[]> => {
  try {
    const response = await axios.get<Job[]>(`${API_URL}/job?type=service`);
    return response.data;
  } catch (error) {
    console.error("Error fetching service jobs:", error);
    throw error;
  }
};

export const getServiceJobById = async (
  jobId: string,
): Promise<JobDetailResponse> => {
  try {
    const response = await axios.get<JobDetailResponse>(
      `${API_URL}/job/${jobId}`,
    );
    return response.data;
  } catch (error) {
    console.error(`Error fetching service job ${jobId}:`, error);
    throw error;
  }
};

// Job History API
export const getJobHistory = async (jobId: string): Promise<JobHistory[]> => {
  try {
    const response = await axios.get<JobHistory[]>(
      `${API_URL}/job/${jobId}/history`,
    );
    return response.data;
  } catch (error) {
    console.error(`Error fetching history for job ${jobId}:`, error);
    throw error;
  }
};

export const addJobHistoryEntry = async (
  jobId: string,
  historyData: any,
): Promise<JobHistoryCreateResponse> => {
  try {
    const response = await axios.post<JobHistoryCreateResponse>(
      `${API_URL}/job/${jobId}/history`,
      historyData,
    );
    return response.data;
  } catch (error) {
    console.error(`Error adding history for job ${jobId}:`, error);
    throw error;
  }
};
