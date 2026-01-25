-- Migration: Create orders and related tables
-- This migration creates the order management system with multi-tenant support
-- PREREQUISITE: Run 000_supabase_setup.sql, 001_create_tenants_table.sql, and 101_create_person_tables.sql first

-- Create orders table
CREATE TABLE public.orders (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  order_number VARCHAR(50) NOT NULL,
  order_type VARCHAR(20) NOT NULL CHECK (order_type IN ('purchase_order', 'customer_order', 'distributor_order')),
  external_entity_id UUID NOT NULL REFERENCES public.person(id),
  external_entity_type VARCHAR(20) NOT NULL CHECK (external_entity_type IN ('vendor', 'customer', 'distributor')),
  order_date TIMESTAMP WITH TIME ZONE NOT NULL,
  total_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
  status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'submitted', 'approved', 'fulfilled', 'partially_fulfilled', 'cancelled', 'paid')),
  created_by_id UUID NOT NULL REFERENCES public.person(id),
  notes TEXT,
  metadata JSONB DEFAULT '{}',
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  -- order_number should be unique per tenant, not globally
  UNIQUE(tenant_id, order_number)
);

-- Create indexes for orders table
CREATE INDEX idx_orders_tenant_id ON public.orders(tenant_id);
CREATE INDEX idx_orders_order_number ON public.orders(order_number);
CREATE INDEX idx_orders_external_entity_id ON public.orders(external_entity_id);
CREATE INDEX idx_orders_order_type ON public.orders(order_type);
CREATE INDEX idx_orders_status ON public.orders(status);
CREATE INDEX idx_orders_created_by_id ON public.orders(created_by_id);
CREATE INDEX idx_orders_order_date ON public.orders(order_date);
CREATE INDEX idx_orders_created_at ON public.orders(created_at);

-- Create order_items table
CREATE TABLE public.order_items (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  order_id UUID NOT NULL REFERENCES public.orders(id) ON DELETE CASCADE,
  item_id UUID, -- Nullable for now since items table may not exist yet
  item_name VARCHAR(200) NOT NULL,
  item_description TEXT,
  quantity INTEGER NOT NULL CHECK (quantity > 0),
  unit_price DOUBLE PRECISION NOT NULL CHECK (unit_price >= 0),
  extended_price DOUBLE PRECISION NOT NULL CHECK (extended_price >= 0),
  notes TEXT,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for order_items table
CREATE INDEX idx_order_items_order_id ON public.order_items(order_id);
CREATE INDEX idx_order_items_item_id ON public.order_items(item_id);
CREATE INDEX idx_order_items_item_name ON public.order_items(item_name);

-- Add trigger to automatically calculate extended_price
CREATE OR REPLACE FUNCTION public.calculate_order_item_extended_price()
RETURNS TRIGGER
SECURITY DEFINER
SET search_path = public
AS $$
BEGIN
    NEW.extended_price = NEW.quantity * NEW.unit_price;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER calculate_extended_price_trigger
    BEFORE INSERT OR UPDATE OF quantity, unit_price ON public.order_items
    FOR EACH ROW
    EXECUTE FUNCTION public.calculate_order_item_extended_price();

-- Add trigger to automatically update order total_amount
CREATE OR REPLACE FUNCTION public.update_order_total()
RETURNS TRIGGER
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
    new_total DOUBLE PRECISION;
BEGIN
    -- Calculate total from all order items
    SELECT COALESCE(SUM(extended_price), 0) INTO new_total
    FROM public.order_items
    WHERE order_id = COALESCE(NEW.order_id, OLD.order_id);
    
    -- Update the order total
    UPDATE public.orders
    SET total_amount = new_total
    WHERE id = COALESCE(NEW.order_id, OLD.order_id);
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_order_total_trigger
    AFTER INSERT OR UPDATE OR DELETE ON public.order_items
    FOR EACH ROW
    EXECUTE FUNCTION public.update_order_total();

-- Create order_history table for tracking status changes and updates
CREATE TABLE public.order_history (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  order_id UUID NOT NULL REFERENCES public.orders(id) ON DELETE CASCADE,
  tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
  person_id UUID REFERENCES public.person(id),
  action VARCHAR(50) NOT NULL,
  previous_status VARCHAR(20),
  new_status VARCHAR(20),
  notes TEXT,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for order_history table
CREATE INDEX idx_order_history_order_id ON public.order_history(order_id);
CREATE INDEX idx_order_history_tenant_id ON public.order_history(tenant_id);
CREATE INDEX idx_order_history_person_id ON public.order_history(person_id);
CREATE INDEX idx_order_history_action ON public.order_history(action);
CREATE INDEX idx_order_history_created_at ON public.order_history(created_at);

-- Create triggers for updated_at timestamps (uses function from 000_supabase_setup.sql)
CREATE TRIGGER update_orders_updated_at BEFORE UPDATE ON public.orders FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();
CREATE TRIGGER update_order_items_updated_at BEFORE UPDATE ON public.order_items FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

-- Add RLS (Row Level Security) policies for tenant isolation
ALTER TABLE public.orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.order_items ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.order_history ENABLE ROW LEVEL SECURITY;

-- Create RLS policies for orders table
CREATE POLICY "orders_tenant_isolation" ON public.orders
    FOR ALL USING (
        tenant_id = public.get_current_tenant_id()
    );

-- Create RLS policies for order_items table (inherits tenant from orders)
CREATE POLICY "order_items_tenant_isolation" ON public.order_items
    FOR ALL USING (
        order_id IN (
            SELECT id FROM public.orders
            WHERE tenant_id = public.get_current_tenant_id()
        )
    );

-- Create RLS policies for order_history table
CREATE POLICY "order_history_tenant_isolation" ON public.order_history
    FOR ALL USING (
        tenant_id = public.get_current_tenant_id()
    );

-- Create trigger function for order status change logging
CREATE OR REPLACE FUNCTION public.log_order_status_change()
RETURNS TRIGGER
SECURITY DEFINER
SET search_path = public
AS $$
BEGIN
    -- Only log if status has changed
    IF OLD.status IS DISTINCT FROM NEW.status THEN
        INSERT INTO public.order_history (order_id, tenant_id, action, previous_status, new_status, notes)
        VALUES (NEW.id, NEW.tenant_id, 'status_change', OLD.status, NEW.status, 'Automatic status change log');
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for automatic order status change logging
CREATE TRIGGER log_order_status_change_trigger
    AFTER UPDATE ON public.orders
    FOR EACH ROW
    EXECUTE FUNCTION public.log_order_status_change();

-- Grant necessary permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON public.orders TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.order_items TO authenticated, service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON public.order_history TO authenticated, service_role;
GRANT EXECUTE ON FUNCTION public.log_order_status_change() TO postgres, service_role;
GRANT EXECUTE ON FUNCTION public.calculate_order_item_extended_price() TO postgres, service_role, authenticated;
GRANT EXECUTE ON FUNCTION public.update_order_total() TO postgres, service_role, authenticated;