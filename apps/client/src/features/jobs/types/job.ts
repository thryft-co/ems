// Job status options
export type JobStatus =
  | "pending"
  | "in_progress"
  | "on_hold"
  | "completed"
  | "cancelled";

// Job priority options
export type JobPriority = "low" | "normal" | "high" | "urgent";

// Job types
export type JobType = "manufacturing" | "qa" | "service";

// Base Job Interface
export interface Job {
  id: string;
  tenant_id?: string;
  job_number: string;
  item_id: string;
  quantity: number;
  assigned_user_id?: string;
  supervisor_id?: string;
  customer_id?: string;
  job_type: JobType;
  priority: JobPriority;
  start_date?: string;
  end_date?: string;
  due_date?: string;
  status: JobStatus;
  comments?: string;
  materials_consumed?: any[];
  labor_hours?: number;
  meta_info?: Record<string, any>;
  created_at?: string;
  updated_at?: string;
}

// Manufacturing Job Interface
export interface ManufacturingJobDetails {
  production_line?: string;
  batch_number?: string;
  raw_materials?: any[];
  quality_check_required?: boolean;
}

export interface ManufacturingJob extends Job {
  manufacturing: ManufacturingJobDetails;
}

// QA Job Interface
export interface QAJobDetails {
  test_procedure?: string;
  pass_criteria?: any[];
  inspection_type?: string;
  qa_results?: Record<string, any>;
}

export interface QAJob extends Job {
  qa: QAJobDetails;
}

// Service Job Interface
export interface ServiceJobDetails {
  service_type?: string;
  problem_description?: string;
  diagnosis?: string;
  solution?: string;
  parts_replaced?: any[];
}

export interface ServiceJob extends Job {
  service: ServiceJobDetails;
}

// Job History Interface
export interface JobHistory {
  id: string;
  job_id: string;
  user_id?: string;
  action: string;
  previous_status?: JobStatus;
  new_status?: JobStatus;
  notes?: string;
  created_at?: string;
}

// Job Form Interface for creating/updating jobs
export interface JobFormData {
  job_number: string;
  item_id: string;
  quantity: number;
  assigned_user_id?: string;
  supervisor_id?: string;
  customer_id?: string;
  job_type: JobType;
  priority: JobPriority;
  start_date?: string;
  end_date?: string;
  due_date?: string;
  status: JobStatus;
  comments?: string;
  materials_consumed?: string; // JSON string
  labor_hours?: number;
  meta_info?: string; // JSON string

  // Manufacturing specific
  production_line?: string;
  batch_number?: string;
  raw_materials?: string; // JSON string
  quality_check_required?: boolean;

  // QA specific
  test_procedure?: string;
  pass_criteria?: string; // JSON string
  inspection_type?: string;
  qa_results?: string; // JSON string

  // Service specific
  service_type?: string;
  problem_description?: string;
  diagnosis?: string;
  solution?: string;
  parts_replaced?: string; // JSON string
}

// API Response Interfaces
export interface JobListResponse extends Array<Job> {}

export interface JobDetailResponse extends Job {
  manufacturing?: ManufacturingJobDetails;
  qa?: QAJobDetails;
  service?: ServiceJobDetails;
  history?: JobHistory[];
}

export interface JobCreateResponse {
  id: string;
}

export interface JobUpdateResponse {
  id: string;
  message: string;
}

export interface JobHistoryCreateResponse {
  id: string;
  job_id: string;
  action: string;
  created_at?: string;
}

export interface JobHistoryListResponse extends Array<JobHistory> {}
