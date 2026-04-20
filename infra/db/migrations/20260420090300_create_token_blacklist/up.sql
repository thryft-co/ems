-- Migration: Create token blacklist table for logout functionality
-- This migration creates a table to store blacklisted tokens for secure logout
-- PREREQUISITE: Run 000_supabase_setup.sql, 001_create_tenants_table.sql, and 101_create_person_tables.sql first

-- Create token_blacklist table
CREATE TABLE public.token_blacklist (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  token_hash VARCHAR(255) NOT NULL UNIQUE, -- Hash of the token for security
  token_type VARCHAR(20) NOT NULL CHECK (token_type IN ('access', 'refresh')),
  person_id UUID NOT NULL REFERENCES public.person(id) ON DELETE CASCADE,
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
  blacklisted_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for token_blacklist table
CREATE INDEX idx_token_blacklist_token_hash ON public.token_blacklist(token_hash);
CREATE INDEX idx_token_blacklist_person_id ON public.token_blacklist(person_id);
CREATE INDEX idx_token_blacklist_tenant_id ON public.token_blacklist(tenant_id);
CREATE INDEX idx_token_blacklist_expires_at ON public.token_blacklist(expires_at);
CREATE INDEX idx_token_blacklist_type ON public.token_blacklist(token_type);

-- Add RLS (Row Level Security) for tenant isolation
ALTER TABLE public.token_blacklist ENABLE ROW LEVEL SECURITY;

-- Create RLS policy for token blacklist
CREATE POLICY "token_blacklist_tenant_isolation" ON public.token_blacklist
    FOR ALL USING (
        tenant_id = public.get_current_tenant_id()
    );

-- Create function to automatically clean up expired tokens
CREATE OR REPLACE FUNCTION public.cleanup_expired_tokens()
RETURNS INTEGER 
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM public.token_blacklist 
    WHERE expires_at < NOW();
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Grant execute permissions on the cleanup function
GRANT EXECUTE ON FUNCTION public.cleanup_expired_tokens() TO postgres, service_role;

-- Grant necessary permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON public.token_blacklist TO authenticated, service_role;

-- Create a scheduled job to run cleanup (this would typically be done via pg_cron or external scheduler)
-- For now, just create the function that can be called manually or via cron
COMMENT ON FUNCTION public.cleanup_expired_tokens() IS 'Clean up expired tokens from blacklist. Should be run periodically via cron job.'; 