use async_graphql::*;

use crate::models::ClassificationSchema;
use uuid::Uuid;

//use crate::common_utils::{RoleGuard, is_admin, UserRole};

#[derive(Default)]
pub struct ClassificationSchemaQuery;

#[Object]
impl ClassificationSchemaQuery {
    /// Returns count of ClassificationSchemas in the system
    pub async fn classification_schema_count(&self, _context: &Context<'_>) -> Result<i64> {
        ClassificationSchema::get_count()
    }

    /// Returns a classification schema by its Uuid
    pub async fn classification_schema_by_id(&self, _context: &Context<'_>, id: Uuid) -> Result<ClassificationSchema> {
        ClassificationSchema::get_by_id(&id)
    }

    /// Returns classification schemas by creator ID
    pub async fn classification_schemas_by_creator_id(
        &self,
        _context: &Context<'_>,
        creator_id: Uuid,
    ) -> Result<Vec<ClassificationSchema>> {
        ClassificationSchema::get_by_creator_id(creator_id)
    }

    /// Returns classification schemas by nation code
    pub async fn classification_schemas_by_nation_code(
        &self,
        _context: &Context<'_>,
        nation_code: String,
    ) -> Result<Vec<ClassificationSchema>> {
        ClassificationSchema::get_by_nation_code(&nation_code)
    }

    /// Returns a specific classification schema by nation code and version
    pub async fn classification_schema_by_nation_code_and_version(
        &self,
        _context: &Context<'_>,
        nation_code: String,
        version: String,
    ) -> Result<ClassificationSchema> {
        ClassificationSchema::get_by_nation_code_and_version(&nation_code, &version)
    }

    /// Returns classification schemas by nation ID
    pub async fn classification_schemas_by_nation_id(
        &self,
        _context: &Context<'_>,
        nation_id: Uuid,
    ) -> Result<Vec<ClassificationSchema>> {
        ClassificationSchema::get_by_nation_id(&nation_id)
    }

    /// Returns the latest classification schema for a given nation code
    pub async fn classification_schema_latest_by_nation_code(
        &self,
        _context: &Context<'_>,
        nation_code: String,
    ) -> Result<ClassificationSchema> {
        ClassificationSchema::get_latest_by_nation_code(&nation_code)
    }

    /// Returns vector of all classification schemas
    pub async fn classification_schemas(&self, _context: &Context<'_>) -> Result<Vec<ClassificationSchema>> {
        ClassificationSchema::get_all()
    }
}
