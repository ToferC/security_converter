use std::fmt::Debug;
use std::collections::HashMap;

use async_graphql::*;
use chrono::prelude::*;
use diesel::{self, ExpressionMethods, Insertable, Queryable};
use diesel::{QueryDsl, RunQueryDsl};
use diesel::sql_types::{Array, Nullable, Text};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::encryption::EncryptedString;
use crate::models::{Authority, DataObject};
use crate::{database, schema::*};

// Define PostgreSQL array contains function
diesel::define_sql_function! {
    /// Check if a PostgreSQL array contains a specific element
    fn array_position(array: Array<Nullable<Text>>, element: Nullable<Text>) -> Nullable<diesel::sql_types::Integer>;
}

/// Represents the count of metadata records for a specific domain
#[derive(Debug, Clone, Deserialize, Serialize, SimpleObject)]
pub struct DomainCount {
    pub domain: String,
    pub count: i64,
}

/// Database model with encrypted fields
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = metadata)]
pub struct Metadata {
    pub id: Uuid,
    pub data_object_id: Uuid,

    // Global Identifier
    pub identifier: String,

    /// Authorization Reference - Legal basis for mission activities - 🔒 ENCRYPTED
    /// Examples: U.S. Law, DoD Policy, OPORD, FRAGO, MOU, Court Order
    pub authorization_reference: Option<EncryptedString>,
    pub authorization_reference_date: Option<NaiveDateTime>,

    /// Originator - Organization primarily responsible for generating the resource
    /// Should not change throughout data asset life cycle
    pub originator_organization_id: Uuid, // References Authority
    pub custodian_organization_id: Uuid, // References Authority

    /// Format - Physical attributes of data asset (e.g., email, JPEG, XML)
    /// Important for machine-to-machine interoperability
    pub format: String,
    pub format_size: Option<i64>,

    // Safeguarding and Securing
    pub security_classification: String, // e.g., "UNCLASSIFIED", "SECRET", "TOP SECRET"

    /// Disclosure & Releasability - Who can receive the resource
    /// Must have at least one: Country/Countries, Organization, or Category of People
    pub releasable_to_countries: Option<Vec<Option<String>>>, // e.g., ["USA", "GBR", "CAN"]
    pub releasable_to_organizations: Option<Vec<Option<String>>>, // e.g., ["NATO", "FVEY"]
    pub releasable_to_categories: Option<Vec<Option<String>>>, // e.g., ["contractors", "public"]
    pub disclosure_category: Option<String>, // e.g., "Category C"

    /// Handling Restrictions - Limitations beyond classification
    /// Examples: CUI, privacy controls, PII, law enforcement, medical restrictions
    pub handling_restrictions: Option<Vec<Option<String>>>,
    pub handling_authority: Option<String>, // legislation/policy authorizing restrictions
    pub no_handling_restrictions: Option<bool>, // Explicitly indicates no restrictions

    pub domain: String,
    pub tags: Vec<Option<String>>, // PostgreSQL TEXT[] array
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// GraphQL model with decrypted String fields
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct MetadataGraphQL {
    pub id: Uuid,
    pub data_object_id: Uuid,

    pub identifier: String,
    pub authorization_reference: Option<String>, // Decrypted for GraphQL
    pub authorization_reference_date: Option<NaiveDateTime>,

    #[graphql(skip)]
    pub originator_organization_id: Uuid,
    #[graphql(skip)]
    pub custodian_organization_id: Uuid,

    pub format: String,
    pub format_size: Option<i64>,

    pub security_classification: String,
    pub releasable_to_countries: Option<Vec<Option<String>>>,
    pub releasable_to_organizations: Option<Vec<Option<String>>>,
    pub releasable_to_categories: Option<Vec<Option<String>>>,
    pub disclosure_category: Option<String>,

    pub handling_restrictions: Option<Vec<Option<String>>>,
    pub handling_authority: Option<String>,
    pub no_handling_restrictions: Option<bool>,

    pub domain: String,
    pub tags: Vec<Option<String>>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// Convert database model to GraphQL model
impl From<Metadata> for MetadataGraphQL {
    fn from(meta: Metadata) -> Self {
        Self {
            id: meta.id,
            data_object_id: meta.data_object_id,
            identifier: meta.identifier,
            authorization_reference: meta.authorization_reference.map(|e| e.into_string()),
            authorization_reference_date: meta.authorization_reference_date,
            originator_organization_id: meta.originator_organization_id,
            custodian_organization_id: meta.custodian_organization_id,
            format: meta.format,
            format_size: meta.format_size,
            security_classification: meta.security_classification,
            releasable_to_countries: meta.releasable_to_countries,
            releasable_to_organizations: meta.releasable_to_organizations,
            releasable_to_categories: meta.releasable_to_categories,
            disclosure_category: meta.disclosure_category,
            handling_restrictions: meta.handling_restrictions,
            handling_authority: meta.handling_authority,
            no_handling_restrictions: meta.no_handling_restrictions,
            domain: meta.domain,
            tags: meta.tags,
            created_at: meta.created_at,
            updated_at: meta.updated_at,
        }
    }
}

// GraphQL implementation for complex fields
#[ComplexObject]
impl MetadataGraphQL {
    pub async fn data_object(&self) -> Result<crate::models::data_object::DataObjectGraphQL> {
        let db_obj = DataObject::get_by_id(&self.data_object_id)?;
        Ok(db_obj.into())
    }

    pub async fn originating_organization(&self) -> Result<Authority> {
        Authority::get_by_id(&self.originator_organization_id)
    }

    pub async fn custodian_organization(&self) -> Result<Authority> {
        Authority::get_by_id(&self.custodian_organization_id)
    }
}

// Non Graphql
impl Metadata {
    pub fn create(metadata: &NewMetadata) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = diesel::insert_into(metadata::table)
            .values(metadata)
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn get_or_create(metadata: &NewMetadata) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = metadata::table
            .filter(metadata::domain.eq(&metadata.domain))
            .distinct()
            .first(&mut conn);

        let metadata = match res {
            Ok(m) => m,
            Err(e) => {
                // Metadata not found
                println!("{:?}", e);
                let m = Metadata::create(metadata).expect("Unable to create metadata");
                m
            }
        };
        Ok(metadata)
    }

    pub fn get_all() -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = metadata::table.load::<Metadata>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = metadata::table
            .filter(metadata::id.eq(id))
            .first(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_domain(domain: String) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = metadata::table
            .filter(metadata::domain.eq(domain))
            .load::<Metadata>(&mut conn)?;
        Ok(res)
    }

    pub fn get_data_object_ids_by_domain(domain: String) -> Result<Vec<Uuid>> {
        let mut conn = database::connection()?;
        let res = metadata::table
            .filter(metadata::domain.eq(domain))
            .select(metadata::data_object_id)
            .load::<Uuid>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_data_object_id(data_object_id: &Uuid) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = metadata::table
            .filter(metadata::data_object_id.eq(data_object_id))
            .first(&mut conn)?;
        Ok(res)
    }

    pub fn get_counts_by_domain() -> Result<Vec<DomainCount>> {
        let mut conn = database::connection()?;
        let all_metadata = metadata::table.load::<Metadata>(&mut conn)?;

        let mut domain_counts: HashMap<String, i64> = HashMap::new();

        for metadata in all_metadata {
            *domain_counts.entry(metadata.domain.clone()).or_insert(0) += 1;
        }

        let result: Vec<DomainCount> = domain_counts
            .into_iter()
            .map(|(domain, count)| DomainCount { domain, count })
            .collect();

        Ok(result)
    }

    /// Search for metadata records that contain the specified tag in their tags array (exact match)
    pub fn get_by_tag(tag: &str) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;

        let tag_value: Option<String> = Some(tag.to_string());
        let res = metadata::table
            .filter(array_position(metadata::tags, tag_value).is_not_null())
            .load::<Metadata>(&mut conn)?;

        Ok(res)
    }

    /// Search for metadata records that contain ANY of the specified tags (exact match)
    pub fn get_by_tags(tags: &[String]) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;

        let mut query = metadata::table.into_boxed();

        // Build OR conditions for each tag
        for tag in tags {
            let tag_value: Option<String> = Some(tag.clone());
            query = query.or_filter(
                array_position(metadata::tags, tag_value).is_not_null()
            );
        }

        let res = query.load::<Metadata>(&mut conn)?;
        Ok(res)
    }

    /// Search for metadata records where any tag matches the pattern (case-insensitive, partial match)
    /// Uses SQL LIKE pattern matching on the array cast to text
    pub fn search_by_tag_pattern(pattern: &str) -> Result<Vec<Self>> {
        use diesel::dsl::sql;
        use diesel::sql_types::Bool;

        let mut conn = database::connection()?;

        // Use PostgreSQL's array_to_string to convert array to text and search with ILIKE
        let search_pattern = format!("%{}%", pattern);

        let res = metadata::table
            .filter(sql::<Bool>(&format!(
                "array_to_string(tags, ',') ILIKE '{}'",
                search_pattern.replace("'", "''") // Escape single quotes
            )))
            .load::<Metadata>(&mut conn)?;

        Ok(res)
    }

    pub fn update(&self) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = diesel::update(metadata::table)
            .filter(metadata::id.eq(&self.id))
            .set(self)
            .get_result(&mut conn)?;

        Ok(res)
    }
}

/// Database insertable model with encrypted fields
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = metadata)]
pub struct NewMetadata {
    pub data_object_id: Uuid,
    // Global Identifier
    pub identifier: String,

    // Authorization Reference - 🔒 ENCRYPTED
    pub authorization_reference: Option<EncryptedString>,
    pub authorization_reference_date: Option<NaiveDateTime>,

    // Originator and Custodian
    pub originator_organization_id: Uuid,
    pub custodian_organization_id: Uuid,

    // Format
    pub format: String,
    pub format_size: Option<i64>,

    // Safeguarding and Securing
    pub security_classification: String,

    // Disclosure & Releasability
    pub releasable_to_countries: Option<Vec<Option<String>>>,
    pub releasable_to_organizations: Option<Vec<Option<String>>>,
    pub releasable_to_categories: Option<Vec<Option<String>>>,
    pub disclosure_category: Option<String>,

    // Handling Restrictions
    pub handling_restrictions: Option<Vec<Option<String>>>,
    pub handling_authority: Option<String>,
    pub no_handling_restrictions: Option<bool>,

    // Legacy fields
    pub domain: String,
    pub tags: Vec<Option<String>>,
}

/// A light struct to accept the JSON formatted Metadata included with
/// a ConversionRequest
/// GraphQL input type accepts plain String (will be encrypted internally)
#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(name = "MetadataInput")]
pub struct InsertableMetadata {
    // Global Identifier
    pub identifier: String,

    // Authorization Reference - will be encrypted
    pub authorization_reference: Option<String>,
    pub authorization_reference_date: Option<NaiveDateTime>,

    // Originator and Custodian
    pub originator_organization_id: Uuid, // Authority
    pub custodian_organization_id: Uuid, // Authority

    // Format
    pub format: String,
    pub format_size: Option<i64>,

    // Safeguarding and Securing
    pub security_classification: String,

    // Disclosure & Releasability
    pub releasable_to_countries: Option<Vec<Option<String>>>,
    pub releasable_to_organizations: Option<Vec<Option<String>>>,
    pub releasable_to_categories: Option<Vec<Option<String>>>,
    pub disclosure_category: Option<String>,

    // Handling Restrictions
    pub handling_restrictions: Option<Vec<Option<String>>>,
    pub handling_authority: Option<String>,
    pub no_handling_restrictions: Option<bool>,

    // Legacy fields
    pub domain: String,
    pub tags: Vec<Option<String>>,
}

impl InsertableMetadata {
    /// Convert to NewMetadata with encrypted fields
    pub fn to_new_metadata(&self, data_object_id: Uuid) -> NewMetadata {
        NewMetadata {
            data_object_id,
            identifier: self.identifier.clone(),
            authorization_reference: self.authorization_reference.as_ref().map(|s| EncryptedString::from(s.clone())),
            authorization_reference_date: self.authorization_reference_date,
            originator_organization_id: self.originator_organization_id,
            custodian_organization_id: self.custodian_organization_id,
            format: self.format.clone(),
            format_size: self.format_size,
            security_classification: self.security_classification.clone(),
            releasable_to_countries: self.releasable_to_countries.clone(),
            releasable_to_organizations: self.releasable_to_organizations.clone(),
            releasable_to_categories: self.releasable_to_categories.clone(),
            disclosure_category: self.disclosure_category.clone(),
            handling_restrictions: self.handling_restrictions.clone(),
            handling_authority: self.handling_authority.clone(),
            no_handling_restrictions: self.no_handling_restrictions,
            domain: self.domain.clone(),
            tags: self.tags.clone(),
        }
    }
}
