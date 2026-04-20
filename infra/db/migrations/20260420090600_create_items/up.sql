-- Migration: Create item tables
-- This migration creates the item management system

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create base items table (template for single item)
CREATE TABLE public.items (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  internal_part_number VARCHAR(50) UNIQUE NOT NULL,
  mfr_part_number VARCHAR(50),
  manufacturer VARCHAR(100) NOT NULL,
  datasheet VARCHAR(500),
  lifecycle VARCHAR(20) DEFAULT 'production' CHECK (lifecycle IN ('production', 'prototype', 'obsolete', 'nrfnd')),
  description TEXT,
  category VARCHAR(50),
  metadata JSONB DEFAULT '{}',
  linked_resources JSONB DEFAULT '[]',
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_items_internal_part_number ON public.items(internal_part_number);
CREATE INDEX idx_items_mfr_part_number ON public.items(mfr_part_number);
CREATE INDEX idx_items_manufacturer ON public.items(manufacturer);
CREATE INDEX idx_items_category ON public.items(category);

-- Create inventory_items table for contextual inventory tracking
CREATE TABLE public.inventory_items (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  item_id UUID NOT NULL REFERENCES public.items(id) ON DELETE CASCADE,
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  context VARCHAR(20) NOT NULL CHECK (context IN ('finished_goods', 'store', 'vendor')),
  quantity INTEGER DEFAULT 0 CHECK (quantity >= 0),
  location VARCHAR(100),
  pricing JSONB DEFAULT '{}',
  lead_time INTEGER,
  min_stock_level INTEGER DEFAULT 0,
  max_stock_level INTEGER,
  reorder_point INTEGER,
  vendor_id UUID REFERENCES public.person(id),
  last_received_date TIMESTAMP WITH TIME ZONE,
  status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'discontinued')),
  notes TEXT,
  metadata JSONB DEFAULT '{}',
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(item_id, tenant_id, context)
);

CREATE INDEX idx_inventory_items_item_id ON public.inventory_items(item_id);
CREATE INDEX idx_inventory_items_tenant_id ON public.inventory_items(tenant_id);
CREATE INDEX idx_inventory_items_context ON public.inventory_items(context);
CREATE INDEX idx_inventory_items_vendor_id ON public.inventory_items(vendor_id);

-- Create item_bom table for BOM relationships
CREATE TABLE public.item_bom (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  parent_item_id UUID NOT NULL REFERENCES public.items(id) ON DELETE CASCADE,
  component_item_id UUID NOT NULL REFERENCES public.items(id) ON DELETE CASCADE,
  quantity INTEGER DEFAULT 1 CHECK (quantity > 0),
  notes TEXT,
  is_optional BOOLEAN DEFAULT false,
  substitutes UUID[] DEFAULT ARRAY[]::UUID[],
  assembly_order INTEGER,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(tenant_id, parent_item_id, component_item_id)
);

CREATE INDEX idx_item_bom_tenant_id ON public.item_bom(tenant_id);
CREATE INDEX idx_item_bom_parent_item_id ON public.item_bom(parent_item_id);
CREATE INDEX idx_item_bom_component_item_id ON public.item_bom(component_item_id);

-- Create triggers for updated_at timestamps (reuse function from previous migrations)
CREATE TRIGGER update_items_updated_at
  BEFORE UPDATE ON public.items
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_inventory_items_updated_at
  BEFORE UPDATE ON public.inventory_items
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_item_bom_updated_at
  BEFORE UPDATE ON public.item_bom
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

-- Add RLS (Row Level Security) policies for tenant isolation
ALTER TABLE public.inventory_items ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.item_bom ENABLE ROW LEVEL SECURITY;

-- Note: items table is global (no tenant isolation needed)

-- Create RLS policy for inventory_items table (tenant isolation)
CREATE POLICY "inventory_items_tenant_isolation" ON public.inventory_items
    FOR ALL USING (
        tenant_id = (current_setting('app.current_tenant_id', true))::uuid
    );

-- Create RLS policy for item_bom table (tenant isolation)
CREATE POLICY "item_bom_tenant_isolation" ON public.item_bom
    FOR ALL USING (
        tenant_id = (current_setting('app.current_tenant_id', true))::uuid
    );