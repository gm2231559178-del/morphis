pub(crate) mod db;
mod input;
mod mutation;
mod query;
mod search;
mod table;
mod util;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use async_graphql::dynamic::{InputObject, InputValue, Schema, TypeRef};
use sqlx::{Pool, Postgres};

use crate::circuit_breaker::CircuitBreaker;
use crate::config::{
    ColumnType, Config, PermissionCache, RowFilterConfig, SearchJoinConfig, TableConfig,
};

#[derive(Clone)]
pub(crate) struct AppContext {
    pub pool: Pool<Postgres>,
    pub es_client: Option<reqwest::Client>,
    pub es_url: Option<String>,
    pub permission_cache: Arc<Mutex<PermissionCache>>,
    pub es_circuit_breaker: Option<CircuitBreaker>,
}

#[derive(Clone, Default)]
pub(crate) struct Identity {
    headers: HashMap<String, String>,
}

impl Identity {
    pub fn from_raw(headers: HashMap<String, String>) -> Self {
        Self { headers }
    }

    pub fn header_value(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(String::as_str)
    }
}

pub(crate) fn apply_row_filters(
    sql: &mut String,
    params: &mut Vec<String>,
    identity: &Identity,
    row_filters: &[RowFilterConfig],
) {
    for rf in row_filters {
        if let Some(val) = identity.header_value(rf.header_name()) {
            let clause = match rf {
                RowFilterConfig::ColumnFilter { column, .. } => {
                    if params.is_empty() {
                        format!(" WHERE {} = ${}", column, params.len() + 1)
                    } else {
                        format!(" AND {} = ${}", column, params.len() + 1)
                    }
                }
                RowFilterConfig::SubqueryFilter {
                    columns,
                    match_columns,
                    from_source,
                    user_column,
                    ..
                } => {
                    let prefix = if params.is_empty() {
                        " WHERE "
                    } else {
                        " AND "
                    };
                    format!(
                        "{} ({}) IN (SELECT {} FROM {} WHERE {} = ${})",
                        prefix,
                        columns.join(", "),
                        match_columns.join(", "),
                        from_source,
                        user_column,
                        params.len() + 1,
                    )
                }
            };
            sql.push_str(&clause);
            params.push(val.to_string());
        }
    }
}

fn build_string_operators_input() -> InputObject {
    let mut input = InputObject::new("StringOperatorsInput");
    input = input.field(InputValue::new("eq", TypeRef::named(TypeRef::STRING)));
    input = input.field(InputValue::new("ne", TypeRef::named(TypeRef::STRING)));
    input = input.field(
        InputValue::new("in", TypeRef::named_nn_list(TypeRef::STRING)),
    );
    input = input.field(
        InputValue::new("all", TypeRef::named_nn_list(TypeRef::STRING)),
    );
    input = input.field(InputValue::new("contains", TypeRef::named(TypeRef::STRING)));
    input = input.field(
        InputValue::new("starts_with", TypeRef::named(TypeRef::STRING)),
    );
    input = input.field(
        InputValue::new("ends_with", TypeRef::named(TypeRef::STRING)),
    );
    input
}

fn build_int_operators_input() -> InputObject {
    let mut input = InputObject::new("IntOperatorsInput");
    input = input.field(InputValue::new("eq", TypeRef::named(TypeRef::INT)));
    input = input.field(InputValue::new("ne", TypeRef::named(TypeRef::INT)));
    input = input.field(
        InputValue::new("in", TypeRef::named_nn_list(TypeRef::INT)),
    );
    input = input.field(
        InputValue::new("all", TypeRef::named_nn_list(TypeRef::INT)),
    );
    input = input.field(InputValue::new("gt", TypeRef::named(TypeRef::INT)));
    input = input.field(InputValue::new("gte", TypeRef::named(TypeRef::INT)));
    input = input.field(InputValue::new("lt", TypeRef::named(TypeRef::INT)));
    input = input.field(InputValue::new("lte", TypeRef::named(TypeRef::INT)));
    input
}

fn build_float_operators_input() -> InputObject {
    let mut input = InputObject::new("FloatOperatorsInput");
    input = input.field(InputValue::new("eq", TypeRef::named(TypeRef::FLOAT)));
    input = input.field(InputValue::new("ne", TypeRef::named(TypeRef::FLOAT)));
    input = input.field(
        InputValue::new("in", TypeRef::named_nn_list(TypeRef::FLOAT)),
    );
    input = input.field(
        InputValue::new("all", TypeRef::named_nn_list(TypeRef::FLOAT)),
    );
    input = input.field(InputValue::new("gt", TypeRef::named(TypeRef::FLOAT)));
    input = input.field(InputValue::new("gte", TypeRef::named(TypeRef::FLOAT)));
    input = input.field(InputValue::new("lt", TypeRef::named(TypeRef::FLOAT)));
    input = input.field(InputValue::new("lte", TypeRef::named(TypeRef::FLOAT)));
    input
}

fn build_boolean_operators_input() -> InputObject {
    let mut input = InputObject::new("BooleanOperatorsInput");
    input = input.field(InputValue::new("eq", TypeRef::named(TypeRef::BOOLEAN)));
    input = input.field(InputValue::new("ne", TypeRef::named(TypeRef::BOOLEAN)));
    input
}

fn operator_type_name(col_type: &ColumnType) -> &'static str {
    match col_type {
        ColumnType::Int | ColumnType::Int64 => "IntOperatorsInput",
        ColumnType::Float => "FloatOperatorsInput",
        ColumnType::Boolean => "BooleanOperatorsInput",
        _ => "StringOperatorsInput",
    }
}

fn lookup_column_type<'a>(
    field_name: &str,
    table_config: &'a TableConfig,
) -> Option<&'a ColumnType> {
    table_config
        .columns
        .iter()
        .find(|c| c.name == field_name)
        .map(|c| &c.col_type)
}

fn resolve_table_config<'a>(
    table_name: &str,
    tables: &'a std::collections::HashMap<String, TableConfig>,
) -> Option<&'a TableConfig> {
    tables.values().find(|t| t.table == table_name)
}

fn build_nested_search_filters(
    join_fields: &[SearchJoinConfig],
    accumulator: &mut Vec<InputObject>,
    tables: &std::collections::HashMap<String, TableConfig>,
) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    for jf in join_fields {
        let type_name = format!("{}Filter", util::capitalize_words(&jf.index_field));
        let nested = build_nested_search_filters(&jf.join_fields, accumulator, tables);
        let mut input = InputObject::new(&type_name);
        let target_table = resolve_table_config(&jf.table, tables);
        for f in &jf.searchable_fields {
            let op_type = target_table
                .and_then(|tc| lookup_column_type(f, tc))
                .map(operator_type_name)
                .unwrap_or("StringOperatorsInput");
            input = input.field(InputValue::new(f.clone(), TypeRef::named(op_type)));
        }
        for (field_name, nested_type) in nested {
            input = input.field(InputValue::new(field_name, TypeRef::named(nested_type)));
        }
        accumulator.push(input);
        fields.push((jf.index_field.clone(), type_name));
    }
    fields
}

pub async fn build_schema(config: Arc<Config>, pool: Pool<Postgres>) -> Schema {
    let es_client = config
        .elasticsearch
        .as_ref()
        .map(|_| reqwest::Client::new());
    let es_url = config.elasticsearch.as_ref().map(|c| c.url.clone());
    let es_circuit_breaker = config
        .elasticsearch
        .as_ref()
        .map(|_| CircuitBreaker::new(config.circuit_breakers.es.to_circuit_breaker_config()));
    let ctx = Arc::new(AppContext {
        pool,
        es_client,
        es_url,
        es_circuit_breaker,
        permission_cache: Arc::new(Mutex::new(PermissionCache::new())),
    });

    let mut schema_builder = Schema::build("Query", Some("Mutation"), None);
    schema_builder = schema_builder.data(ctx);

    let mut table_type_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for (name, table_config) in &config.tables {
        table_type_map.insert(table_config.table.clone(), name.clone());
    }

    let mut table_objects = Vec::new();
    for (name, table_config) in &config.tables {
        if table_config.primary_key.is_empty() {
            panic!("Table '{}' has no primary_key defined", name);
        }
        let name_caps = util::capitalize_first(name);
        let filter = input::build_filter_input(&name_caps, table_config);
        schema_builder = schema_builder.register(filter);
        let input_obj = input::build_create_input(&name_caps, table_config);
        schema_builder = schema_builder.register(input_obj);
        let update_input = input::build_update_input(&name_caps, table_config);
        schema_builder = schema_builder.register(update_input);

        let obj = table::build_table_object(name, table_config, &config.tables, &table_type_map);
        schema_builder = schema_builder.register(obj);
        table_objects.push((
            name.clone(),
            table_config.table.clone(),
            table_config.clone(),
        ));
    }

    let mut query = query::build_query_object(&config, &table_objects);

    for index_cfg in &config.search_indexes {
        tracing::debug!("Registering search index: {}", index_cfg.name);

        // Register operator input types (idempotent — same types shared across all indexes)
        schema_builder = schema_builder.register(build_string_operators_input());
        schema_builder = schema_builder.register(build_int_operators_input());
        schema_builder = schema_builder.register(build_float_operators_input());
        schema_builder = schema_builder.register(build_boolean_operators_input());

        // Build nested filter input types from join_fields
        let mut nested_filters: Vec<InputObject> = Vec::new();
        let nested_fields =
            build_nested_search_filters(&index_cfg.join_fields, &mut nested_filters, &config.tables);
        for input_obj in nested_filters {
            schema_builder = schema_builder.register(input_obj);
        }

        // Build top-level search filter input
        let sf = index_cfg.searchable_fields.clone();
        let source_table = config.tables.get(&index_cfg.graphql_type);
        let mut input_obj = InputObject::new(format!(
            "{}SearchFilter",
            util::capitalize_first(&index_cfg.index)
        ));
        for f in &sf {
            let op_type = source_table
                .and_then(|tc| lookup_column_type(f, tc))
                .map(operator_type_name)
                .unwrap_or("StringOperatorsInput");
            input_obj = input_obj.field(InputValue::new(f.clone(), TypeRef::named(op_type)));
        }
        for (field_name, type_name) in &nested_fields {
            input_obj = input_obj.field(InputValue::new(
                field_name.clone(),
                TypeRef::named(type_name.clone()),
            ));
        }
        schema_builder = schema_builder.register(input_obj);
        let search_row_filters = config
            .tables
            .get(&index_cfg.graphql_type)
            .map(|t| t.row_filters.clone())
            .unwrap_or_default();
        query = search::add_search_field(query, index_cfg, search_row_filters);
    }

    let mutation = mutation::build_mutation_object(&config, &table_objects);

    schema_builder = schema_builder.register(query);
    schema_builder = schema_builder.register(mutation);

    schema_builder.finish().unwrap()
}
