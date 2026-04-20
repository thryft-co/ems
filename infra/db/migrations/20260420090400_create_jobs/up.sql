-- Migration: Create jobs and related tables
-- This migration creates the jobs management system with multi-tenant support
-- PREREQUISITE: Run 000_supabase_setup.sql, 000_create_tenants_table.sql, and 101_create_person_tables.sql first

-- Note: Using VARCHAR with CHECK constraints instead of ENUMs for better Supabase compatibility
-- CREATE TYPE job_type_enum AS ENUM ('manufacturing', 'qa', 'service');
-- CREATE TYPE job_status_enum AS ENUM ('pending', 'in_progress', 'on_hold', 'completed', 'cancelled');
-- CREATE TYPE job_priority_enum AS ENUM ('low', 'normal', 'high', 'urgent');

-- Create main jobs table
CREATE TABLE public.jobs (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  job_number VARCHAR(50) NOT NULL,
  item_id UUID,
  quantity INTEGER NOT NULL CHECK (quantity > 0),
  assigned_person_id UUID REFERENCES public.person(id),
  supervisor_id UUID REFERENCES public.person(id),
  customer_id UUID REFERENCES public.person(id),
  job_type VARCHAR(20) NOT NULL CHECK (job_type IN ('manufacturing', 'qa', 'service')),
  priority VARCHAR(20) DEFAULT 'normal' CHECK (priority IN ('low', 'normal', 'high', 'urgent')),
  start_date TIMESTAMP WITH TIME ZONE,
  end_date TIMESTAMP WITH TIME ZONE,
  due_date TIMESTAMP WITH TIME ZONE,
  status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'on_hold', 'completed', 'cancelled')),
  comments TEXT,
  materials_consumed JSONB,
  labor_hours DOUBLE PRECISION,
  metadata JSONB,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(tenant_id, job_number)
);

-- Create manufacturing job specific table
CREATE TABLE public.manufacturing_job (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  job_id UUID NOT NULL REFERENCES public.jobs(id) ON DELETE CASCADE,
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  work_order_number VARCHAR(50),
  production_line VARCHAR(100),
  machine_id VARCHAR(50),
  setup_time_hours DOUBLE PRECISION,
  cycle_time_minutes DOUBLE PRECISION,
  quality_check_required BOOLEAN DEFAULT false,
  batch_size INTEGER,
  tool_requirements JSONB,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(job_id, tenant_id)
);

-- Create QA job specific table
CREATE TABLE public.qa_job (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  job_id UUID NOT NULL REFERENCES public.jobs(id) ON DELETE CASCADE,
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  inspection_type VARCHAR(50),
  test_procedure_id VARCHAR(50),
  acceptance_criteria TEXT,
  sampling_size INTEGER,
  test_equipment JSONB,
  calibration_required BOOLEAN DEFAULT false,
  environmental_conditions JSONB,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(job_id, tenant_id)
);

-- Create service job specific table
CREATE TABLE public.service_job (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  job_id UUID NOT NULL REFERENCES public.jobs(id) ON DELETE CASCADE,
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  service_type VARCHAR(50),
  location VARCHAR(200),
  equipment_serial_number VARCHAR(100),
  maintenance_type VARCHAR(50),
  parts_required JSONB,
  safety_requirements JSONB,
  travel_time_hours DOUBLE PRECISION,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(job_id, tenant_id)
);

-- Create job history table for audit trail
CREATE TABLE public.job_history (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  job_id UUID NOT NULL REFERENCES public.jobs(id) ON DELETE CASCADE,
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  person_id UUID REFERENCES public.person(id),
  action VARCHAR(50) NOT NULL,
  previous_status VARCHAR(20),
  new_status VARCHAR(20),
  notes TEXT,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for jobs table
CREATE INDEX idx_jobs_tenant_id ON public.jobs(tenant_id);
CREATE INDEX idx_jobs_job_number ON public.jobs(job_number);
CREATE INDEX idx_jobs_job_type ON public.jobs(job_type);
CREATE INDEX idx_jobs_status ON public.jobs(status);
CREATE INDEX idx_jobs_priority ON public.jobs(priority);
CREATE INDEX idx_jobs_assigned_person_id ON public.jobs(assigned_person_id);
CREATE INDEX idx_jobs_supervisor_id ON public.jobs(supervisor_id);
CREATE INDEX idx_jobs_customer_id ON public.jobs(customer_id);
CREATE INDEX idx_jobs_due_date ON public.jobs(due_date);
CREATE INDEX idx_jobs_created_at ON public.jobs(created_at);

-- Create indexes for type-specific tables
CREATE INDEX idx_manufacturing_job_tenant_id ON public.manufacturing_job(tenant_id);
CREATE INDEX idx_manufacturing_job_work_order ON public.manufacturing_job(work_order_number);
CREATE INDEX idx_manufacturing_job_production_line ON public.manufacturing_job(production_line);
CREATE INDEX idx_manufacturing_job_machine_id ON public.manufacturing_job(machine_id);

CREATE INDEX idx_qa_job_tenant_id ON public.qa_job(tenant_id);
CREATE INDEX idx_qa_job_inspection_type ON public.qa_job(inspection_type);
CREATE INDEX idx_qa_job_test_procedure ON public.qa_job(test_procedure_id);

CREATE INDEX idx_service_job_tenant_id ON public.service_job(tenant_id);
CREATE INDEX idx_service_job_service_type ON public.service_job(service_type);
CREATE INDEX idx_service_job_location ON public.service_job(location);
CREATE INDEX idx_service_job_equipment_serial ON public.service_job(equipment_serial_number);

CREATE INDEX idx_job_history_tenant_id ON public.job_history(tenant_id);
CREATE INDEX idx_job_history_job_id ON public.job_history(job_id);
CREATE INDEX idx_job_history_person_id ON public.job_history(person_id);
CREATE INDEX idx_job_history_action ON public.job_history(action);
CREATE INDEX idx_job_history_created_at ON public.job_history(created_at);

-- Create triggers for updated_at timestamps (uses function from 000_supabase_setup.sql)
CREATE TRIGGER update_jobs_updated_at BEFORE UPDATE ON public.jobs FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();
CREATE TRIGGER update_manufacturing_job_updated_at BEFORE UPDATE ON public.manufacturing_job FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();
CREATE TRIGGER update_qa_job_updated_at BEFORE UPDATE ON public.qa_job FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();
CREATE TRIGGER update_service_job_updated_at BEFORE UPDATE ON public.service_job FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

-- Add RLS (Row Level Security) policies for tenant isolation
ALTER TABLE public.jobs ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.manufacturing_job ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.qa_job ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.service_job ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.job_history ENABLE ROW LEVEL SECURITY;

-- Create RLS policies for jobs table
CREATE POLICY "jobs_tenant_isolation" ON public.jobs
    FOR ALL USING (
        tenant_id = public.get_current_tenant_id()
    );

-- Create RLS policies for type-specific tables
CREATE POLICY "manufacturing_job_tenant_isolation" ON public.manufacturing_job
    FOR ALL USING (
        tenant_id = public.get_current_tenant_id()
    );

CREATE POLICY "qa_job_tenant_isolation" ON public.qa_job
    FOR ALL USING (
        tenant_id = public.get_current_tenant_id()
    );

CREATE POLICY "service_job_tenant_isolation" ON public.service_job
    FOR ALL USING (
        tenant_id = public.get_current_tenant_id()
    );

CREATE POLICY "job_history_tenant_isolation" ON public.job_history
    FOR ALL USING (
        tenant_id = public.get_current_tenant_id()
    );

-- Create trigger function for job status change logging
CREATE OR REPLACE FUNCTION public.log_job_status_change()
RETURNS TRIGGER 
SECURITY DEFINER
SET search_path = public
AS $$
BEGIN
    -- Only log if status has changed
    IF OLD.status IS DISTINCT FROM NEW.status THEN
        INSERT INTO public.job_history (job_id, tenant_id, action, previous_status, new_status, notes)
        VALUES (NEW.id, NEW.tenant_id, 'status_change', OLD.status, NEW.status, 'Automatic status change log');
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for automatic job status change logging
CREATE TRIGGER log_job_status_change_trigger 
    AFTER UPDATE ON public.jobs 
    FOR EACH ROW 
    EXECUTE FUNCTION public.log_job_status_change();

-- Grant necessary permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON public.jobs TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.manufacturing_job TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.qa_job TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.service_job TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.job_history TO authenticated, service_role;
GRANT EXECUTE ON FUNCTION public.log_job_status_change() TO postgres, service_role; 