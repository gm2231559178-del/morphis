pub(crate) mod db;
mod search;
mod table;
mod util;

use std::sync::Arc;

use async_graphql::dynamic::{Object, Scalar, Schema};
use sqlx::{Pool, Postgres};

use crate::config::Config;

#[derive(Clone)]
pub(crate) struct AppContext {
    pub pool: Pool<Postgres>,
}

/// Claim-derived request identity consumed by row filters and RBAC.
///
/// Re-exported from the shared `identity-auth` crate so the HTTP handler, the MCP tools and
/// the schema all operate on the same type.
pub(crate) use identity_auth::Identity;

/// Shared in-process GraphQL execution seam.
///
/// Both the HTTP `/graphql` handler and the MCP `graphql` / `graphql_schema` tools cross this
/// seam, so a query + identity yields the same result regardless of entry point. The `Identity`
/// travels explicitly from the caller into the request data, where row filters and RBAC read it.
pub(crate) async fn execute(
    schema: &Schema,
    request: async_graphql::Request,
    identity: Identity,
) -> async_graphql::Response {
    let mut request = request;
    request.data.insert(identity);
    schema.execute(request).await
}

pub async fn build_schema(config: Arc<Config>, pool: Pool<Postgres>) -> Schema {
    let search_service = Arc::new(search::SearchService::new(&config, pool.clone()));
    build_schema_with_search(config, pool, search_service).await
}

pub(crate) async fn build_schema_with_search(
    config: Arc<Config>,
    pool: Pool<Postgres>,
    search_service: Arc<search::SearchService>,
) -> Schema {
    let ctx = Arc::new(AppContext { pool });

    let mut schema_builder = Schema::build("Query", Some("Mutation"), None);
    schema_builder = schema_builder.data(ctx);
    schema_builder = schema_builder.register(Scalar::new("BigInt"));

    let mut query = Object::new("Query");
    let mut mutation = Object::new("Mutation");
    for (name, table_config) in &config.tables {
        if table_config.primary_key.is_empty() {
            panic!("Table '{}' has no primary_key defined", name);
        }
        let surface = table::build_table_surface(&config, name, table_config);
        for input_obj in surface.inputs {
            schema_builder = schema_builder.register(input_obj);
        }
        schema_builder = schema_builder.register(surface.object);
        for field in surface.query_fields {
            query = query.field(field);
        }
        for field in surface.mutation_fields {
            mutation = mutation.field(field);
        }
    }

    // Register operator input types (idempotent — same types shared across all indexes)
    for input_obj in search::operator_inputs() {
        schema_builder = schema_builder.register(input_obj);
    }

    for index_cfg in &config.search_indexes {
        tracing::debug!("Registering search index: {}", index_cfg.name);

        // Build nested filter input types from join_fields
        let (nested_filters, nested_fields) =
            search::nested_filter_inputs(index_cfg, &config.tables);
        for input_obj in nested_filters {
            schema_builder = schema_builder.register(input_obj);
        }

        // Build top-level search filter input
        schema_builder = schema_builder.register(search::build_search_filter_input(
            index_cfg,
            &config.tables,
            &nested_fields,
        ));

        let search_row_filters = config
            .tables
            .get(&index_cfg.graphql_type)
            .map(|t| t.row_filters.clone())
            .unwrap_or_default();
        query = search::add_search_field(query, index_cfg, search_row_filters, search_service.clone());
    }

    schema_builder = schema_builder.register(query);
    schema_builder = schema_builder.register(mutation);

    schema_builder.finish().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn execute_runs_introspection_in_process_without_http() {
        let config = Arc::new(crate::config::Config::from_file("config.yaml").unwrap());
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy(&config.database.url)
            .unwrap();
        let schema = build_schema(config.clone(), pool).await;

        let resp = execute(
            &schema,
            async_graphql::Request::new("{ __schema { queryType { name } } }"),
            Identity::default(),
        )
        .await;

        assert!(resp.is_ok(), "introspection returned errors");
        let data = serde_json::to_value(&resp.data).unwrap();
        assert_eq!(data["__schema"]["queryType"]["name"], "Query");
    }

    #[tokio::test]
    async fn schema_shape_matches_naming_conventions() {
        let config = Arc::new(crate::config::Config::from_file("config.yaml").unwrap());
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy(&config.database.url)
            .unwrap();
        let schema = build_schema(config.clone(), pool).await;

        let resp = execute(
            &schema,
            async_graphql::Request::new(
                "{ __schema { types { name } queryType { fields { name } } mutationType { fields { name } } } }",
            ),
            Identity::default(),
        )
        .await;

        assert!(resp.is_ok(), "introspection returned errors");
        let data = serde_json::to_value(&resp.data).unwrap();

        let types = data["__schema"]["types"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| t["name"].as_str())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for expected in [
            "materials",
            "sizes",
            "MaterialsFilterInput",
            "CreateMaterialsInput",
            "UpdateMaterialsInput",
            "CreateSizesInput",
        ] {
            assert!(types.contains(&expected), "missing type {expected}");
        }

        let query_fields = data["__schema"]["queryType"]["fields"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|f| f["name"].as_str())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for expected in ["materials", "materialsList", "sizesList"] {
            assert!(query_fields.contains(&expected), "missing query {expected}");
        }

        let mutation_fields = data["__schema"]["mutationType"]["fields"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|f| f["name"].as_str())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for expected in ["createMaterials", "updateMaterials", "deleteMaterials"] {
            assert!(
                mutation_fields.contains(&expected),
                "missing mutation {expected}"
            );
        }
        assert!(
            !mutation_fields.contains(&"createUser_permissions"),
            "crud-disabled table must not generate mutations"
        );
    }
}
