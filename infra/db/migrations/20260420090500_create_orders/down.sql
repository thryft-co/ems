DROP TRIGGER IF EXISTS log_order_status_change_trigger ON public.orders;
DROP TRIGGER IF EXISTS update_order_total_trigger ON public.order_items;
DROP TRIGGER IF EXISTS calculate_extended_price_trigger ON public.order_items;

DROP FUNCTION IF EXISTS public.log_order_status_change();
DROP FUNCTION IF EXISTS public.update_order_total();
DROP FUNCTION IF EXISTS public.calculate_order_item_extended_price();

DROP TABLE IF EXISTS public.order_history CASCADE;
DROP TABLE IF EXISTS public.order_items CASCADE;
DROP TABLE IF EXISTS public.orders CASCADE;
