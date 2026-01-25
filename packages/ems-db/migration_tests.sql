-- ============================================================================
-- COMPREHENSIVE MIGRATION TEST SUITE FOR SUPABASE
-- ============================================================================
-- This test suite validates all migrations and their interactions
-- Run each section in order and verify results
-- ============================================================================

-- ============================================================================
-- SECTION 1: PRE-MIGRATION VALIDATION
-- ============================================================================

-- Test 1.1: Verify Supabase environment
SELECT
    current_database() as database_name,
    current_user as current_user,
    version() as postgres_version;

-- Test 1.2: Check for existing tables (should be empty before migrations)
SELECT table_name
FROM information_schema.tables
WHERE table_schema = 'public'
  AND table_type = 'BASE TABLE'
ORDER BY table_name;

-- ============================================================================
-- SECTION 2: MIGRATION 000 - SUPABASE SETUP TESTS
-- ============================================================================

-- Test 2.1: Verify extensions are installed
SELECT extname, extversion
FROM pg_extension
WHERE extname IN ('uuid-ossp', 'pgcrypto');

-- Test 2.2: Verify helper functions exist
SELECT routine_name, routine_type
FROM information_schema.routines
WHERE routine_schema = 'public'
  AND routine_name IN ('update_updated_at_column', 'set_tenant_context', 'get_current_tenant_id')
ORDER BY routine_name;

-- Test 2.3: Test tenant context functions
DO $$
DECLARE
    test_uuid UUID := 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee';
    retrieved_uuid UUID;
BEGIN
    -- Set tenant context
    PERFORM public.set_tenant_context(test_uuid);

    -- Retrieve tenant context
    SELECT public.get_current_tenant_id() INTO retrieved_uuid;

    -- Verify
    IF retrieved_uuid = test_uuid THEN
        RAISE NOTICE 'PASS: Tenant context functions working correctly';
    ELSE
        RAISE EXCEPTION 'FAIL: Tenant context mismatch. Expected %, got %', test_uuid, retrieved_uuid;
    END IF;
END $$;

-- ============================================================================
-- SECTION 3: MIGRATION 001 - TENANTS TABLE TESTS
-- ============================================================================

-- Test 3.1: Verify tenants table structure
SELECT column_name, data_type, is_nullable, column_default
FROM information_schema.columns
WHERE table_schema = 'public' AND table_name = 'tenants'
ORDER BY ordinal_position;

-- Test 3.2: Verify indexes exist
SELECT indexname, indexdef
FROM pg_indexes
WHERE schemaname = 'public' AND tablename = 'tenants'
ORDER BY indexname;

-- Test 3.3: Verify RLS is enabled
SELECT tablename, rowsecurity
FROM pg_tables
WHERE schemaname = 'public' AND tablename = 'tenants';

-- Test 3.4: Insert test tenant and verify default tenant exists
INSERT INTO public.tenants (name, subdomain, is_active)
VALUES ('Test Tenant', 'test', true)
RETURNING id, name, subdomain, created_at;

SELECT id, name, subdomain, is_active, created_at
FROM public.tenants
ORDER BY created_at;

-- Test 3.5: Test subdomain uniqueness constraint
DO $$
BEGIN
    INSERT INTO public.tenants (name, subdomain, is_active)
    VALUES ('Duplicate Tenant', 'test', true);
    RAISE EXCEPTION 'FAIL: Subdomain uniqueness constraint not working';
EXCEPTION
    WHEN unique_violation THEN
        RAISE NOTICE 'PASS: Subdomain uniqueness constraint working';
END $$;

-- Test 3.6: Test updated_at trigger
DO $$
DECLARE
    tenant_id UUID;
    initial_updated_at TIMESTAMP WITH TIME ZONE;
    new_updated_at TIMESTAMP WITH TIME ZONE;
BEGIN
    -- Get a tenant ID
    SELECT id INTO tenant_id FROM public.tenants LIMIT 1;

    -- Get initial updated_at
    SELECT updated_at INTO initial_updated_at FROM public.tenants WHERE id = tenant_id;

    -- Wait a moment
    PERFORM pg_sleep(0.1);

    -- Update the tenant
    UPDATE public.tenants SET name = 'Updated Name' WHERE id = tenant_id;

    -- Get new updated_at
    SELECT updated_at INTO new_updated_at FROM public.tenants WHERE id = tenant_id;

    IF new_updated_at > initial_updated_at THEN
        RAISE NOTICE 'PASS: updated_at trigger working';
    ELSE
        RAISE EXCEPTION 'FAIL: updated_at not updated. Initial: %, New: %', initial_updated_at, new_updated_at;
    END IF;
END $$;

-- ============================================================================
-- SECTION 4: MIGRATION 101 - PERSON TABLES TESTS
-- ============================================================================

-- Test 4.1: Verify all person tables exist
SELECT table_name
FROM information_schema.tables
WHERE table_schema = 'public'
  AND table_name IN ('person', 'tenant_person', 'internal_person', 'customer_person', 'vendor_person', 'distributor_person')
ORDER BY table_name;

-- Test 4.2: Create test persons with different roles
DO $$
DECLARE
    tenant_id UUID;
    person1_id UUID;
    person2_id UUID;
    person3_id UUID;
BEGIN
    -- Get default tenant
    SELECT id INTO tenant_id FROM public.tenants WHERE subdomain = 'default';

    -- Set tenant context for RLS
    PERFORM public.set_tenant_context(tenant_id);

    -- Create person 1 (internal)
    INSERT INTO public.person (supabase_uid, name, email, phone, is_active)
    VALUES (uuid_generate_v4(), 'John Doe', 'john@example.com', '1234567890', true)
    RETURNING id INTO person1_id;

    -- Create tenant_person relationship for internal
    INSERT INTO public.tenant_person (person_id, tenant_id, role, is_primary)
    VALUES (person1_id, tenant_id, 'internal', true);

    -- Create internal_person details
    INSERT INTO public.internal_person (person_id, tenant_id, department, position, employee_id)
    VALUES (person1_id, tenant_id, 'Engineering', 'Software Engineer', 'EMP001');

    -- Create person 2 (customer)
    INSERT INTO public.person (supabase_uid, name, email, phone, is_active)
    VALUES (uuid_generate_v4(), 'Jane Smith', 'jane@customer.com', '0987654321', true)
    RETURNING id INTO person2_id;

    INSERT INTO public.tenant_person (person_id, tenant_id, role, is_primary)
    VALUES (person2_id, tenant_id, 'customer', true);

    INSERT INTO public.customer_person (person_id, tenant_id, company, industry)
    VALUES (person2_id, tenant_id, 'Acme Corp', 'Technology');

    -- Create person 3 (vendor)
    INSERT INTO public.person (supabase_uid, name, email, phone, is_active)
    VALUES (uuid_generate_v4(), 'Bob Johnson', 'bob@vendor.com', '5555555555', true)
    RETURNING id INTO person3_id;

    INSERT INTO public.tenant_person (person_id, tenant_id, role, is_primary)
    VALUES (person3_id, tenant_id, 'vendor', true);

    INSERT INTO public.vendor_person (person_id, tenant_id, company, service_type)
    VALUES (person3_id, tenant_id, 'Vendor Inc', 'Manufacturing');

    RAISE NOTICE 'PASS: Created test persons with different roles';
END $$;

-- Test 4.3: Verify person data with joins
SELECT
    p.id,
    p.name,
    p.email,
    tp.role,
    tp.is_primary,
    COALESCE(ip.department, cp.company, vp.company) as org_info
FROM public.person p
JOIN public.tenant_person tp ON p.id = tp.person_id
LEFT JOIN public.internal_person ip ON p.id = ip.person_id
LEFT JOIN public.customer_person cp ON p.id = cp.person_id
LEFT JOIN public.vendor_person vp ON p.id = vp.person_id
ORDER BY p.created_at;

-- Test 4.4: Test email uniqueness
DO $$
BEGIN
    INSERT INTO public.person (supabase_uid, name, email)
    VALUES (uuid_generate_v4(), 'Duplicate', 'john@example.com');
    RAISE EXCEPTION 'FAIL: Email uniqueness constraint not working';
EXCEPTION
    WHEN unique_violation THEN
        RAISE NOTICE 'PASS: Email uniqueness constraint working';
END $$;

-- Test 4.5: Test multi-role support (person can have multiple roles)
DO $$
DECLARE
    tenant_id UUID;
    person_id UUID;
BEGIN
    SELECT id INTO tenant_id FROM public.tenants WHERE subdomain = 'default';
    SELECT id INTO person_id FROM public.person WHERE email = 'john@example.com';

    -- Add customer role to internal person
    INSERT INTO public.tenant_person (person_id, tenant_id, role)
    VALUES (person_id, tenant_id, 'customer');

    INSERT INTO public.customer_person (person_id, tenant_id, company)
    VALUES (person_id, tenant_id, 'Johns Consulting');

    RAISE NOTICE 'PASS: Multi-role support working';
END $$;

-- Verify multi-role
SELECT
    p.name,
    array_agg(tp.role) as roles
FROM public.person p
JOIN public.tenant_person tp ON p.id = tp.person_id
WHERE p.email = 'john@example.com'
GROUP BY p.name;

-- ============================================================================
-- SECTION 5: MIGRATION 102 - TOKEN BLACKLIST TESTS
-- ============================================================================

-- Test 5.1: Verify token_blacklist table
SELECT column_name, data_type
FROM information_schema.columns
WHERE table_schema = 'public' AND table_name = 'token_blacklist'
ORDER BY ordinal_position;

-- Test 5.2: Insert and query blacklisted tokens
DO $$
DECLARE
    tenant_id UUID;
    person_id UUID;
    token_id UUID;
BEGIN
    SELECT id INTO tenant_id FROM public.tenants WHERE subdomain = 'default';
    SELECT id INTO person_id FROM public.person WHERE email = 'john@example.com';

    PERFORM public.set_tenant_context(tenant_id);

    -- Blacklist a token
    INSERT INTO public.token_blacklist (token_hash, token_type, person_id, tenant_id, expires_at)
    VALUES (
        'hash_' || md5(random()::text),
        'access',
        person_id,
        tenant_id,
        NOW() + INTERVAL '1 hour'
    )
    RETURNING id INTO token_id;

    RAISE NOTICE 'PASS: Token blacklisted successfully with id: %', token_id;
END $$;

-- Test 5.3: Test cleanup function
SELECT public.cleanup_expired_tokens() as deleted_count;

-- ============================================================================
-- SECTION 6: MIGRATION 201 - JOBS TABLES TESTS
-- ============================================================================

-- Test 6.1: Create test jobs
DO $$
DECLARE
    tenant_id UUID;
    person_id UUID;
    customer_id UUID;
    mfg_job_id UUID;
    qa_job_id UUID;
    svc_job_id UUID;
BEGIN
    SELECT id INTO tenant_id FROM public.tenants WHERE subdomain = 'default';
    SELECT id INTO person_id FROM public.person WHERE email = 'john@example.com';
    SELECT id INTO customer_id FROM public.person WHERE email = 'jane@customer.com';

    PERFORM public.set_tenant_context(tenant_id);

    -- Create manufacturing job
    INSERT INTO public.jobs (
        tenant_id, job_number, quantity, assigned_person_id, customer_id,
        job_type, priority, status, due_date
    )
    VALUES (
        tenant_id, 'MFG-001', 100, person_id, customer_id,
        'manufacturing', 'high', 'pending', NOW() + INTERVAL '7 days'
    )
    RETURNING id INTO mfg_job_id;

    INSERT INTO public.manufacturing_job (
        job_id, tenant_id, work_order_number, production_line, batch_size
    )
    VALUES (
        mfg_job_id, tenant_id, 'WO-001', 'Line A', 100
    );

    -- Create QA job
    INSERT INTO public.jobs (
        tenant_id, job_number, quantity, assigned_person_id,
        job_type, priority, status
    )
    VALUES (
        tenant_id, 'QA-001', 10, person_id,
        'qa', 'urgent', 'pending'
    )
    RETURNING id INTO qa_job_id;

    INSERT INTO public.qa_job (
        job_id, tenant_id, inspection_type, sampling_size
    )
    VALUES (
        qa_job_id, tenant_id, 'Final Inspection', 10
    );

    -- Create service job
    INSERT INTO public.jobs (
        tenant_id, job_number, quantity, assigned_person_id,
        job_type, status
    )
    VALUES (
        tenant_id, 'SVC-001', 1, person_id,
        'service', 'pending'
    )
    RETURNING id INTO svc_job_id;

    INSERT INTO public.service_job (
        job_id, tenant_id, service_type, location
    )
    VALUES (
        svc_job_id, tenant_id, 'Maintenance', 'Customer Site A'
    );

    RAISE NOTICE 'PASS: Created test jobs (mfg: %, qa: %, svc: %)', mfg_job_id, qa_job_id, svc_job_id;
END $$;

-- Test 6.2: Verify jobs with type-specific data
SELECT
    j.job_number,
    j.job_type,
    j.status,
    j.priority,
    CASE
        WHEN j.job_type = 'manufacturing' THEN mj.production_line
        WHEN j.job_type = 'qa' THEN qj.inspection_type
        WHEN j.job_type = 'service' THEN sj.service_type
    END as type_detail
FROM public.jobs j
LEFT JOIN public.manufacturing_job mj ON j.id = mj.job_id
LEFT JOIN public.qa_job qj ON j.id = qj.job_id
LEFT JOIN public.service_job sj ON j.id = sj.job_id
ORDER BY j.created_at;

-- Test 6.3: Test job status change logging
DO $$
DECLARE
    job_id UUID;
    history_count INTEGER;
BEGIN
    SELECT id INTO job_id FROM public.jobs WHERE job_number = 'MFG-001';

    -- Update status
    UPDATE public.jobs SET status = 'in_progress' WHERE id = job_id;

    -- Check history
    SELECT COUNT(*) INTO history_count
    FROM public.job_history
    WHERE job_id = job_id AND action = 'status_change';

    IF history_count > 0 THEN
        RAISE NOTICE 'PASS: Job status change logged (% entries)', history_count;
    ELSE
        RAISE EXCEPTION 'FAIL: Job status change not logged';
    END IF;
END $$;

-- View job history
SELECT
    jh.action,
    jh.previous_status,
    jh.new_status,
    j.job_number,
    jh.created_at
FROM public.job_history jh
JOIN public.jobs j ON jh.job_id = j.id
ORDER BY jh.created_at DESC;

-- ============================================================================
-- SECTION 7: MIGRATION 301 - ORDERS TABLES TESTS
-- ============================================================================

-- Test 7.1: Create test orders
DO $$
DECLARE
    tenant_id UUID;
    person_id UUID;
    vendor_id UUID;
    customer_id UUID;
    po_id UUID;
    co_id UUID;
BEGIN
    SELECT id INTO tenant_id FROM public.tenants WHERE subdomain = 'default';
    SELECT id INTO person_id FROM public.person WHERE email = 'john@example.com';
    SELECT id INTO vendor_id FROM public.person WHERE email = 'bob@vendor.com';
    SELECT id INTO customer_id FROM public.person WHERE email = 'jane@customer.com';

    PERFORM public.set_tenant_context(tenant_id);

    -- Create Purchase Order
    INSERT INTO public.orders (
        tenant_id, order_number, order_type, external_entity_id,
        external_entity_type, order_date, created_by_id, status
    )
    VALUES (
        tenant_id, 'PO-001', 'purchase_order', vendor_id,
        'vendor', NOW(), person_id, 'draft'
    )
    RETURNING id INTO po_id;

    -- Add order items
    INSERT INTO public.order_items (order_id, item_name, quantity, unit_price)
    VALUES
        (po_id, 'Widget A', 100, 10.50),
        (po_id, 'Widget B', 50, 25.00);

    -- Create Customer Order
    INSERT INTO public.orders (
        tenant_id, order_number, order_type, external_entity_id,
        external_entity_type, order_date, created_by_id, status
    )
    VALUES (
        tenant_id, 'CO-001', 'customer_order', customer_id,
        'customer', NOW(), person_id, 'submitted'
    )
    RETURNING id INTO co_id;

    INSERT INTO public.order_items (order_id, item_name, quantity, unit_price)
    VALUES
        (co_id, 'Product X', 25, 100.00),
        (co_id, 'Product Y', 10, 150.00);

    RAISE NOTICE 'PASS: Created test orders (PO: %, CO: %)', po_id, co_id;
END $$;

-- Test 7.2: Verify order totals are calculated
SELECT
    o.order_number,
    o.order_type,
    o.total_amount,
    COUNT(oi.id) as item_count,
    SUM(oi.extended_price) as calculated_total
FROM public.orders o
LEFT JOIN public.order_items oi ON o.id = oi.order_id
GROUP BY o.id, o.order_number, o.order_type, o.total_amount
ORDER BY o.created_at;

-- Test 7.3: Test extended price calculation
DO $$
DECLARE
    order_id UUID;
    item_id UUID;
    expected_extended DOUBLE PRECISION;
    actual_extended DOUBLE PRECISION;
BEGIN
    SELECT id INTO order_id FROM public.orders WHERE order_number = 'PO-001';

    -- Insert item with specific price
    INSERT INTO public.order_items (order_id, item_name, quantity, unit_price)
    VALUES (order_id, 'Test Item', 10, 5.50)
    RETURNING id, extended_price INTO item_id, actual_extended;

    expected_extended := 10 * 5.50;

    IF actual_extended = expected_extended THEN
        RAISE NOTICE 'PASS: Extended price calculated correctly: %', actual_extended;
    ELSE
        RAISE EXCEPTION 'FAIL: Extended price mismatch. Expected: %, Got: %', expected_extended, actual_extended;
    END IF;
END $$;

-- Test 7.4: Test order status change logging
DO $$
DECLARE
    order_id UUID;
BEGIN
    SELECT id INTO order_id FROM public.orders WHERE order_number = 'CO-001';

    UPDATE public.orders SET status = 'approved' WHERE id = order_id;
    UPDATE public.orders SET status = 'fulfilled' WHERE id = order_id;
END $$;

-- View order history
SELECT
    oh.action,
    oh.previous_status,
    oh.new_status,
    o.order_number,
    oh.created_at
FROM public.order_history oh
JOIN public.orders o ON oh.order_id = o.id
ORDER BY oh.created_at;

-- ============================================================================
-- SECTION 8: MIGRATION 401 - ITEMS TABLES TESTS
-- ============================================================================

-- Test 8.1: Create test items
DO $$
DECLARE
    tenant_id UUID;
    item1_id UUID;
    item2_id UUID;
    component_id UUID;
BEGIN
    SELECT id INTO tenant_id FROM public.tenants WHERE subdomain = 'default';
    PERFORM public.set_tenant_context(tenant_id);

    -- Create parent item
    INSERT INTO public.items (
        internal_part_number, mfr_part_number, manufacturer,
        description, category, lifecycle
    )
    VALUES (
        'IPN-001', 'MPN-001', 'Test Manufacturer',
        'Test Widget Assembly', 'Electronics', 'production'
    )
    RETURNING id INTO item1_id;

    -- Create component item
    INSERT INTO public.items (
        internal_part_number, manufacturer, description, category
    )
    VALUES (
        'IPN-002', 'Component Mfg', 'Test Component', 'Parts'
    )
    RETURNING id INTO component_id;

    -- Create another item
    INSERT INTO public.items (
        internal_part_number, manufacturer, description
    )
    VALUES (
        'IPN-003', 'Test Mfg', 'Another Widget'
    )
    RETURNING id INTO item2_id;

    -- Create BOM relationship
    INSERT INTO public.item_bom (
        tenant_id, parent_item_id, component_item_id, quantity
    )
    VALUES (
        tenant_id, item1_id, component_id, 5
    );

    -- Create inventory records
    INSERT INTO public.inventory_items (
        item_id, tenant_id, context, quantity, location
    )
    VALUES
        (item1_id, tenant_id, 'finished_goods', 100, 'Warehouse A'),
        (component_id, tenant_id, 'store', 500, 'Parts Bin 1'),
        (item2_id, tenant_id, 'store', 50, 'Warehouse B');

    RAISE NOTICE 'PASS: Created test items and inventory';
END $$;

-- Test 8.2: Query items with BOM and inventory
SELECT
    i.internal_part_number,
    i.description,
    i.lifecycle,
    inv.context,
    inv.quantity,
    inv.location
FROM public.items i
LEFT JOIN public.inventory_items inv ON i.id = inv.item_id
ORDER BY i.internal_part_number;

-- Test 8.3: Query BOM hierarchy
SELECT
    parent.internal_part_number as parent_part,
    parent.description as parent_desc,
    component.internal_part_number as component_part,
    component.description as component_desc,
    bom.quantity as qty_required
FROM public.item_bom bom
JOIN public.items parent ON bom.parent_item_id = parent.id
JOIN public.items component ON bom.component_item_id = component.id;

-- ============================================================================
-- SECTION 9: MIGRATION 402 - ASSET TABLES TESTS
-- ============================================================================

-- Test 9.1: Verify asset types exist
SELECT id, name, description FROM public.asset_types ORDER BY name;

-- Test 9.2: Create test assets
DO $$
DECLARE
    tenant_id UUID;
    person_id UUID;
    item_id UUID;
    firmware_type_id UUID;
    asset_id UUID;
BEGIN
    SELECT id INTO tenant_id FROM public.tenants WHERE subdomain = 'default';
    SELECT id INTO person_id FROM public.person WHERE email = 'john@example.com';
    SELECT id INTO item_id FROM public.items WHERE internal_part_number = 'IPN-001';
    SELECT id INTO firmware_type_id FROM public.asset_types WHERE name = 'firmware';

    PERFORM public.set_tenant_context(tenant_id);

    -- Create firmware asset
    INSERT INTO public.assets (
        tenant_id, item_id, asset_type_id, name, version,
        file_type, created_by_id
    )
    VALUES (
        tenant_id, item_id, firmware_type_id, 'Widget Firmware',
        'v1.0.0', 'binary', person_id
    )
    RETURNING id INTO asset_id;

    -- Add firmware-specific data
    INSERT INTO public.firmware_specific (
        asset_id, hardware_version, is_critical
    )
    VALUES (
        asset_id, 'HW-v2.0', true
    );

    RAISE NOTICE 'PASS: Created test asset';
END $$;

-- Test 9.3: Query assets with type info
SELECT
    a.name,
    a.version,
    at.name as asset_type,
    i.internal_part_number,
    fs.hardware_version,
    fs.is_critical
FROM public.assets a
JOIN public.asset_types at ON a.asset_type_id = at.id
JOIN public.items i ON a.item_id = i.id
LEFT JOIN public.firmware_specific fs ON a.id = fs.asset_id;

-- ============================================================================
-- SECTION 10: MIGRATION 403 - MACHINE TABLES TESTS
-- ============================================================================

-- Test 10.1: Create test machines
DO $$
DECLARE
    tenant_id UUID;
    person_id UUID;
    item_id UUID;
    asset_id UUID;
    job_id UUID;
    machine_id UUID;
BEGIN
    SELECT id INTO tenant_id FROM public.tenants WHERE subdomain = 'default';
    SELECT id INTO person_id FROM public.person WHERE email = 'john@example.com';
    SELECT id INTO item_id FROM public.items WHERE internal_part_number = 'IPN-001';
    SELECT id INTO asset_id FROM public.assets LIMIT 1;
    SELECT id INTO job_id FROM public.jobs WHERE job_number = 'MFG-001';

    PERFORM public.set_tenant_context(tenant_id);

    -- Create machine
    INSERT INTO public.machines (
        tenant_id, name, ip, port, protocol, status
    )
    VALUES (
        tenant_id, 'Test Machine 1', '192.168.1.100', 8080, 'http', 'idle'
    )
    RETURNING id INTO machine_id;

    -- Create machine-item relationship
    INSERT INTO public.machine_item_relationships (
        machine_id, item_id, relationship_type
    )
    VALUES (
        machine_id, item_id, 'builds'
    );

    -- Create machine-asset relationship
    INSERT INTO public.machine_asset_relationships (
        machine_id, asset_id, relationship_type
    )
    VALUES (
        machine_id, asset_id, 'firmware'
    );

    -- Create machine-operator assignment
    INSERT INTO public.machine_operator_assignments (
        machine_id, person_id, assignment_type
    )
    VALUES (
        machine_id, person_id, 'primary'
    );

    -- Create machine-job assignment
    INSERT INTO public.machine_job_assignments (
        machine_id, job_id, status
    )
    VALUES (
        machine_id, job_id, 'pending'
    );

    RAISE NOTICE 'PASS: Created test machine with relationships';
END $$;

-- Test 10.2: Query machines with all relationships
SELECT
    m.name as machine_name,
    m.ip,
    m.status,
    i.internal_part_number as builds_item,
    p.name as operator_name,
    j.job_number as assigned_job
FROM public.machines m
LEFT JOIN public.machine_item_relationships mir ON m.id = mir.machine_id
LEFT JOIN public.items i ON mir.item_id = i.id
LEFT JOIN public.machine_operator_assignments moa ON m.id = moa.machine_id
LEFT JOIN public.person p ON moa.person_id = p.id
LEFT JOIN public.machine_job_assignments mja ON m.id = mja.machine_id
LEFT JOIN public.jobs j ON mja.job_id = j.id;

-- ============================================================================
-- SECTION 11: RLS (ROW LEVEL SECURITY) TESTS
-- ============================================================================

-- Test 11.1: Verify RLS is enabled on all tables
SELECT
    schemaname,
    tablename,
    rowsecurity as rls_enabled
FROM pg_tables
WHERE schemaname = 'public'
  AND tablename NOT IN ('asset_types')  -- Global lookup table
ORDER BY tablename;

-- Test 11.2: List all RLS policies
SELECT
    schemaname,
    tablename,
    policyname,
    permissive,
    roles,
    cmd,
    qual
FROM pg_policies
WHERE schemaname = 'public'
ORDER BY tablename, policyname;

-- Test 11.3: Test tenant isolation
DO $$
DECLARE
    tenant1_id UUID;
    tenant2_id UUID;
    person_id UUID;
    visible_count INTEGER;
BEGIN
    -- Create second tenant
    INSERT INTO public.tenants (name, subdomain)
    VALUES ('Tenant 2', 'tenant2')
    RETURNING id INTO tenant2_id;

    SELECT id INTO tenant1_id FROM public.tenants WHERE subdomain = 'default';

    -- Create person in tenant 2
    INSERT INTO public.person (supabase_uid, name, email)
    VALUES (uuid_generate_v4(), 'Tenant 2 User', 'user@tenant2.com')
    RETURNING id INTO person_id;

    INSERT INTO public.tenant_person (person_id, tenant_id, role)
    VALUES (person_id, tenant2_id, 'internal');

    -- Set context to tenant 1
    PERFORM public.set_tenant_context(tenant1_id);

    -- Count visible persons (should not see tenant 2 person)
    SELECT COUNT(*) INTO visible_count
    FROM public.person
    WHERE email = 'user@tenant2.com';

    IF visible_count = 0 THEN
        RAISE NOTICE 'PASS: Tenant isolation working - cannot see other tenant data';
    ELSE
        RAISE EXCEPTION 'FAIL: Tenant isolation breach - can see other tenant data';
    END IF;

    -- Set context to tenant 2
    PERFORM public.set_tenant_context(tenant2_id);

    -- Now should see the person
    SELECT COUNT(*) INTO visible_count
    FROM public.person
    WHERE email = 'user@tenant2.com';

    IF visible_count = 1 THEN
        RAISE NOTICE 'PASS: Can see own tenant data after context switch';
