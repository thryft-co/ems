-- Migration: Create asset management tables
-- This migration creates the asset management system with multi-tenant support
-- PREREQUISITE: Run 000_supabase_setup.sql, 000_create_tenants_table.sql, 101_create_person_tables.sql, and 401_create_item_tables.sql first

-- Create asset_types table (global lookup table)
CREATE TABLE public.asset_types (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name VARCHAR(50) UNIQUE NOT NULL,
  description TEXT,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create assets table (tenant-specific)
CREATE TABLE public.assets (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  item_id UUID NOT NULL REFERENCES public.items(id) ON DELETE CASCADE,
  asset_type_id UUID NOT NULL REFERENCES public.asset_types(id) ON DELETE RESTRICT,
  name VARCHAR(100) NOT NULL,
  version VARCHAR(50),
  description TEXT,
  file_path VARCHAR(500),
  file_size BIGINT CHECK (file_size >= 0),
  file_type VARCHAR(50),
  checksum VARCHAR(64),
  is_active BOOLEAN DEFAULT true,
  metadata JSONB,
  created_by_id UUID NOT NULL REFERENCES public.person(id) ON DELETE RESTRICT,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create firmware_specific table for firmware assets
CREATE TABLE public.firmware_specific (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  asset_id UUID NOT NULL REFERENCES public.assets(id) ON DELETE CASCADE,
  hardware_version VARCHAR(50),
  min_hardware_version VARCHAR(50),
  max_hardware_version VARCHAR(50),
  release_notes TEXT,
  is_beta BOOLEAN DEFAULT false,
  is_critical BOOLEAN DEFAULT false,
  requires_manual_update BOOLEAN DEFAULT false,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(asset_id)
);

-- Create indexes for asset_types table
CREATE INDEX idx_asset_types_name ON public.asset_types(name);

-- Create indexes for assets table
CREATE INDEX idx_assets_tenant_id ON public.assets(tenant_id);
CREATE INDEX idx_assets_item_id ON public.assets(item_id);
CREATE INDEX idx_assets_asset_type_id ON public.assets(asset_type_id);
CREATE INDEX idx_assets_created_by_id ON public.assets(created_by_id);
CREATE INDEX idx_assets_name ON public.assets(name);
CREATE INDEX idx_assets_version ON public.assets(version);
CREATE INDEX idx_assets_file_type ON public.assets(file_type);
CREATE INDEX idx_assets_is_active ON public.assets(is_active);
CREATE INDEX idx_assets_metadata ON public.assets USING GIN (metadata);
CREATE INDEX idx_assets_composite ON public.assets(tenant_id, asset_type_id, is_active);

-- Create indexes for firmware_specific table
CREATE INDEX idx_firmware_specific_asset_id ON public.firmware_specific(asset_id);
CREATE INDEX idx_firmware_specific_hardware_version ON public.firmware_specific(hardware_version);
CREATE INDEX idx_firmware_specific_is_beta ON public.firmware_specific(is_beta);
CREATE INDEX idx_firmware_specific_is_critical ON public.firmware_specific(is_critical);

-- Create triggers for updated_at timestamps (uses function from 000_supabase_setup.sql)
CREATE TRIGGER update_asset_types_updated_at 
  BEFORE UPDATE ON public.asset_types 
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_assets_updated_at 
  BEFORE UPDATE ON public.assets 
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_firmware_specific_updated_at 
  BEFORE UPDATE ON public.firmware_specific 
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

-- Add RLS (Row Level Security) policies for tenant isolation
ALTER TABLE public.assets ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.firmware_specific ENABLE ROW LEVEL SECURITY;

-- Note: asset_types table is global (no RLS needed)

-- Create RLS policy for assets table (tenant isolation)
CREATE POLICY "assets_tenant_isolation" ON public.assets
    FOR ALL USING (
        tenant_id = public.get_current_tenant_id()
    );

-- Create RLS policy for firmware_specific table (through assets relationship)
CREATE POLICY "firmware_specific_tenant_isolation" ON public.firmware_specific
    FOR ALL USING (
        asset_id IN (
            SELECT id FROM public.assets 
            WHERE tenant_id = public.get_current_tenant_id()
        )
    );

-- Insert default asset types
INSERT INTO public.asset_types (name, description) VALUES
('invoice', 'Invoice documents and related files'),
('firmware', 'Firmware files including binary, hex, s19 formats'),
('report', 'Reports and analysis documents'),
('document', 'General documentation and specifications'),
('certificate', 'Certificates and compliance documents');

-- Grant necessary permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON public.asset_types TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.assets TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.firmware_specific TO authenticated, service_role;

-- Add comments for documentation
COMMENT ON TABLE public.asset_types IS 'Global lookup table for asset types';
COMMENT ON TABLE public.assets IS 'Main asset storage table with tenant isolation';
COMMENT ON TABLE public.firmware_specific IS 'Additional metadata for firmware assets';

COMMENT ON COLUMN public.assets.file_size IS 'File size in bytes';
COMMENT ON COLUMN public.assets.checksum IS 'File checksum for integrity verification';
COMMENT ON COLUMN public.assets.metadata IS 'Additional asset metadata in JSON format';
COMMENT ON COLUMN public.firmware_specific.hardware_version IS 'Target hardware version for firmware';
COMMENT ON COLUMN public.firmware_specific.min_hardware_version IS 'Minimum compatible hardware version';
COMMENT ON COLUMN public.firmware_specific.max_hardware_version IS 'Maximum compatible hardware version';
COMMENT ON COLUMN public.firmware_specific.is_beta IS 'Whether this is a beta/preview firmware';
COMMENT ON COLUMN public.firmware_specific.is_critical IS 'Whether this is a critical security update';
COMMENT ON COLUMN public.firmware_specific.requires_manual_update IS 'Whether manual intervention is required for update'; 