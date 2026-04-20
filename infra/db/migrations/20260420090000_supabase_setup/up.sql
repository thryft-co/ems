-- Migration: Supabase PostgreSQL Setup
-- This migration ensures Supabase compatibility and sets up required infrastructure
-- IMPORTANT: This should be run FIRST, before all other migrations

-- Create public schema if it doesn't exist (required for Supabase)
CREATE SCHEMA IF NOT EXISTS public;

-- Grant necessary permissions on public schema
GRANT USAGE ON SCHEMA public TO postgres, anon, authenticated, service_role;
GRANT ALL ON SCHEMA public TO postgres, service_role;
GRANT USAGE ON SCHEMA public TO anon, authenticated;

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp" SCHEMA public;
CREATE EXTENSION IF NOT EXISTS "pgcrypto" SCHEMA public; -- For additional crypto functions if needed

-- Create reusable trigger function for updating updated_at timestamps
-- This function will be used by multiple tables across different migrations
CREATE OR REPLACE FUNCTION public.update_updated_at_column()
RETURNS TRIGGER 
SECURITY DEFINER 
SET search_path = public
AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Grant execute permissions on the function
GRANT EXECUTE ON FUNCTION public.update_updated_at_column() TO postgres, service_role;

-- Create a function to safely set tenant context (for RLS policies)
CREATE OR REPLACE FUNCTION public.set_tenant_context(tenant_uuid UUID)
RETURNS void
SECURITY DEFINER
SET search_path = public
AS $$
BEGIN
    PERFORM set_config('app.current_tenant_id', tenant_uuid::text, false);
END;
$$ LANGUAGE plpgsql;

-- Grant execute permissions on the tenant context function
GRANT EXECUTE ON FUNCTION public.set_tenant_context(UUID) TO postgres, service_role, authenticated;

-- Create a function to get current tenant context
CREATE OR REPLACE FUNCTION public.get_current_tenant_id()
RETURNS UUID
SECURITY DEFINER
SET search_path = public
AS $$
BEGIN
    RETURN (current_setting('app.current_tenant_id', true))::uuid;
EXCEPTION
    WHEN OTHERS THEN
        RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Grant execute permissions on the get tenant function
GRANT EXECUTE ON FUNCTION public.get_current_tenant_id() TO postgres, service_role, authenticated;

-- Add comments for documentation
COMMENT ON SCHEMA public IS 'Main application schema for multi-tenant EMS system';
COMMENT ON FUNCTION public.update_updated_at_column() IS 'Trigger function to automatically update updated_at timestamps';
COMMENT ON FUNCTION public.set_tenant_context(UUID) IS 'Set the current tenant context for RLS policies';
COMMENT ON FUNCTION public.get_current_tenant_id() IS 'Get the current tenant ID from session context'; 