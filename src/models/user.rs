// Modelled off https://github.com/clifinger/canduma/blob/master/src/user

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use diesel::{self, ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl};
use uuid::Uuid;
use async_graphql::*;

use crate::{schema::*};
use crate::common_utils::{is_admin, RoleGuard, UserRole};
use crate::models::hash_password;
use crate::database::connection;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserInstance {
    id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, SimpleObject, Queryable, AsChangeset)]
#[graphql(complex)]
pub struct User {
    #[graphql(
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub id: Uuid,
    #[graphql(skip)]
    pub hash: String,

    #[graphql(
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub email: String,
    pub role: String,

    #[graphql(
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub name: String,
    pub access_level: String, // AccessLevelEnum
    pub created_at: NaiveDateTime,
    #[graphql(
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]

    pub updated_at: NaiveDateTime,
    #[graphql(
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]

    /// Access Level: Admin
    pub access_key: String,

    #[graphql(
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    /// Access Level: Admin
    pub approved_by_user_uid: Option<Uuid>,

    #[graphql(
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    /// Foreign key to Authority - Required for non-ADMIN users
    /// Establishes User -> Authority -> Nation hierarchy
    pub authority_id: Option<Uuid>,
}

impl User {
    /// Validates that authority_id is provided for non-admin users and that the authority exists
    pub fn validate_authority_requirement(
        role: &str,
        authority_id: Option<Uuid>
    ) -> Result<(), Error> {
        // Admin users don't need an authority
        if role == "ADMIN" {
            return Ok(());
        }

        // All other roles require an authority
        let auth_id = authority_id.ok_or_else(|| {
            Error::new(format!(
                "Users with role '{}' must be associated with an Authority. Only ADMIN users can exist without an authority.",
                role
            ))
        })?;

        // Validate that the authority exists
        use crate::models::Authority;
        Authority::get_by_id(&auth_id).map_err(|_| {
            Error::new(format!(
                "Authority with id {} does not exist. Please provide a valid authority_id.",
                auth_id
            ))
        })?;

        Ok(())
    }

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;
        let user = users::table
            .filter(users::id.eq(id))
            .get_result(&mut conn)?;

        Ok(user)
    }

    /// Batch load users by multiple IDs (for DataLoader)
    pub fn get_by_ids(ids: Vec<Uuid>) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let users = users::table
            .filter(users::id.eq_any(ids))
            .load::<User>(&mut conn)?;

        Ok(users)
    }

    pub fn get_by_email(email: &String) -> Result<Self> {
        let mut conn = connection()?;
        let user = users::table
            .filter(users::email.eq(email))
            .get_result(&mut conn)?;

        Ok(user)
    }

    pub fn create(user: InsertableUser) -> Result<Self> {
        let mut conn = connection()?;
        let user = diesel::insert_into(users::table)
            .values(&user)
            .get_result(&mut conn)?;

        Ok(user)
    }

    pub fn get_all() -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let users = users::table.load::<User>(&mut conn)?;
        Ok(users)
    }

    pub fn update(&mut self) -> Result<Self> {
        let mut conn = connection()?;

        self.updated_at = chrono::Utc::now().naive_utc();

        let user = diesel::update(users::table)
            .filter(users::id.eq(&self.id))
            .set(self.clone())
            .get_result(&mut conn)?;

        Ok(user)
    }
}

// GraphQL ComplexObject implementation for User->Authority relationship
#[ComplexObject]
impl User {
    /// Get the authority this user belongs to (if any)
    /// Returns None for ADMIN users who don't belong to an authority
    pub async fn authority(&self, ctx: &Context<'_>) -> Result<Option<crate::models::Authority>> {
        match self.authority_id {
            Some(authority_id) => {
                let loaders = ctx.data::<crate::graphql::Loaders>()?;
                let authority = loaders.authority_loader.load(authority_id).await;
                Ok(Some(authority))
            },
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable)]
#[diesel(table_name = users)]
pub struct InsertableUser {
    pub hash: String,
    pub email: String,
    pub role: String,
    pub name: String,
    pub access_level: String, // AccessLevelEnum
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub access_key: String,
    pub approved_by_user_uid: Option<Uuid>,
    pub authority_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, InputObject)]
/// Input Struct to create a new user. Only accessible by Administrators.
pub struct UserData {
    pub name: String,
    pub email: String,
    pub password: String,
    /// UserRole in system: USER, OPERATOR, ANALYST, ADMIN
    pub role: String,
    /// Authority ID - Required for non-ADMIN users
    pub authority_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, InputObject)]
/// Input Struct to update a user. Only accessible by Administrators.
pub struct UserUpdate {
    pub id: Uuid,
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    /// UserRole in system: USER, OPERATOR, ANALYST, ADMIN
    pub role: Option<String>,
    /// Authority ID - Required for non-ADMIN users
    pub authority_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, Clone, SimpleObject)]
pub struct SlimUser {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub access_level: String,
    pub authority_id: Option<Uuid>,
}

#[derive(Shrinkwrap, Clone, Default)]
pub struct LoggedUser(pub Option<SlimUser>);

impl From<SlimUser> for LoggedUser {
    fn from(slim_user: SlimUser) -> Self {
        LoggedUser(Some(slim_user))
    }
}

impl From<UserData> for InsertableUser {
    fn from(user_data: UserData) -> Self {

        let updated_at = chrono::Utc::now().naive_utc();

        let UserData {
            name,
            email,
            password,
            role,
            authority_id,
            ..
        } = user_data;

        let hash = hash_password(&password)
            .expect("Unable to hash password")
            .to_string();

        Self {
            email,
            hash,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at,
            name,
            role,
            access_key: "".to_owned(),
            access_level: "detailed".to_owned(),
            approved_by_user_uid: None,
            authority_id,
        }
    }
}

impl From<User> for SlimUser {
    fn from(user: User) -> Self {
        let User {
            id,
            email,
            role,
            access_level,
            authority_id,
            ..
        } = user;

        Self {
            id,
            email,
            role,
            access_level,
            authority_id,
        }
    }
}

#[derive(Debug, Deserialize, InputObject)]
pub struct LoginQuery {
    pub email: String,
    pub password: String,
}