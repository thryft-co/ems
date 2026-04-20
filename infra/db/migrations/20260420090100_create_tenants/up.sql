-- Migration: Create tenants table
-- This migration creates the multi-tenancy infrastructure
-- PREREQUISITE: Run 000_supabase_setup.sql first

-- Create tenants table
CREATE TABLE public.tenants (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name VARCHAR(100) NOT NULL,
  subdomain VARCHAR(50) UNIQUE NOT NULL,
  database_url VARCHAR(500),
  settings JSONB DEFAULT '{}',
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for tenants table
CREATE INDEX idx_tenants_subdomain ON public.tenants(subdomain);
CREATE INDEX idx_tenants_is_active ON public.tenants(is_active);
CREATE INDEX idx_tenants_name ON public.tenants(name);

-- Create trigger for updated_at timestamp (uses function from 000_supabase_setup.sql)
CREATE TRIGGER update_tenants_updated_at
    BEFORE UPDATE ON public.tenants
    FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

-- Add RLS (Row Level Security) policies for tenant isolation
ALTER TABLE public.tenants ENABLE ROW LEVEL SECURITY;

-- Modified RLS policy to handle NULL tenant context gracefully
-- Service role can see all tenants, authenticated users see their tenant
CREATE POLICY "tenant_isolation" ON public.tenants
    FOR ALL
    USING (
        -- Allow service_role to see all tenants
        (auth.jwt() ->> 'role' = 'service_role')
        OR
        -- Allow authenticated users to see their tenant
        (id = public.get_current_tenant_id())
    );

-- Add policy for INSERT operations (needed for tenant creation)
CREATE POLICY "tenant_insert" ON public.tenants
    FOR INSERT
    WITH CHECK (
        auth.jwt() ->> 'role' = 'service_role'
    );

-- Grant necessary permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON public.tenants TO authenticated, service_role;

-- Insert a default tenant for development/testing
INSERT INTO public.tenants (name, subdomain, is_active)
VALUES ('Default Tenant', 'default', true)
ON CONFLICT (subdomain) DO NOTHING; -- Add conflict handling

-- COMMENT for documentation
COMMENT ON TABLE public.tenants IS 'Multi-tenant organization table with RLS isolation';
COMMENT ON COLUMN public.tenants.subdomain IS 'Unique subdomain identifier for tenant routing';
COMMENT ON COLUMN public.tenants.database_url IS 'Optional separate database URL for tenant data isolation';