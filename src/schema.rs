// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "graphql_operation_type"))]
    pub struct GraphqlOperationType;
}

diesel::table! {
    authorities (id) {
        id -> Uuid,
        creator_id -> Uuid,
        nation_id -> Uuid,
        #[max_length = 256]
        name -> Varchar,
        #[max_length = 128]
        email -> Varchar,
        #[max_length = 32]
        phone -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        expires_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    classification_schemas (id) {
        id -> Uuid,
        creator_id -> Uuid,
        #[max_length = 3]
        nation_code -> Varchar,
        #[max_length = 128]
        to_nato_unclassified -> Varchar,
        #[max_length = 128]
        to_nato_restricted -> Varchar,
        #[max_length = 128]
        to_nato_confidential -> Varchar,
        #[max_length = 128]
        to_nato_secret -> Varchar,
        #[max_length = 128]
        to_nato_top_secret -> Varchar,
        #[max_length = 128]
        from_nato_unclassified -> Varchar,
        #[max_length = 128]
        from_nato_restricted -> Varchar,
        #[max_length = 128]
        from_nato_confidential -> Varchar,
        #[max_length = 128]
        from_nato_secret -> Varchar,
        #[max_length = 128]
        from_nato_top_secret -> Varchar,
        caveats -> Text,
        #[max_length = 32]
        version -> Varchar,
        authority_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        expires_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    conversion_requests (id) {
        id -> Uuid,
        creator_id -> Uuid,
        authority_id -> Uuid,
        data_object_id -> Uuid,
        #[max_length = 128]
        source_nation_classification -> Varchar,
        #[max_length = 3]
        source_nation_code -> Varchar,
        target_nation_codes -> Array<Nullable<Text>>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    conversion_responses (id) {
        id -> Uuid,
        conversion_request_id -> Uuid,
        subject_data_id -> Uuid,
        #[max_length = 128]
        nato_equivalent -> Varchar,
        target_nation_classifications -> Jsonb,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        expires_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    data_objects (id) {
        id -> Uuid,
        creator_id -> Uuid,
        #[max_length = 512]
        title -> Varchar,
        description -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::GraphqlOperationType;

    graphql_audit_logs (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        #[max_length = 64]
        user_role -> Nullable<Varchar>,
        #[max_length = 64]
        user_access_level -> Nullable<Varchar>,
        authority_id -> Nullable<Uuid>,
        #[max_length = 3]
        nation_code -> Nullable<Varchar>,
        operation_type -> GraphqlOperationType,
        #[max_length = 256]
        operation_name -> Nullable<Varchar>,
        query_text -> Text,
        variables_json -> Nullable<Jsonb>,
        request_id -> Nullable<Uuid>,
        #[max_length = 45]
        client_ip -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        execution_time_ms -> Nullable<Int4>,
        #[max_length = 32]
        response_status -> Varchar,
        error_message -> Nullable<Text>,
        errors_json -> Nullable<Jsonb>,
        accessed_data_objects -> Nullable<Array<Nullable<Uuid>>>,
        accessed_classifications -> Nullable<Array<Nullable<Text>>>,
        executed_at -> Timestamp,
        #[max_length = 256]
        session_id -> Nullable<Varchar>,
        request_headers -> Nullable<Jsonb>,
    }
}

diesel::table! {
    metadata (id) {
        id -> Uuid,
        data_object_id -> Uuid,
        #[max_length = 256]
        identifier -> Varchar,
        #[max_length = 512]
        authorization_reference -> Nullable<Varchar>,
        authorization_reference_date -> Nullable<Timestamp>,
        originator_organization_id -> Uuid,
        custodian_organization_id -> Uuid,
        #[max_length = 128]
        format -> Varchar,
        format_size -> Nullable<Int8>,
        #[max_length = 128]
        security_classification -> Varchar,
        releasable_to_countries -> Nullable<Array<Nullable<Text>>>,
        releasable_to_organizations -> Nullable<Array<Nullable<Text>>>,
        releasable_to_categories -> Nullable<Array<Nullable<Text>>>,
        #[max_length = 128]
        disclosure_category -> Nullable<Varchar>,
        handling_restrictions -> Nullable<Array<Nullable<Text>>>,
        #[max_length = 512]
        handling_authority -> Nullable<Varchar>,
        no_handling_restrictions -> Nullable<Bool>,
        #[max_length = 256]
        domain -> Varchar,
        tags -> Array<Nullable<Text>>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    nations (id) {
        id -> Uuid,
        creator_id -> Uuid,
        #[max_length = 3]
        nation_code -> Varchar,
        #[max_length = 128]
        nation_name -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        hash -> Varchar,
        #[max_length = 128]
        email -> Varchar,
        #[max_length = 64]
        role -> Varchar,
        #[max_length = 256]
        name -> Varchar,
        #[max_length = 64]
        access_level -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        #[max_length = 256]
        access_key -> Varchar,
        approved_by_user_uid -> Nullable<Uuid>,
        authority_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    valid_roles (role) {
        #[max_length = 64]
        role -> Varchar,
    }
}

diesel::joinable!(authorities -> nations (nation_id));
diesel::joinable!(classification_schemas -> authorities (authority_id));
diesel::joinable!(classification_schemas -> users (creator_id));
diesel::joinable!(conversion_requests -> authorities (authority_id));
diesel::joinable!(conversion_requests -> data_objects (data_object_id));
diesel::joinable!(conversion_requests -> users (creator_id));
diesel::joinable!(conversion_responses -> conversion_requests (conversion_request_id));
diesel::joinable!(conversion_responses -> data_objects (subject_data_id));
diesel::joinable!(data_objects -> users (creator_id));
diesel::joinable!(graphql_audit_logs -> authorities (authority_id));
diesel::joinable!(graphql_audit_logs -> users (user_id));
diesel::joinable!(metadata -> data_objects (data_object_id));
diesel::joinable!(nations -> users (creator_id));
diesel::joinable!(users -> valid_roles (role));

diesel::allow_tables_to_appear_in_same_query!(
    authorities,
    classification_schemas,
    conversion_requests,
    conversion_responses,
    data_objects,
    graphql_audit_logs,
    metadata,
    nations,
    users,
    valid_roles,
);
