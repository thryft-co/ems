use anyhow::Result;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl, SimpleAsyncConnection};
use uuid::Uuid;

use crate::models::{
    CreatePersonIdResponse, CreatePersonRequest, CustomerPerson, CustomerPersonData,
    CustomerPersonResponse, DistributorPerson, DistributorPersonData, DistributorPersonResponse,
    InternalPerson, InternalPersonData, InternalPersonResponse, NewCustomerPerson,
    NewDistributorPerson, NewInternalPerson, NewPerson, NewTenantPerson, NewVendorPerson, Person,
    PersonResponse, PersonRole, TenantPerson, UpdatePersonRequest, VendorPerson, VendorPersonData,
    VendorPersonResponse,
};
use crate::schema::*;
use crate::repositories::DatabaseService;

pub struct PersonService {
    database: DatabaseService,
}

impl PersonService {
    pub fn new(database: DatabaseService) -> Self {
        Self { database }
    }

    // General Person API methods

    pub async fn create_person(
        &self,
        tenant_id: Uuid,
        request: CreatePersonRequest,
    ) -> Result<CreatePersonIdResponse> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let person_id = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    // Create person record
                    let new_person = NewPerson {
                        supabase_uid: Uuid::new_v4(), // Generate temporary UID
                        name: request.name.clone(),
                        email: request.email.clone(),
                        phone: request.phone.clone(),
                        global_access: request
                            .global_access
                            .map(|ga| ga.into_iter().map(Some).collect()),
                        is_active: Some(true),
                    };

                    let person: Person = diesel::insert_into(person::table)
                        .values(&new_person)
                        .returning(Person::as_returning())
                        .get_result(conn)
                        .await?;

                    // Create tenant_person relationship
                    let new_tenant_person = NewTenantPerson {
                        person_id: person.id,
                        tenant_id,
                        role: request.role.to_string(),
                        access_level: Some(vec![Some("standard".to_string())]),
                        is_primary: Some(true),
                    };

                    diesel::insert_into(tenant_person::table)
                        .values(&new_tenant_person)
                        .execute(conn)
                        .await?;

                    // Create type-specific data based on role
                    match request.role {
                        PersonRole::Pending => {
                            // No type-specific data for pending users
                        }
                        PersonRole::Internal => {
                            let new_internal = NewInternalPerson {
                                person_id: person.id,
                                tenant_id,
                                department: request.department,
                                position: request.position,
                                employee_id: request.employee_id,
                                hire_date: request.hire_date,
                            };
                            diesel::insert_into(internal_person::table)
                                .values(&new_internal)
                                .execute(conn)
                                .await?;
                        }
                        PersonRole::Customer => {
                            let new_customer = NewCustomerPerson {
                                person_id: person.id,
                                tenant_id,
                                company: request.company,
                                industry: request.industry,
                                customer_since: request.customer_since,
                                account_manager_id: request.account_manager_id,
                            };
                            diesel::insert_into(customer_person::table)
                                .values(&new_customer)
                                .execute(conn)
                                .await?;
                        }
                        PersonRole::Vendor => {
                            let new_vendor = NewVendorPerson {
                                person_id: person.id,
                                tenant_id,
                                company: request.company,
                                service_type: request.service_type,
                                contract_start: request.contract_start,
                                contract_end: request.contract_end,
                            };
                            diesel::insert_into(vendor_person::table)
                                .values(&new_vendor)
                                .execute(conn)
                                .await?;
                        }
                        PersonRole::Distributor => {
                            let new_distributor = NewDistributorPerson {
                                person_id: person.id,
                                tenant_id,
                                company: request.company,
                                territory: request.territory,
                                distribution_tier: request.distribution_tier,
                                commission_rate: request.commission_rate,
                            };
                            diesel::insert_into(distributor_person::table)
                                .values(&new_distributor)
                                .execute(conn)
                                .await?;
                        }
                    }

                    Ok(person.id)
                })
            })
            .await
            .map_err(|e| anyhow::anyhow!("Transaction failed: {}", e))?;

        Ok(CreatePersonIdResponse { id: person_id })
    }

    pub async fn get_person_by_id(
        &self,
        tenant_id: Uuid,
        person_id: Uuid,
    ) -> Result<Option<PersonResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // Get person and tenant_person relationship
        let result = person::table
            .inner_join(tenant_person::table.on(person::id.eq(tenant_person::person_id)))
            .filter(person::id.eq(person_id))
            .filter(tenant_person::tenant_id.eq(tenant_id))
            .select((Person::as_select(), TenantPerson::as_select()))
            .first::<(Person, TenantPerson)>(&mut conn)
            .await
            .optional()?;

        if let Some((person, tenant_person)) = result {
            let role = PersonRole::try_from(tenant_person.role)
                .map_err(|e| anyhow::anyhow!("Invalid role: {}", e))?;

            // Get type-specific data based on role
            let (internal, customer, vendor, distributor) = match role {
                PersonRole::Pending => {
                    // No type-specific data for pending users
                    (None, None, None, None)
                }
                PersonRole::Internal => {
                    let internal_data = internal_person::table
                        .filter(internal_person::person_id.eq(person_id))
                        .filter(internal_person::tenant_id.eq(tenant_id))
                        .select(InternalPerson::as_select())
                        .first::<InternalPerson>(&mut conn)
                        .await
                        .optional()?;

                    let internal = internal_data.map(|i| InternalPersonData {
                        department: i.department,
                        position: i.position,
                        employee_id: i.employee_id,
                        hire_date: i.hire_date,
                    });

                    (internal, None, None, None)
                }
                PersonRole::Customer => {
                    let customer_data = customer_person::table
                        .filter(customer_person::person_id.eq(person_id))
                        .filter(customer_person::tenant_id.eq(tenant_id))
                        .select(CustomerPerson::as_select())
                        .first::<CustomerPerson>(&mut conn)
                        .await
                        .optional()?;

                    let customer = customer_data.map(|c| CustomerPersonData {
                        company: c.company,
                        industry: c.industry,
                        customer_since: c.customer_since,
                        account_manager_id: c.account_manager_id,
                    });

                    (None, customer, None, None)
                }
                PersonRole::Vendor => {
                    let vendor_data = vendor_person::table
                        .filter(vendor_person::person_id.eq(person_id))
                        .filter(vendor_person::tenant_id.eq(tenant_id))
                        .select(VendorPerson::as_select())
                        .first::<VendorPerson>(&mut conn)
                        .await
                        .optional()?;

                    let vendor = vendor_data.map(|v| VendorPersonData {
                        company: v.company,
                        service_type: v.service_type,
                        contract_start: v.contract_start,
                        contract_end: v.contract_end,
                    });

                    (None, None, vendor, None)
                }
                PersonRole::Distributor => {
                    let distributor_data = distributor_person::table
                        .filter(distributor_person::person_id.eq(person_id))
                        .filter(distributor_person::tenant_id.eq(tenant_id))
                        .select(DistributorPerson::as_select())
                        .first::<DistributorPerson>(&mut conn)
                        .await
                        .optional()?;

                    let distributor = distributor_data.map(|d| DistributorPersonData {
                        company: d.company,
                        territory: d.territory,
                        distribution_tier: d.distribution_tier,
                        commission_rate: d.commission_rate,
                    });

                    (None, None, None, distributor)
                }
            };

            Ok(Some(PersonResponse {
                id: person.id,
                name: person.name,
                email: person.email,
                phone: person.phone,
                person_type: role,
                global_access: person
                    .global_access
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|x| x)
                    .collect(),
                is_active: person.is_active.unwrap_or(true),
                last_login: person.last_login,
                created_at: person.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: person.updated_at.unwrap_or_else(|| Utc::now()),
                internal,
                customer,
                vendor,
                distributor,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_person(
        &self,
        tenant_id: Uuid,
        person_id: Uuid,
        request: UpdatePersonRequest,
    ) -> Result<PersonResponse> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            Box::pin(async move {
                // Update person fields individually to avoid Diesel type issues
                if let Some(name) = &request.name {
                    diesel::update(person::table.filter(person::id.eq(person_id)))
                        .set(person::name.eq(name))
                        .execute(conn)
                        .await?;
                }
                if let Some(phone) = &request.phone {
                    diesel::update(person::table.filter(person::id.eq(person_id)))
                        .set(person::phone.eq(phone))
                        .execute(conn)
                        .await?;
                }
                if let Some(global_access) = &request.global_access {
                    diesel::update(person::table.filter(person::id.eq(person_id)))
                        .set(person::global_access.eq(global_access))
                        .execute(conn)
                        .await?;
                }
                if let Some(is_active) = request.is_active {
                    diesel::update(person::table.filter(person::id.eq(person_id)))
                        .set(person::is_active.eq(is_active))
                        .execute(conn)
                        .await?;
                }

                // Update type-specific data based on role
                if let Some(role) = &request.role {
                    match role {
                        PersonRole::Pending => {
                            // No type-specific data to update for pending users
                        }
                        PersonRole::Internal => {
                            if let Some(department) = &request.department {
                                diesel::update(
                                    internal_person::table
                                        .filter(internal_person::person_id.eq(person_id))
                                        .filter(internal_person::tenant_id.eq(tenant_id)),
                                )
                                .set(internal_person::department.eq(department))
                                .execute(conn)
                                .await?;
                            }
                            if let Some(position) = &request.position {
                                diesel::update(
                                    internal_person::table
                                        .filter(internal_person::person_id.eq(person_id))
                                        .filter(internal_person::tenant_id.eq(tenant_id)),
                                )
                                .set(internal_person::position.eq(position))
                                .execute(conn)
                                .await?;
                            }
                            if let Some(employee_id) = &request.employee_id {
                                diesel::update(
                                    internal_person::table
                                        .filter(internal_person::person_id.eq(person_id))
                                        .filter(internal_person::tenant_id.eq(tenant_id)),
                                )
                                .set(internal_person::employee_id.eq(employee_id))
                                .execute(conn)
                                .await?;
                            }
                            if let Some(hire_date) = &request.hire_date {
                                diesel::update(
                                    internal_person::table
                                        .filter(internal_person::person_id.eq(person_id))
                                        .filter(internal_person::tenant_id.eq(tenant_id)),
                                )
                                .set(internal_person::hire_date.eq(hire_date))
                                .execute(conn)
                                .await?;
                            }
                        }
                        PersonRole::Customer => {
                            if let Some(company) = &request.company {
                                diesel::update(
                                    customer_person::table
                                        .filter(customer_person::person_id.eq(person_id))
                                        .filter(customer_person::tenant_id.eq(tenant_id)),
                                )
                                .set(customer_person::company.eq(company))
                                .execute(conn)
                                .await?;
                            }
                            if let Some(industry) = &request.industry {
                                diesel::update(
                                    customer_person::table
                                        .filter(customer_person::person_id.eq(person_id))
                                        .filter(customer_person::tenant_id.eq(tenant_id)),
                                )
                                .set(customer_person::industry.eq(industry))
                                .execute(conn)
                                .await?;
                            }
                            if let Some(customer_since) = &request.customer_since {
                                diesel::update(
                                    customer_person::table
                                        .filter(customer_person::person_id.eq(person_id))
                                        .filter(customer_person::tenant_id.eq(tenant_id)),
                                )
                                .set(customer_person::customer_since.eq(customer_since))
                                .execute(conn)
                                .await?;
                            }
                            if let Some(account_manager_id) = &request.account_manager_id {
                                diesel::update(
                                    customer_person::table
                                        .filter(customer_person::person_id.eq(person_id))
                                        .filter(customer_person::tenant_id.eq(tenant_id)),
                                )
                                .set(customer_person::account_manager_id.eq(account_manager_id))
                                .execute(conn)
                                .await?;
                            }
                        }
                        PersonRole::Vendor => {
                            if let Some(company) = &request.company {
                                diesel::update(
                                    vendor_person::table
                                        .filter(vendor_person::person_id.eq(person_id))
                                        .filter(vendor_person::tenant_id.eq(tenant_id)),
                                )
                                .set(vendor_person::company.eq(company))
                                .execute(conn)
                                .await?;
                            }
                            if let Some(service_type) = &request.service_type {
                                diesel::update(
                                    vendor_person::table
                                        .filter(vendor_person::person_id.eq(person_id))
                                        .filter(vendor_person::tenant_id.eq(tenant_id)),
                                )
                                .set(vendor_person::service_type.eq(service_type))
                                .execute(conn)
                                .await?;
                            }
                            if let Some(contract_start) = &request.contract_start {
                                diesel::update(
                                    vendor_person::table
                                        .filter(vendor_person::person_id.eq(person_id))
                                        .filter(vendor_person::tenant_id.eq(tenant_id)),
                                )
                                .set(vendor_person::contract_start.eq(contract_start))
                                .execute(conn)
                                .await?;
                            }
                            if let Some(contract_end) = &request.contract_end {
                                diesel::update(
                                    vendor_person::table
                                        .filter(vendor_person::person_id.eq(person_id))
                                        .filter(vendor_person::tenant_id.eq(tenant_id)),
                                )
                                .set(vendor_person::contract_end.eq(contract_end))
                                .execute(conn)
                                .await?;
                            }
                        }
                        PersonRole::Distributor => {
                            if let Some(company) = &request.company {
                                diesel::update(
                                    distributor_person::table
                                        .filter(distributor_person::person_id.eq(person_id))
                                        .filter(distributor_person::tenant_id.eq(tenant_id)),
                                )
                                .set(distributor_person::company.eq(company))
                                .execute(conn)
                                .await?;
                            }
                            if let Some(territory) = &request.territory {
                                diesel::update(
                                    distributor_person::table
                                        .filter(distributor_person::person_id.eq(person_id))
                                        .filter(distributor_person::tenant_id.eq(tenant_id)),
                                )
                                .set(distributor_person::territory.eq(territory))
                                .execute(conn)
                                .await?;
                            }
                            if let Some(distribution_tier) = &request.distribution_tier {
                                diesel::update(
                                    distributor_person::table
                                        .filter(distributor_person::person_id.eq(person_id))
                                        .filter(distributor_person::tenant_id.eq(tenant_id)),
                                )
                                .set(distributor_person::distribution_tier.eq(distribution_tier))
                                .execute(conn)
                                .await?;
                            }
                            if let Some(commission_rate) = &request.commission_rate {
                                diesel::update(
                                    distributor_person::table
                                        .filter(distributor_person::person_id.eq(person_id))
                                        .filter(distributor_person::tenant_id.eq(tenant_id)),
                                )
                                .set(distributor_person::commission_rate.eq(commission_rate))
                                .execute(conn)
                                .await?;
                            }
                        }
                    }
                }

                Ok(())
            })
        })
        .await
        .map_err(|e| anyhow::anyhow!("Update transaction failed: {}", e))?;

        // Return updated person
        self.get_person_by_id(tenant_id, person_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Person not found after update"))
    }

    pub async fn delete_person(&self, tenant_id: Uuid, person_id: Uuid) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // Delete tenant_person relationship (cascades will handle the rest)
        diesel::delete(
            tenant_person::table
                .filter(tenant_person::person_id.eq(person_id))
                .filter(tenant_person::tenant_id.eq(tenant_id)),
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    pub async fn list_persons(
        &self,
        tenant_id: Uuid,
        person_type: Option<PersonRole>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<PersonResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let mut query = person::table
            .inner_join(tenant_person::table.on(person::id.eq(tenant_person::person_id)))
            .filter(tenant_person::tenant_id.eq(tenant_id))
            .into_boxed();

        // Filter by person type if specified
        if let Some(role) = &person_type {
            query = query.filter(tenant_person::role.eq(role.to_string()));
        }

        // Apply pagination
        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i64);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val as i64);
        }

        let results = query
            .select((Person::as_select(), TenantPerson::as_select()))
            .load::<(Person, TenantPerson)>(&mut conn)
            .await?;

        let mut persons = Vec::new();
        for (person, tenant_person) in results {
            let role = PersonRole::try_from(tenant_person.role)
                .map_err(|e| anyhow::anyhow!("Invalid role: {}", e))?;

            // Get type-specific data based on role
            let (internal, customer, vendor, distributor) = match role {
                PersonRole::Pending => {
                    // No type-specific data for pending users
                    (None, None, None, None)
                }
                PersonRole::Internal => {
                    let internal_data = internal_person::table
                        .filter(internal_person::person_id.eq(person.id))
                        .filter(internal_person::tenant_id.eq(tenant_id))
                        .select(InternalPerson::as_select())
                        .first::<InternalPerson>(&mut conn)
                        .await
                        .optional()?;

                    let internal = internal_data.map(|i| InternalPersonData {
                        department: i.department,
                        position: i.position,
                        employee_id: i.employee_id,
                        hire_date: i.hire_date,
                    });

                    (internal, None, None, None)
                }
                PersonRole::Customer => {
                    let customer_data = customer_person::table
                        .filter(customer_person::person_id.eq(person.id))
                        .filter(customer_person::tenant_id.eq(tenant_id))
                        .select(CustomerPerson::as_select())
                        .first::<CustomerPerson>(&mut conn)
                        .await
                        .optional()?;

                    let customer = customer_data.map(|c| CustomerPersonData {
                        company: c.company,
                        industry: c.industry,
                        customer_since: c.customer_since,
                        account_manager_id: c.account_manager_id,
                    });

                    (None, customer, None, None)
                }
                PersonRole::Vendor => {
                    let vendor_data = vendor_person::table
                        .filter(vendor_person::person_id.eq(person.id))
                        .filter(vendor_person::tenant_id.eq(tenant_id))
                        .select(VendorPerson::as_select())
                        .first::<VendorPerson>(&mut conn)
                        .await
                        .optional()?;

                    let vendor = vendor_data.map(|v| VendorPersonData {
                        company: v.company,
                        service_type: v.service_type,
                        contract_start: v.contract_start,
                        contract_end: v.contract_end,
                    });

                    (None, None, vendor, None)
                }
                PersonRole::Distributor => {
                    let distributor_data = distributor_person::table
                        .filter(distributor_person::person_id.eq(person.id))
                        .filter(distributor_person::tenant_id.eq(tenant_id))
                        .select(DistributorPerson::as_select())
                        .first::<DistributorPerson>(&mut conn)
                        .await
                        .optional()?;

                    let distributor = distributor_data.map(|d| DistributorPersonData {
                        company: d.company,
                        territory: d.territory,
                        distribution_tier: d.distribution_tier,
                        commission_rate: d.commission_rate,
                    });

                    (None, None, None, distributor)
                }
            };

            persons.push(PersonResponse {
                id: person.id,
                name: person.name,
                email: person.email,
                phone: person.phone,
                person_type: role,
                global_access: person
                    .global_access
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|x| x)
                    .collect(),
                is_active: person.is_active.unwrap_or(true),
                last_login: person.last_login,
                created_at: person.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: person.updated_at.unwrap_or_else(|| Utc::now()),
                internal,
                customer,
                vendor,
                distributor,
            });
        }

        Ok(persons)
    }

    // Internal Person API methods

    pub async fn list_internal_persons(
        &self,
        tenant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<InternalPersonResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let mut query = internal_person::table
            .inner_join(person::table.on(internal_person::person_id.eq(person::id)))
            .filter(internal_person::tenant_id.eq(tenant_id))
            .into_boxed();

        // Apply pagination
        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i64);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val as i64);
        }

        let results = query
            .select((InternalPerson::as_select(), Person::as_select()))
            .load::<(InternalPerson, Person)>(&mut conn)
            .await?;

        let internal_persons = results
            .into_iter()
            .map(|(internal, person)| InternalPersonResponse {
                id: person.id,
                name: person.name,
                email: person.email,
                phone: person.phone,
                department: internal.department,
                position: internal.position,
                employee_id: internal.employee_id,
                hire_date: internal.hire_date,
                created_at: person.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: person.updated_at.unwrap_or_else(|| Utc::now()),
            })
            .collect();

        Ok(internal_persons)
    }

    pub async fn get_internal_person_by_id(
        &self,
        tenant_id: Uuid,
        person_id: Uuid,
    ) -> Result<Option<InternalPersonResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let result = internal_person::table
            .inner_join(person::table.on(internal_person::person_id.eq(person::id)))
            .filter(internal_person::person_id.eq(person_id))
            .filter(internal_person::tenant_id.eq(tenant_id))
            .select((InternalPerson::as_select(), Person::as_select()))
            .first::<(InternalPerson, Person)>(&mut conn)
            .await
            .optional()?;

        if let Some((internal, person)) = result {
            Ok(Some(InternalPersonResponse {
                id: person.id,
                name: person.name,
                email: person.email,
                phone: person.phone,
                department: internal.department,
                position: internal.position,
                employee_id: internal.employee_id,
                hire_date: internal.hire_date,
                created_at: person.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: person.updated_at.unwrap_or_else(|| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_internal_person(
        &self,
        tenant_id: Uuid,
        person_id: Uuid,
        request: UpdatePersonRequest,
    ) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // Check if there are any internal-specific fields to update
        let has_internal_updates = request.department.is_some()
            || request.position.is_some()
            || request.employee_id.is_some()
            || request.hire_date.is_some();

        if has_internal_updates {
            // Build the changeset using conditional values
            let changeset = (
                request
                    .department
                    .map(|d| internal_person::department.eq(d)),
                request.position.map(|p| internal_person::position.eq(p)),
                request
                    .employee_id
                    .map(|e| internal_person::employee_id.eq(e)),
                request.hire_date.map(|h| internal_person::hire_date.eq(h)),
            );

            // Apply only the Some values
            let target = internal_person::table
                .filter(internal_person::person_id.eq(person_id))
                .filter(internal_person::tenant_id.eq(tenant_id));

            if let Some(dept) = changeset.0 {
                diesel::update(target.clone())
                    .set(dept)
                    .execute(&mut conn)
                    .await?;
            }
            if let Some(pos) = changeset.1 {
                diesel::update(target.clone())
                    .set(pos)
                    .execute(&mut conn)
                    .await?;
            }
            if let Some(emp_id) = changeset.2 {
                diesel::update(target.clone())
                    .set(emp_id)
                    .execute(&mut conn)
                    .await?;
            }
            if let Some(hire) = changeset.3 {
                diesel::update(target).set(hire).execute(&mut conn).await?;
            }
        }

        Ok(())
    }

    // Customer Person API methods

    pub async fn list_customer_persons(
        &self,
        tenant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<CustomerPersonResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let mut query = customer_person::table
            .inner_join(person::table.on(customer_person::person_id.eq(person::id)))
            .filter(customer_person::tenant_id.eq(tenant_id))
            .into_boxed();

        // Apply pagination
        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i64);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val as i64);
        }

        let results = query
            .select((CustomerPerson::as_select(), Person::as_select()))
            .load::<(CustomerPerson, Person)>(&mut conn)
            .await?;

        let customer_persons = results
            .into_iter()
            .map(|(customer, person)| CustomerPersonResponse {
                id: person.id,
                name: person.name,
                email: person.email,
                phone: person.phone,
                company: customer.company,
                industry: customer.industry,
                customer_since: customer.customer_since,
                account_manager_id: customer.account_manager_id,
                created_at: person.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: person.updated_at.unwrap_or_else(|| Utc::now()),
            })
            .collect();

        Ok(customer_persons)
    }

    pub async fn get_customer_person_by_id(
        &self,
        tenant_id: Uuid,
        person_id: Uuid,
    ) -> Result<Option<CustomerPersonResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let result = customer_person::table
            .inner_join(person::table.on(customer_person::person_id.eq(person::id)))
            .filter(customer_person::person_id.eq(person_id))
            .filter(customer_person::tenant_id.eq(tenant_id))
            .select((CustomerPerson::as_select(), Person::as_select()))
            .first::<(CustomerPerson, Person)>(&mut conn)
            .await
            .optional()?;

        if let Some((customer, person)) = result {
            Ok(Some(CustomerPersonResponse {
                id: person.id,
                name: person.name,
                email: person.email,
                phone: person.phone,
                company: customer.company,
                industry: customer.industry,
                customer_since: customer.customer_since,
                account_manager_id: customer.account_manager_id,
                created_at: person.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: person.updated_at.unwrap_or_else(|| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_customer_person(
        &self,
        tenant_id: Uuid,
        person_id: Uuid,
        request: UpdatePersonRequest,
    ) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // Check if there are any customer-specific fields to update
        let has_customer_updates = request.company.is_some()
            || request.industry.is_some()
            || request.customer_since.is_some()
            || request.account_manager_id.is_some();

        if has_customer_updates {
            let target = customer_person::table
                .filter(customer_person::person_id.eq(person_id))
                .filter(customer_person::tenant_id.eq(tenant_id));

            if let Some(company) = request.company {
                diesel::update(target.clone())
                    .set(customer_person::company.eq(company))
                    .execute(&mut conn)
                    .await?;
            }
            if let Some(industry) = request.industry {
                diesel::update(target.clone())
                    .set(customer_person::industry.eq(industry))
                    .execute(&mut conn)
                    .await?;
            }
            if let Some(customer_since) = request.customer_since {
                diesel::update(target.clone())
                    .set(customer_person::customer_since.eq(customer_since))
                    .execute(&mut conn)
                    .await?;
            }
            if let Some(account_manager_id) = request.account_manager_id {
                diesel::update(target)
                    .set(customer_person::account_manager_id.eq(account_manager_id))
                    .execute(&mut conn)
                    .await?;
            }
        }

        Ok(())
    }

    // Vendor Person API methods

    pub async fn list_vendor_persons(
        &self,
        tenant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<VendorPersonResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let mut query = vendor_person::table
            .inner_join(person::table.on(vendor_person::person_id.eq(person::id)))
            .filter(vendor_person::tenant_id.eq(tenant_id))
            .into_boxed();

        // Apply pagination
        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i64);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val as i64);
        }

        let results = query
            .select((VendorPerson::as_select(), Person::as_select()))
            .load::<(VendorPerson, Person)>(&mut conn)
            .await?;

        let vendor_persons = results
            .into_iter()
            .map(|(vendor, person)| VendorPersonResponse {
                id: person.id,
                name: person.name,
                email: person.email,
                phone: person.phone,
                company: vendor.company,
                service_type: vendor.service_type,
                contract_start: vendor.contract_start,
                contract_end: vendor.contract_end,
                created_at: person.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: person.updated_at.unwrap_or_else(|| Utc::now()),
            })
            .collect();

        Ok(vendor_persons)
    }

    pub async fn get_vendor_person_by_id(
        &self,
        tenant_id: Uuid,
        person_id: Uuid,
    ) -> Result<Option<VendorPersonResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let result = vendor_person::table
            .inner_join(person::table.on(vendor_person::person_id.eq(person::id)))
            .filter(vendor_person::person_id.eq(person_id))
            .filter(vendor_person::tenant_id.eq(tenant_id))
            .select((VendorPerson::as_select(), Person::as_select()))
            .first::<(VendorPerson, Person)>(&mut conn)
            .await
            .optional()?;

        if let Some((vendor, person)) = result {
            Ok(Some(VendorPersonResponse {
                id: person.id,
                name: person.name,
                email: person.email,
                phone: person.phone,
                company: vendor.company,
                service_type: vendor.service_type,
                contract_start: vendor.contract_start,
                contract_end: vendor.contract_end,
                created_at: person.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: person.updated_at.unwrap_or_else(|| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_vendor_person(
        &self,
        tenant_id: Uuid,
        person_id: Uuid,
        request: UpdatePersonRequest,
    ) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // Check if there are any vendor-specific fields to update
        let has_vendor_updates = request.company.is_some()
            || request.service_type.is_some()
            || request.contract_start.is_some()
            || request.contract_end.is_some();

        if has_vendor_updates {
            let target = vendor_person::table
                .filter(vendor_person::person_id.eq(person_id))
                .filter(vendor_person::tenant_id.eq(tenant_id));

            if let Some(company) = request.company {
                diesel::update(target.clone())
                    .set(vendor_person::company.eq(company))
                    .execute(&mut conn)
                    .await?;
            }
            if let Some(service_type) = request.service_type {
                diesel::update(target.clone())
                    .set(vendor_person::service_type.eq(service_type))
                    .execute(&mut conn)
                    .await?;
            }
            if let Some(contract_start) = request.contract_start {
                diesel::update(target.clone())
                    .set(vendor_person::contract_start.eq(contract_start))
                    .execute(&mut conn)
                    .await?;
            }
            if let Some(contract_end) = request.contract_end {
                diesel::update(target)
                    .set(vendor_person::contract_end.eq(contract_end))
                    .execute(&mut conn)
                    .await?;
            }
        }

        Ok(())
    }

    // Distributor Person API methods

    pub async fn list_distributor_persons(
        &self,
        tenant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<DistributorPersonResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let mut query = distributor_person::table
            .inner_join(person::table.on(distributor_person::person_id.eq(person::id)))
            .filter(distributor_person::tenant_id.eq(tenant_id))
            .into_boxed();

        // Apply pagination
        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i64);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val as i64);
        }

        let results = query
            .select((DistributorPerson::as_select(), Person::as_select()))
            .load::<(DistributorPerson, Person)>(&mut conn)
            .await?;

        let distributor_persons = results
            .into_iter()
            .map(|(distributor, person)| DistributorPersonResponse {
                id: person.id,
                name: person.name,
                email: person.email,
                phone: person.phone,
                company: distributor.company,
                territory: distributor.territory,
                distribution_tier: distributor.distribution_tier,
                commission_rate: distributor.commission_rate,
                created_at: person.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: person.updated_at.unwrap_or_else(|| Utc::now()),
            })
            .collect();

        Ok(distributor_persons)
    }

    pub async fn get_distributor_person_by_id(
        &self,
        tenant_id: Uuid,
        person_id: Uuid,
    ) -> Result<Option<DistributorPersonResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let result = distributor_person::table
            .inner_join(person::table.on(distributor_person::person_id.eq(person::id)))
            .filter(distributor_person::person_id.eq(person_id))
            .filter(distributor_person::tenant_id.eq(tenant_id))
            .select((DistributorPerson::as_select(), Person::as_select()))
            .first::<(DistributorPerson, Person)>(&mut conn)
            .await
            .optional()?;

        if let Some((distributor, person)) = result {
            Ok(Some(DistributorPersonResponse {
                id: person.id,
                name: person.name,
                email: person.email,
                phone: person.phone,
                company: distributor.company,
                territory: distributor.territory,
                distribution_tier: distributor.distribution_tier,
                commission_rate: distributor.commission_rate,
                created_at: person.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: person.updated_at.unwrap_or_else(|| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_distributor_person(
        &self,
        tenant_id: Uuid,
        person_id: Uuid,
        request: UpdatePersonRequest,
    ) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // Check if there are any distributor-specific fields to update
        let has_distributor_updates = request.company.is_some()
            || request.territory.is_some()
            || request.distribution_tier.is_some()
            || request.commission_rate.is_some();

        if has_distributor_updates {
            let target = distributor_person::table
                .filter(distributor_person::person_id.eq(person_id))
                .filter(distributor_person::tenant_id.eq(tenant_id));

            if let Some(company) = request.company {
                diesel::update(target.clone())
                    .set(distributor_person::company.eq(company))
                    .execute(&mut conn)
                    .await?;
            }
            if let Some(territory) = request.territory {
                diesel::update(target.clone())
                    .set(distributor_person::territory.eq(territory))
                    .execute(&mut conn)
                    .await?;
            }
            if let Some(distribution_tier) = request.distribution_tier {
                diesel::update(target.clone())
                    .set(distributor_person::distribution_tier.eq(distribution_tier))
                    .execute(&mut conn)
                    .await?;
            }
            if let Some(commission_rate) = request.commission_rate {
                diesel::update(target)
                    .set(distributor_person::commission_rate.eq(commission_rate))
                    .execute(&mut conn)
                    .await?;
            }
        }

        Ok(())
    }
}
