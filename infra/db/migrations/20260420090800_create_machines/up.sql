-- Migration: Create machine management tables
-- This migration creates the machine management system with multi-tenant support
-- PREREQUISITE: Run 000_supabase_setup.sql, 000_create_tenants_table.sql, 101_create_person_tables.sql, 201_create_jobs_tables.sql, and 402_create_asset_tables.sql first

-- Create machines table
CREATE TABLE public.machines (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  name VARCHAR(100) NOT NULL,
  ip VARCHAR(45) NOT NULL,
  port INTEGER NOT NULL CHECK (port > 0 AND port <= 65535),
  protocol VARCHAR(20) NOT NULL CHECK (protocol IN ('http', 'mqtt', 'graph', 'tcp', 'udp', 'websocket')),
  status VARCHAR(20) NOT NULL DEFAULT 'offline' CHECK (status IN ('offline', 'idle', 'busy', 'maintenance', 'error')),
  action VARCHAR(20) CHECK (action IN ('run', 'test', 'calibrate', 'diagnostics', 'emergency_stop')),
  payload JSONB DEFAULT '{}',
  last_heartbeat TIMESTAMP WITH TIME ZONE,
  metadata JSONB DEFAULT '{}',
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for machines table
CREATE INDEX idx_machines_tenant_id ON public.machines(tenant_id);
CREATE INDEX idx_machines_name ON public.machines(name);
CREATE INDEX idx_machines_status ON public.machines(status);
CREATE INDEX idx_machines_last_heartbeat ON public.machines(last_heartbeat);
CREATE INDEX idx_machines_ip_port ON public.machines(ip, port);
CREATE INDEX idx_machines_protocol ON public.machines(protocol);
CREATE INDEX idx_machines_action ON public.machines(action);
CREATE INDEX idx_machines_metadata ON public.machines USING GIN (metadata);
CREATE INDEX idx_machines_payload ON public.machines USING GIN (payload);

-- Create machine_item_relationships table for tracking which items can be built/tested by which machines
CREATE TABLE public.machine_item_relationships (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  machine_id UUID NOT NULL REFERENCES public.machines(id) ON DELETE CASCADE,
  item_id UUID NOT NULL REFERENCES public.items(id) ON DELETE CASCADE,
  relationship_type VARCHAR(20) NOT NULL CHECK (relationship_type IN ('builds', 'tests', 'calibrates')),
  notes TEXT,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(machine_id, item_id, relationship_type)
);

CREATE INDEX idx_machine_item_relationships_machine_id ON public.machine_item_relationships(machine_id);
CREATE INDEX idx_machine_item_relationships_item_id ON public.machine_item_relationships(item_id);
CREATE INDEX idx_machine_item_relationships_type ON public.machine_item_relationships(relationship_type);

-- Create machine_asset_relationships table for tracking which assets (firmware, etc.) are used by which machines
CREATE TABLE public.machine_asset_relationships (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  machine_id UUID NOT NULL REFERENCES public.machines(id) ON DELETE CASCADE,
  asset_id UUID NOT NULL REFERENCES public.assets(id) ON DELETE CASCADE,
  relationship_type VARCHAR(20) NOT NULL CHECK (relationship_type IN ('firmware', 'configuration', 'calibration_data')),
  notes TEXT,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(machine_id, asset_id, relationship_type)
);

CREATE INDEX idx_machine_asset_relationships_machine_id ON public.machine_asset_relationships(machine_id);
CREATE INDEX idx_machine_asset_relationships_asset_id ON public.machine_asset_relationships(asset_id);
CREATE INDEX idx_machine_asset_relationships_type ON public.machine_asset_relationships(relationship_type);

-- Create machine_operator_assignments table for tracking which people are assigned to operate which machines
CREATE TABLE public.machine_operator_assignments (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  machine_id UUID NOT NULL REFERENCES public.machines(id) ON DELETE CASCADE,
  person_id UUID NOT NULL REFERENCES public.person(id) ON DELETE CASCADE,
  assignment_type VARCHAR(20) NOT NULL CHECK (assignment_type IN ('primary', 'backup', 'maintenance')),
  notes TEXT,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(machine_id, person_id, assignment_type)
);

CREATE INDEX idx_machine_operator_assignments_machine_id ON public.machine_operator_assignments(machine_id);
CREATE INDEX idx_machine_operator_assignments_person_id ON public.machine_operator_assignments(person_id);
CREATE INDEX idx_machine_operator_assignments_type ON public.machine_operator_assignments(assignment_type);

-- Create machine_job_assignments table for tracking which jobs are assigned to which machines
CREATE TABLE public.machine_job_assignments (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  machine_id UUID NOT NULL REFERENCES public.machines(id) ON DELETE CASCADE,
  job_id UUID NOT NULL REFERENCES public.jobs(id) ON DELETE CASCADE,
  status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'completed', 'failed')),
  start_time TIMESTAMP WITH TIME ZONE,
  end_time TIMESTAMP WITH TIME ZONE,
  notes TEXT,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(machine_id, job_id)
);

CREATE INDEX idx_machine_job_assignments_machine_id ON public.machine_job_assignments(machine_id);
CREATE INDEX idx_machine_job_assignments_job_id ON public.machine_job_assignments(job_id);
CREATE INDEX idx_machine_job_assignments_status ON public.machine_job_assignments(status);
CREATE INDEX idx_machine_job_assignments_start_time ON public.machine_job_assignments(start_time);
CREATE INDEX idx_machine_job_assignments_end_time ON public.machine_job_assignments(end_time);

-- Create triggers for updated_at timestamps (uses function from 000_supabase_setup.sql)
CREATE TRIGGER update_machines_updated_at 
  BEFORE UPDATE ON public.machines 
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_machine_item_relationships_updated_at 
  BEFORE UPDATE ON public.machine_item_relationships 
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_machine_asset_relationships_updated_at 
  BEFORE UPDATE ON public.machine_asset_relationships 
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_machine_operator_assignments_updated_at 
  BEFORE UPDATE ON public.machine_operator_assignments 
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_machine_job_assignments_updated_at 
  BEFORE UPDATE ON public.machine_job_assignments 
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

-- Add RLS (Row Level Security) policies for tenant isolation
ALTER TABLE public.machines ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.machine_item_relationships ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.machine_asset_relationships ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.machine_operator_assignments ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.machine_job_assignments ENABLE ROW LEVEL SECURITY;

-- Create RLS policy for machines table (tenant isolation)
CREATE POLICY "machines_tenant_isolation" ON public.machines
    FOR ALL USING (
        tenant_id = public.get_current_tenant_id()
    );

-- Create RLS policies for relationship tables (through machines relationship)
CREATE POLICY "machine_item_relationships_tenant_isolation" ON public.machine_item_relationships
    FOR ALL USING (
        machine_id IN (
            SELECT id FROM public.machines 
            WHERE tenant_id = public.get_current_tenant_id()
        )
    );

CREATE POLICY "machine_asset_relationships_tenant_isolation" ON public.machine_asset_relationships
    FOR ALL USING (
        machine_id IN (
            SELECT id FROM public.machines 
            WHERE tenant_id = public.get_current_tenant_id()
        )
    );

CREATE POLICY "machine_operator_assignments_tenant_isolation" ON public.machine_operator_assignments
    FOR ALL USING (
        machine_id IN (
            SELECT id FROM public.machines 
            WHERE tenant_id = public.get_current_tenant_id()
        )
    );

CREATE POLICY "machine_job_assignments_tenant_isolation" ON public.machine_job_assignments
    FOR ALL USING (
        machine_id IN (
            SELECT id FROM public.machines 
            WHERE tenant_id = public.get_current_tenant_id()
        )
    );

-- Grant necessary permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON public.machines TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.machine_item_relationships TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.machine_asset_relationships TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.machine_operator_assignments TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.machine_job_assignments TO authenticated, service_role;

-- Add comments for documentation
COMMENT ON TABLE public.machines IS 'Main machine registry with communication and status details';
COMMENT ON TABLE public.machine_item_relationships IS 'Mapping of which items can be processed by which machines';
COMMENT ON TABLE public.machine_asset_relationships IS 'Mapping of which assets are used by which machines';
COMMENT ON TABLE public.machine_operator_assignments IS 'Assignment of operators to machines';
COMMENT ON TABLE public.machine_job_assignments IS 'Assignment of jobs to machines for execution';

COMMENT ON COLUMN public.machines.ip IS 'IP address for machine communication';
COMMENT ON COLUMN public.machines.port IS 'Port number for machine communication';
COMMENT ON COLUMN public.machines.protocol IS 'Communication protocol (http, mqtt, tcp, etc.)';
COMMENT ON COLUMN public.machines.status IS 'Current operational status of the machine';
COMMENT ON COLUMN public.machines.action IS 'Current or requested action for the machine';
COMMENT ON COLUMN public.machines.payload IS 'Action-specific payload data in JSON format';
COMMENT ON COLUMN public.machines.last_heartbeat IS 'Timestamp of last heartbeat from machine';
COMMENT ON COLUMN public.machines.metadata IS 'Additional machine metadata in JSON format'; 