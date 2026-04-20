// @generated automatically by Diesel CLI.

diesel::table! {
    asset_types (id) {
        id -> Uuid,
        #[max_length = 50]
        name -> Varchar,
        description -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    assets (id) {
        id -> Uuid,
        tenant_id -> Uuid,
        item_id -> Uuid,
        asset_type_id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 50]
        version -> Nullable<Varchar>,
        description -> Nullable<Text>,
        #[max_length = 500]
        file_path -> Nullable<Varchar>,
        file_size -> Nullable<Int8>,
        #[max_length = 50]
        file_type -> Nullable<Varchar>,
        #[max_length = 64]
        checksum -> Nullable<Varchar>,
        is_active -> Nullable<Bool>,
        metadata -> Nullable<Jsonb>,
        created_by_id -> Uuid,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    customer_person (id) {
        id -> Uuid,
        person_id -> Uuid,
        tenant_id -> Uuid,
        #[max_length = 100]
        company -> Nullable<Varchar>,
        #[max_length = 50]
        industry -> Nullable<Varchar>,
        customer_since -> Nullable<Timestamptz>,
        account_manager_id -> Nullable<Uuid>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    distributor_person (id) {
        id -> Uuid,
        person_id -> Uuid,
        tenant_id -> Uuid,
        #[max_length = 100]
        company -> Nullable<Varchar>,
        #[max_length = 100]
        territory -> Nullable<Varchar>,
        #[max_length = 50]
        distribution_tier -> Nullable<Varchar>,
        #[max_length = 10]
        commission_rate -> Nullable<Varchar>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    firmware_specific (id) {
        id -> Uuid,
        asset_id -> Uuid,
        #[max_length = 50]
        hardware_version -> Nullable<Varchar>,
        #[max_length = 50]
        min_hardware_version -> Nullable<Varchar>,
        #[max_length = 50]
        max_hardware_version -> Nullable<Varchar>,
        release_notes -> Nullable<Text>,
        is_beta -> Nullable<Bool>,
        is_critical -> Nullable<Bool>,
        requires_manual_update -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    internal_person (id) {
        id -> Uuid,
        person_id -> Uuid,
        tenant_id -> Uuid,
        #[max_length = 50]
        department -> Nullable<Varchar>,
        #[max_length = 100]
        position -> Nullable<Varchar>,
        #[max_length = 20]
        employee_id -> Nullable<Varchar>,
        hire_date -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    inventory_items (id) {
        id -> Uuid,
        item_id -> Uuid,
        tenant_id -> Uuid,
        #[max_length = 20]
        context -> Varchar,
        quantity -> Nullable<Int4>,
        #[max_length = 100]
        location -> Nullable<Varchar>,
        pricing -> Nullable<Jsonb>,
        lead_time -> Nullable<Int4>,
        min_stock_level -> Nullable<Int4>,
        max_stock_level -> Nullable<Int4>,
        reorder_point -> Nullable<Int4>,
        vendor_id -> Nullable<Uuid>,
        last_received_date -> Nullable<Timestamptz>,
        #[max_length = 20]
        status -> Nullable<Varchar>,
        notes -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    item_bom (id) {
        id -> Uuid,
        tenant_id -> Uuid,
        parent_item_id -> Uuid,
        component_item_id -> Uuid,
        quantity -> Nullable<Int4>,
        notes -> Nullable<Text>,
        is_optional -> Nullable<Bool>,
        substitutes -> Nullable<Array<Nullable<Uuid>>>,
        assembly_order -> Nullable<Int4>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    items (id) {
        id -> Uuid,
        #[max_length = 50]
        internal_part_number -> Varchar,
        #[max_length = 50]
        mfr_part_number -> Nullable<Varchar>,
        #[max_length = 100]
        manufacturer -> Varchar,
        #[max_length = 500]
        datasheet -> Nullable<Varchar>,
        #[max_length = 20]
        lifecycle -> Nullable<Varchar>,
        description -> Nullable<Text>,
        #[max_length = 50]
        category -> Nullable<Varchar>,
        metadata -> Nullable<Jsonb>,
        linked_resources -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    job_history (id) {
        id -> Uuid,
        job_id -> Uuid,
        tenant_id -> Uuid,
        person_id -> Nullable<Uuid>,
        #[max_length = 50]
        action -> Varchar,
        #[max_length = 20]
        previous_status -> Nullable<Varchar>,
        #[max_length = 20]
        new_status -> Nullable<Varchar>,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    jobs (id) {
        id -> Uuid,
        tenant_id -> Uuid,
        #[max_length = 50]
        job_number -> Varchar,
        item_id -> Nullable<Uuid>,
        quantity -> Int4,
        assigned_person_id -> Nullable<Uuid>,
        supervisor_id -> Nullable<Uuid>,
        customer_id -> Nullable<Uuid>,
        #[max_length = 20]
        job_type -> Varchar,
        #[max_length = 20]
        priority -> Nullable<Varchar>,
        start_date -> Nullable<Timestamptz>,
        end_date -> Nullable<Timestamptz>,
        due_date -> Nullable<Timestamptz>,
        #[max_length = 20]
        status -> Varchar,
        comments -> Nullable<Text>,
        materials_consumed -> Nullable<Jsonb>,
        labor_hours -> Nullable<Float8>,
        metadata -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    machine_asset_relationships (id) {
        id -> Uuid,
        machine_id -> Uuid,
        asset_id -> Uuid,
        #[max_length = 20]
        relationship_type -> Varchar,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    machine_item_relationships (id) {
        id -> Uuid,
        machine_id -> Uuid,
        item_id -> Uuid,
        #[max_length = 20]
        relationship_type -> Varchar,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    machine_job_assignments (id) {
        id -> Uuid,
        machine_id -> Uuid,
        job_id -> Uuid,
        #[max_length = 20]
        status -> Varchar,
        start_time -> Nullable<Timestamptz>,
        end_time -> Nullable<Timestamptz>,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    machine_operator_assignments (id) {
        id -> Uuid,
        machine_id -> Uuid,
        person_id -> Uuid,
        #[max_length = 20]
        assignment_type -> Varchar,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    machines (id) {
        id -> Uuid,
        tenant_id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 45]
        ip -> Varchar,
        port -> Int4,
        #[max_length = 20]
        protocol -> Varchar,
        #[max_length = 20]
        status -> Varchar,
        #[max_length = 20]
        action -> Nullable<Varchar>,
        payload -> Nullable<Jsonb>,
        last_heartbeat -> Nullable<Timestamptz>,
        metadata -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    manufacturing_job (id) {
        id -> Uuid,
        job_id -> Uuid,
        tenant_id -> Uuid,
        #[max_length = 50]
        work_order_number -> Nullable<Varchar>,
        #[max_length = 100]
        production_line -> Nullable<Varchar>,
        #[max_length = 50]
        machine_id -> Nullable<Varchar>,
        setup_time_hours -> Nullable<Float8>,
        cycle_time_minutes -> Nullable<Float8>,
        quality_check_required -> Nullable<Bool>,
        batch_size -> Nullable<Int4>,
        tool_requirements -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    order_history (id) {
        id -> Uuid,
        order_id -> Uuid,
        tenant_id -> Uuid,
        person_id -> Nullable<Uuid>,
        #[max_length = 50]
        action -> Varchar,
        #[max_length = 20]
        previous_status -> Nullable<Varchar>,
        #[max_length = 20]
        new_status -> Nullable<Varchar>,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    order_items (id) {
        id -> Uuid,
        order_id -> Uuid,
        item_id -> Nullable<Uuid>,
        #[max_length = 200]
        item_name -> Varchar,
        item_description -> Nullable<Text>,
        quantity -> Int4,
        unit_price -> Float8,
        extended_price -> Float8,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    orders (id) {
        id -> Uuid,
        tenant_id -> Uuid,
        #[max_length = 50]
        order_number -> Varchar,
        #[max_length = 20]
        order_type -> Varchar,
        external_entity_id -> Uuid,
        #[max_length = 20]
        external_entity_type -> Varchar,
        order_date -> Timestamptz,
        total_amount -> Float8,
        #[max_length = 20]
        status -> Varchar,
        created_by_id -> Uuid,
        notes -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    person (id) {
        id -> Uuid,
        supabase_uid -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 100]
        email -> Varchar,
        #[max_length = 20]
        phone -> Nullable<Varchar>,
        global_access -> Nullable<Array<Nullable<Varchar>>>,
        is_active -> Nullable<Bool>,
        last_login -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    qa_job (id) {
        id -> Uuid,
        job_id -> Uuid,
        tenant_id -> Uuid,
        #[max_length = 50]
        inspection_type -> Nullable<Varchar>,
        #[max_length = 50]
        test_procedure_id -> Nullable<Varchar>,
        acceptance_criteria -> Nullable<Text>,
        sampling_size -> Nullable<Int4>,
        test_equipment -> Nullable<Jsonb>,
        calibration_required -> Nullable<Bool>,
        environmental_conditions -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    service_job (id) {
        id -> Uuid,
        job_id -> Uuid,
        tenant_id -> Uuid,
        #[max_length = 50]
        service_type -> Nullable<Varchar>,
        #[max_length = 200]
        location -> Nullable<Varchar>,
        #[max_length = 100]
        equipment_serial_number -> Nullable<Varchar>,
        #[max_length = 50]
        maintenance_type -> Nullable<Varchar>,
        parts_required -> Nullable<Jsonb>,
        safety_requirements -> Nullable<Jsonb>,
        travel_time_hours -> Nullable<Float8>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    tenant_person (id) {
        id -> Uuid,
        person_id -> Uuid,
        tenant_id -> Uuid,
        #[max_length = 20]
        role -> Varchar,
        access_level -> Nullable<Array<Nullable<Varchar>>>,
        is_primary -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    tenants (id) {
        id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 50]
        subdomain -> Varchar,
        #[max_length = 500]
        database_url -> Nullable<Varchar>,
        settings -> Nullable<Jsonb>,
        is_active -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    token_blacklist (id) {
        id -> Uuid,
        #[max_length = 255]
        token_hash -> Varchar,
        #[max_length = 20]
        token_type -> Varchar,
        person_id -> Uuid,
        tenant_id -> Uuid,
        expires_at -> Timestamptz,
        blacklisted_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    vendor_person (id) {
        id -> Uuid,
        person_id -> Uuid,
        tenant_id -> Uuid,
        #[max_length = 100]
        company -> Nullable<Varchar>,
        #[max_length = 100]
        service_type -> Nullable<Varchar>,
        contract_start -> Nullable<Timestamptz>,
        contract_end -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(assets -> asset_types (asset_type_id));
diesel::joinable!(assets -> items (item_id));
diesel::joinable!(assets -> person (created_by_id));
diesel::joinable!(assets -> tenants (tenant_id));
diesel::joinable!(customer_person -> tenants (tenant_id));
diesel::joinable!(distributor_person -> person (person_id));
diesel::joinable!(distributor_person -> tenants (tenant_id));
diesel::joinable!(firmware_specific -> assets (asset_id));
diesel::joinable!(internal_person -> person (person_id));
diesel::joinable!(internal_person -> tenants (tenant_id));
diesel::joinable!(inventory_items -> items (item_id));
diesel::joinable!(inventory_items -> person (vendor_id));
diesel::joinable!(inventory_items -> tenants (tenant_id));
diesel::joinable!(item_bom -> tenants (tenant_id));
diesel::joinable!(job_history -> jobs (job_id));
diesel::joinable!(job_history -> person (person_id));
diesel::joinable!(job_history -> tenants (tenant_id));
diesel::joinable!(jobs -> tenants (tenant_id));
diesel::joinable!(machine_asset_relationships -> assets (asset_id));
diesel::joinable!(machine_asset_relationships -> machines (machine_id));
diesel::joinable!(machine_item_relationships -> items (item_id));
diesel::joinable!(machine_item_relationships -> machines (machine_id));
diesel::joinable!(machine_job_assignments -> jobs (job_id));
diesel::joinable!(machine_job_assignments -> machines (machine_id));
diesel::joinable!(machine_operator_assignments -> machines (machine_id));
diesel::joinable!(machine_operator_assignments -> person (person_id));
diesel::joinable!(machines -> tenants (tenant_id));
diesel::joinable!(manufacturing_job -> jobs (job_id));
diesel::joinable!(manufacturing_job -> tenants (tenant_id));
diesel::joinable!(order_history -> orders (order_id));
diesel::joinable!(order_history -> person (person_id));
diesel::joinable!(order_history -> tenants (tenant_id));
diesel::joinable!(order_items -> orders (order_id));
diesel::joinable!(orders -> tenants (tenant_id));
diesel::joinable!(qa_job -> jobs (job_id));
diesel::joinable!(qa_job -> tenants (tenant_id));
diesel::joinable!(service_job -> jobs (job_id));
diesel::joinable!(service_job -> tenants (tenant_id));
diesel::joinable!(tenant_person -> person (person_id));
diesel::joinable!(tenant_person -> tenants (tenant_id));
diesel::joinable!(token_blacklist -> person (person_id));
diesel::joinable!(token_blacklist -> tenants (tenant_id));
diesel::joinable!(vendor_person -> person (person_id));
diesel::joinable!(vendor_person -> tenants (tenant_id));

diesel::allow_tables_to_appear_in_same_query!(
    asset_types,
    assets,
    customer_person,
    distributor_person,
    firmware_specific,
    internal_person,
    inventory_items,
    item_bom,
    items,
    job_history,
    jobs,
    machine_asset_relationships,
    machine_item_relationships,
    machine_job_assignments,
    machine_operator_assignments,
    machines,
    manufacturing_job,
    order_history,
    order_items,
    orders,
    person,
    qa_job,
    service_job,
    tenant_person,
    tenants,
    token_blacklist,
    vendor_person,
);
