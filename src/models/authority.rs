use std::fmt::Debug;

use async_graphql::*;
use chrono::prelude::*;
use diesel::{self, ExpressionMethods, Insertable, Queryable};
use diesel::{QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{ClassificationSchema, Nation, User, ConversionRequest};
use crate::{database, schema::*};

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset, SimpleObject,
)]
#[graphql(complex)]
#[diesel(table_name = authorities)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Nation))]
pub struct Authority {
    pub id: Uuid,
    pub creator_id: Uuid, // User
    pub nation_id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
}

#[ComplexObject]
impl Authority {
    pub async fn creator(&self, ctx: &Context<'_>) -> Result<User> {
        let loaders = ctx.data::<crate::graphql::Loaders>()?;
        let user = loaders.user_loader.load(self.creator_id).await;
        Ok(user)
    }

    pub async fn nation(&self) -> Result<Nation> {
        Nation::get_by_id(&self.nation_id)
    }

    pub async fn conversion_requests(&self) -> Result<Vec<ConversionRequest>> {
        ConversionRequest::get_by_authority_id(&self.id)
    }

    pub async fn classification_schemas(&self) -> Result<Vec<ClassificationSchema>> {
        ClassificationSchema::get_by_nation_id(&self.nation_id)
    }

    pub async fn most_recent_classification_schema(&self) -> Result<ClassificationSchema> {
        ClassificationSchema::get_latest_by_nation_id(&self.nation_id)
    }
}

// Non Graphql
impl Authority {
    pub fn create(authority: &NewAuthority) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = diesel::insert_into(authorities::table)
            .values(authority)
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn get_or_create(authority: &NewAuthority) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = authorities::table
            .filter(authorities::creator_id.eq(&authority.creator_id))
            .distinct()
            .first(&mut conn);

        let authority = match res {
            Ok(p) => p,
            Err(e) => {
                // Authority not found
                println!("{:?}", e);
                let p = Authority::create(authority).expect("Unable to create authority");
                p
            }
        };
        Ok(authority)
    }

    pub fn get_all() -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = authorities::table.load::<Authority>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = authorities::table
            .filter(authorities::id.eq(id))
            .first(&mut conn)?;
        Ok(res)
    }

    /// Batch load authorities by multiple IDs (for DataLoader)
    pub fn get_by_ids(ids: Vec<Uuid>) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = authorities::table
            .filter(authorities::id.eq_any(ids))
            .load::<Authority>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_creator_id(creator_id: Uuid) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = authorities::table
            .filter(authorities::creator_id.eq(creator_id))
            .load::<Authority>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_nation_id(nation_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = authorities::table
            .filter(authorities::nation_id.eq(nation_id))
            .load::<Authority>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_nation_code(nation_code: &String) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;

        let nation_id = Nation::get_by_code(nation_code)?.id;

        let res = authorities::table
            .filter(authorities::nation_id.eq(nation_id))
            .load::<Authority>(&mut conn)?;
        Ok(res)
    }

    pub fn update(&self) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = diesel::update(authorities::table)
            .filter(authorities::id.eq(&self.id))
            .set(self)
            .get_result(&mut conn)?;

        Ok(res)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, SimpleObject, InputObject)]
#[diesel(table_name = authorities)]
pub struct NewAuthority {
    pub creator_id: Uuid, // User
    pub nation_id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub expires_at: Option<NaiveDateTime>,
}

impl NewAuthority {
    pub fn new(
        creator_id: Uuid,
        nation_id: Uuid,
        name: String,
        email: String,
        phone: String,
        expires_at: Option<NaiveDateTime>,
    ) -> Self {
        NewAuthority {
            creator_id,
            nation_id,
            name,
            email,
            phone,
            expires_at,
        }
    }
}
