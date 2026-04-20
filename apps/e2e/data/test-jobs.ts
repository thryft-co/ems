import { generateUniqueId, generateUUID } from '../utils/test-helpers';

export interface TestJobData {
  job_number: string;
  item_id: string;
  quantity: number;
  assigned_user_id?: string;
  supervisor_id?: string;
  customer_id?: string;
  job_type: 'manufacturing' | 'qa' | 'service';
  priority: 'low' | 'normal' | 'high' | 'urgent';
  status: 'pending' | 'in_progress' | 'on_hold' | 'completed' | 'cancelled';
  due_date?: string;
  comments?: string;
  // Manufacturing
  production_line?: string;
  batch_number?: string;
  quality_check_required?: boolean;
  // QA
  test_procedure?: string;
  inspection_type?: string;
  // Service
  service_type?: string;
  problem_description?: string;
}

/** Create test data for a manufacturing job */
export function createManufacturingJobData(
  overrides?: Partial<TestJobData>,
): TestJobData {
  return {
    job_number: generateUniqueId('MFG'),
    item_id: generateUUID(),
    quantity: 100,
    job_type: 'manufacturing',
    priority: 'normal',
    status: 'pending',
    comments: 'E2E test manufacturing job',
    production_line: 'Line A',
    batch_number: generateUniqueId('BATCH'),
    quality_check_required: true,
    ...overrides,
  };
}

/** Create test data for a QA job */
export function createQAJobData(
  overrides?: Partial<TestJobData>,
): TestJobData {
  return {
    job_number: generateUniqueId('QA'),
    item_id: generateUUID(),
    quantity: 50,
    job_type: 'qa',
    priority: 'high',
    status: 'pending',
    comments: 'E2E test QA job',
    test_procedure: 'Standard inspection protocol',
    inspection_type: 'Visual + Functional',
    ...overrides,
  };
}

/** Create test data for a service job */
export function createServiceJobData(
  overrides?: Partial<TestJobData>,
): TestJobData {
  return {
    job_number: generateUniqueId('SVC'),
    item_id: generateUUID(),
    quantity: 1,
    job_type: 'service',
    priority: 'normal',
    status: 'pending',
    comments: 'E2E test service job',
    service_type: 'Repair',
    problem_description: 'Unit not powering on after firmware update',
    ...overrides,
  };
}
