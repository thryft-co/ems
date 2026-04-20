-- Migration: Create person and tenant_person tables
-- This migration creates the person management system with multi-tenant support
-- PREREQUISITE: Run 000_supabase_setup.sql and 000_create_tenants_table.sql first

-- Create person table (replaces users table)
CREATE TABLE public.person (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  supabase_uid UUID UNIQUE NOT NULL,
  name VARCHAR(100) NOT NULL,
  email VARCHAR(100) UNIQUE NOT NULL,
  phone VARCHAR(20),
  global_access VARCHAR(20)[] DEFAULT ARRAY['standard'],
  is_active BOOLEAN DEFAULT true,
  last_login TIMESTAMP WITH TIME ZONE,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for person table
CREATE INDEX idx_person_email ON public.person(email);
CREATE INDEX idx_person_global_access ON public.person USING GIN (global_access);
CREATE INDEX idx_person_supabase_uid ON public.person(supabase_uid);
CREATE INDEX idx_person_is_active ON public.person(is_active);

-- Create tenant_person table for many-to-many relationship with roles
CREATE TABLE public.tenant_person (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  person_id UUID NOT NULL REFERENCES public.person(id) ON DELETE CASCADE,
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  role VARCHAR(20) NOT NULL CHECK (role IN ('internal', 'customer', 'vendor', 'distributor')),
  access_level VARCHAR(20)[] DEFAULT ARRAY['standard'],
  is_primary BOOLEAN DEFAULT false,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(person_id, tenant_id, role)
);

-- Create indexes for tenant_person table
CREATE INDEX idx_tenant_person_tenant_id ON public.tenant_person(tenant_id);
CREATE INDEX idx_tenant_person_person_id ON public.tenant_person(person_id);
CREATE INDEX idx_tenant_person_role ON public.tenant_person(role);
CREATE INDEX idx_tenant_person_is_primary ON public.tenant_person(is_primary);

-- Create internal person data table
CREATE TABLE public.internal_person (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  person_id UUID NOT NULL REFERENCES public.person(id) ON DELETE CASCADE,
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  department VARCHAR(50),
  position VARCHAR(100),
  employee_id VARCHAR(20),
  hire_date TIMESTAMP WITH TIME ZONE,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(person_id, tenant_id)
);

-- Create customer person data table
CREATE TABLE public.customer_person (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  person_id UUID NOT NULL REFERENCES public.person(id) ON DELETE CASCADE,
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  company VARCHAR(100),
  industry VARCHAR(50),
  customer_since TIMESTAMP WITH TIME ZONE,
  account_manager_id UUID REFERENCES public.person(id),
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(person_id, tenant_id)
);

-- Create vendor person data table
CREATE TABLE public.vendor_person (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  person_id UUID NOT NULL REFERENCES public.person(id) ON DELETE CASCADE,
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  company VARCHAR(100),
  service_type VARCHAR(100),
  contract_start TIMESTAMP WITH TIME ZONE,
  contract_end TIMESTAMP WITH TIME ZONE,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(person_id, tenant_id)
);

-- Create distributor person data table
CREATE TABLE public.distributor_person (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  person_id UUID NOT NULL REFERENCES public.person(id) ON DELETE CASCADE,
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  company VARCHAR(100),
  territory VARCHAR(100),
  distribution_tier VARCHAR(50),
  commission_rate VARCHAR(10),
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(person_id, tenant_id)
);

-- Create indexes for type-specific tables
CREATE INDEX idx_internal_person_tenant_id ON public.internal_person(tenant_id);
CREATE INDEX idx_internal_person_employee_id ON public.internal_person(employee_id);

CREATE INDEX idx_customer_person_tenant_id ON public.customer_person(tenant_id);
CREATE INDEX idx_customer_person_company ON public.customer_person(company);
CREATE INDEX idx_customer_person_account_manager ON public.customer_person(account_manager_id);

CREATE INDEX idx_vendor_person_tenant_id ON public.vendor_person(tenant_id);
CREATE INDEX idx_vendor_person_company ON public.vendor_person(company);

CREATE INDEX idx_distributor_person_tenant_id ON public.distributor_person(tenant_id);
CREATE INDEX idx_distributor_person_territory ON public.distributor_person(territory);

-- Create triggers for updated_at timestamps (uses function from 000_supabase_setup.sql)
CREATE TRIGGER update_person_updated_at BEFORE UPDATE ON public.person FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();
CREATE TRIGGER update_tenant_person_updated_at BEFORE UPDATE ON public.tenant_person FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();
CREATE TRIGGER update_internal_person_updated_at BEFORE UPDATE ON public.internal_person FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();
CREATE TRIGGER update_customer_person_updated_at BEFORE UPDATE ON public.customer_person FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();
CREATE TRIGGER update_vendor_person_updated_at BEFORE UPDATE ON public.vendor_person FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();
CREATE TRIGGER update_distributor_person_updated_at BEFORE UPDATE ON public.distributor_person FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

-- Add RLS (Row Level Security) policies for tenant isolation
ALTER TABLE public.person ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.tenant_person ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.internal_person ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.customer_person ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.vendor_person ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.distributor_person ENABLE ROW LEVEL SECURITY;

-- Create RLS policies for tenant_person table (main access control)
CREATE POLICY "tenant_person_tenant_isolation" ON public.tenant_person
    FOR ALL USING (
        tenant_id = public.get_current_tenant_id()
    );

-- Create RLS policies for type-specific tables
CREATE POLICY "internal_person_tenant_isolation" ON public.internal_person
    FOR ALL USING (
        tenant_id = public.get_current_tenant_id()
    );

CREATE POLICY "customer_person_tenant_isolation" ON public.customer_person
    FOR ALL USING (
        tenant_id = public.get_current_tenant_id()
    );

CREATE POLICY "vendor_person_tenant_isolation" ON public.vendor_person
    FOR ALL USING (
        tenant_id = public.get_current_tenant_id()
    );

CREATE POLICY "distributor_person_tenant_isolation" ON public.distributor_person
    FOR ALL USING (
        tenant_id = public.get_current_tenant_id()
    );

-- Person table RLS - access through tenant_person relationship
CREATE POLICY "person_tenant_isolation" ON public.person
    FOR ALL USING (
        id IN (
            SELECT person_id FROM public.tenant_person 
            WHERE tenant_id = public.get_current_tenant_id()
        )
    );

-- Grant necessary permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON public.person TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.tenant_person TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.internal_person TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.customer_person TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.vendor_person TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.distributor_person TO authenticated, service_role; 