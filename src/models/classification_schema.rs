use std::fmt::Debug;

use async_graphql::*;
use chrono::prelude::*;
use diesel::{self, ExpressionMethods, Insertable, Queryable};
use diesel::{QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Nation, User};
use crate::{database, schema::*};

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset, SimpleObject, QueryableByName
)]
#[graphql(complex)]
#[diesel(table_name = classification_schemas)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Nation))]
pub struct ClassificationSchema {
    pub id: Uuid,
    pub creator_id: Uuid, // User
    pub nation_code: String,
    // Conversions to NATO
    pub to_nato_unclassified: String,
    pub to_nato_restricted: String,
    pub to_nato_confidential: String,
    pub to_nato_secret: String,
    pub to_nato_top_secret: String,
    // Conversions from NATO
    pub from_nato_unclassified: String,
    pub from_nato_restricted: String,
    pub from_nato_confidential: String,
    pub from_nato_secret: String,
    pub from_nato_top_secret: String,
    // Other details
    pub caveats: String,
    pub version: String,
    pub nation_id: Uuid, // Nation
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
}

// GraphQL implementation
#[ComplexObject]
impl ClassificationSchema {
    pub async fn get_creator(&self) -> Result<User> {
        User::get_by_id(&self.creator_id)
    }

    pub async fn get_nation(&self) -> Result<Nation> {
        Nation::get_by_id(&self.nation_id)
    }
}

// Non GraphQL
impl ClassificationSchema {
    pub fn create(schema: &NewClassificationSchema) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = diesel::insert_into(classification_schemas::table)
            .values(schema)
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn get_or_create(schema: &NewClassificationSchema) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = classification_schemas::table
            .filter(classification_schemas::nation_code.eq(&schema.nation_code))
            .filter(classification_schemas::version.eq(&schema.version))
            .distinct()
            .first(&mut conn);

        let schema = match res {
            Ok(s) => s,
            Err(e) => {
                // ClassificationSchema not found
                println!("{:?}", e);
                let s = ClassificationSchema::create(schema)
                    .expect("Unable to create classification_schema");
                s
            }
        };
        Ok(schema)
    }

    pub fn get_all() -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = classification_schemas::table.load::<ClassificationSchema>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = classification_schemas::table
            .filter(classification_schemas::id.eq(id))
            .first(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_ids(ids: Vec<Uuid>) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = classification_schemas::table
            .filter(classification_schemas::id.eq_any(ids))
            .load::<ClassificationSchema>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_nation_codes(code_options: &Vec<Option<String>>) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;

        let mut codes = Vec::new();

        for op in code_options.iter() {
            match op {
                Some(s) => { codes.push(s )},
                None => (),
            }
        };

        let res = classification_schemas::table
            .filter(classification_schemas::nation_code.eq_any(codes))
            .load::<ClassificationSchema>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_creator_id(creator_id: Uuid) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = classification_schemas::table
            .filter(classification_schemas::creator_id.eq(creator_id))
            .load::<ClassificationSchema>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_nation_code(nation_code: &String) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = classification_schemas::table
            .filter(classification_schemas::nation_code.eq(nation_code))
            .load::<ClassificationSchema>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_nation_code_and_version(nation_code: &String, version: &String) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = classification_schemas::table
            .filter(classification_schemas::nation_code.eq(nation_code))
            .filter(classification_schemas::version.eq(version))
            .first(&mut conn)?;
        Ok(res)
    }

    // Return most recently updated Schema for a single nation by nation code
    pub fn get_latest_by_nation_code(nation_code: &String) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = classification_schemas::table
            .filter(classification_schemas::nation_code.eq(nation_code))
            .order(classification_schemas::updated_at.desc())
            .first(&mut conn)?;
        Ok(res)
    }

    // Return most recently updated Schemas by vector of nation codes
    pub fn get_latest_by_nation_codes(code_options: &Vec<Option<String>>) -> Result<Vec<Self>> {
        use diesel::dsl::sql;
        use diesel::sql_types::Bool;

        let mut conn = database::connection()?;

        let mut codes = Vec::new();

        for op in code_options.iter() {
            match op {
                Some(s) => { codes.push(s)},
                None => (),
            }
        };
                
        let results = classification_schemas::table
            .filter(classification_schemas::nation_code.eq_any(codes))
            .filter(sql::<Bool>(
                "updated_at = (
                    SELECT MAX(updated_at) 
                    FROM classification_schemas cs2 
                    WHERE cs2.nation_code = classification_schemas.nation_code
                )"
            ))
            .load::<Self>(&mut conn)?;
        
        Ok(results)
    }

    pub fn get_by_nation_id(nation_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = classification_schemas::table
            .filter(classification_schemas::nation_id.eq(nation_id))
            .load::<ClassificationSchema>(&mut conn)?;
        Ok(res)
    }

    // Return most recently updated Schema for a single nation by nation code
    pub fn get_latest_by_nation_id(nation_id: &Uuid) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = classification_schemas::table
            .filter(classification_schemas::nation_id.eq(nation_id))
            .order(classification_schemas::updated_at.desc())
            .first(&mut conn)?;
        Ok(res)
    }

    pub fn get_count() -> Result<i64> {
        let mut conn = database::connection()?;

        let res = classification_schemas::table
            .count()
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn update(&self) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = diesel::update(classification_schemas::table)
            .filter(classification_schemas::id.eq(&self.id))
            .set(self)
            .get_result(&mut conn)?;

        Ok(res)
    }

    /// Convert a source nation's classification to its NATO equivalent
    ///
    /// This performs the first step of the two-step conversion process:
    /// Source Nation Classification → NATO Standard
    ///
    /// # Arguments
    /// * `source_classification` - The classification level in the source nation's terminology
    ///
    /// # Returns
    /// The NATO equivalent classification level as a string
    ///
    /// # Example
    /// ```
    /// let schema = ClassificationSchema::get_latest_by_nation_code(&"USA".to_string())?;
    /// let nato_level = schema.to_nato("CONFIDENTIAL")?;
    /// // nato_level == "NATO CONFIDENTIAL"
    /// ```
    pub fn to_nato(&self, source_classification: &str) -> Result<String> {
        let source_upper = source_classification.trim().to_uppercase();

        // Match against the schema's mappings (case-insensitive)
        let nato_level = if source_upper == self.to_nato_unclassified.to_uppercase() {
            "NATO UNCLASSIFIED"
        } else if source_upper == self.to_nato_restricted.to_uppercase() {
            "NATO RESTRICTED"
        } else if source_upper == self.to_nato_confidential.to_uppercase() {
            "NATO CONFIDENTIAL"
        } else if source_upper == self.to_nato_secret.to_uppercase() {
            "NATO SECRET"
        } else if source_upper == self.to_nato_top_secret.to_uppercase() {
            "COSMIC TOP SECRET"
        } else {
            return Err(Error::new(format!(
                "Unknown classification '{}' for nation code {}. Valid classifications: {}, {}, {}, {}, {}",
                source_classification,
                self.nation_code,
                self.to_nato_unclassified,
                self.to_nato_restricted,
                self.to_nato_confidential,
                self.to_nato_secret,
                self.to_nato_top_secret
            )));
        };

        Ok(nato_level.to_string())
    }

    /// Convert a NATO classification to this nation's equivalent
    ///
    /// This performs the second step of the two-step conversion process:
    /// NATO Standard → Target Nation Classification
    ///
    /// # Arguments
    /// * `nato_classification` - The NATO classification level
    ///
    /// # Returns
    /// The equivalent classification in this nation's terminology
    ///
    /// # Example
    /// ```
    /// let schema = ClassificationSchema::get_latest_by_nation_code(&"GBR".to_string())?;
    /// let uk_level = schema.from_nato("NATO CONFIDENTIAL")?;
    /// // uk_level == "CONFIDENTIAL"
    /// ```
    pub fn from_nato(&self, nato_classification: &str) -> Result<String> {
        let nato_upper = nato_classification.trim().to_uppercase();

        let national_classification = match nato_upper.as_str() {
            "NATO UNCLASSIFIED" | "UNCLASSIFIED" => &self.from_nato_unclassified,
            "NATO RESTRICTED" | "RESTRICTED" => &self.from_nato_restricted,
            "NATO CONFIDENTIAL" | "CONFIDENTIAL" => &self.from_nato_confidential,
            "NATO SECRET" | "SECRET" => &self.from_nato_secret,
            "COSMIC TOP SECRET" | "TOP SECRET" | "NATO TOP SECRET" => &self.from_nato_top_secret,
            _ => {
                return Err(Error::new(format!(
                    "Unknown NATO classification '{}'. Valid NATO levels: NATO UNCLASSIFIED, NATO RESTRICTED, NATO CONFIDENTIAL, NATO SECRET, COSMIC TOP SECRET",
                    nato_classification
                )));
            }
        };

        Ok(national_classification.clone())
    }

    /// Check if this schema is still valid (not expired)
    pub fn is_valid(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at > Utc::now().naive_utc()
        } else {
            true // No expiration means always valid
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, InputObject)]
#[diesel(table_name = classification_schemas)]
pub struct NewClassificationSchema {
    pub creator_id: Uuid, // User
    pub nation_code: String,
    // Conversions to NATO
    pub to_nato_unclassified: String,
    pub to_nato_restricted: String,
    pub to_nato_confidential: String,
    pub to_nato_secret: String,
    pub to_nato_top_secret: String,
    // Conversions from NATO
    pub from_nato_unclassified: String,
    pub from_nato_restricted: String,
    pub from_nato_confidential: String,
    pub from_nato_secret: String,
    pub from_nato_top_secret: String,
    // Other details
    pub caveats: String,
    pub version: String,
    pub nation_id: Uuid, // Nation
    pub expires_at: Option<NaiveDateTime>,
}

impl NewClassificationSchema {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        creator_id: Uuid,
        nation_code: String,
        to_nato_unclassified: String,
        to_nato_restricted: String,
        to_nato_confidential: String,
        to_nato_secret: String,
        to_nato_top_secret: String,
        from_nato_unclassified: String,
        from_nato_restricted: String,
        from_nato_confidential: String,
        from_nato_secret: String,
        from_nato_top_secret: String,
        caveats: String,
        version: String,
        nation_id: Uuid,
        expires_at: Option<NaiveDateTime>,
    ) -> Self {
        NewClassificationSchema {
            creator_id,
            nation_code,
            to_nato_unclassified,
            to_nato_restricted,
            to_nato_confidential,
            to_nato_secret,
            to_nato_top_secret,
            from_nato_unclassified,
            from_nato_restricted,
            from_nato_confidential,
            from_nato_secret,
            from_nato_top_secret,
            caveats,
            version,
            nation_id,
            expires_at,
        }
    }
}
