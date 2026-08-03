use async_graphql::dynamic::{InputObject, InputValue, TypeRef, ValueAccessor};

use crate::config::TableConfig;

pub(crate) fn build_create_input(name: &str, table_config: &TableConfig) -> InputObject {
    build_input_object(&format!("Create{}Input", name), table_config, false)
}

pub(crate) fn build_update_input(name: &str, table_config: &TableConfig) -> InputObject {
    build_input_object(&format!("Update{}Input", name), table_config, true)
}

pub(crate) fn build_input_object(
    name: &str,
    table_config: &TableConfig,
    all_nullable: bool,
) -> InputObject {
    let mut input = InputObject::new(name);
    for col in &table_config.columns {
        let is_pk = table_config.primary_key.contains(&col.name);
        if !all_nullable && is_pk && col.auto_increment {
            continue;
        }
        let nullable = all_nullable || col.nullable;
        let scalar = match col.col_type.to_string().as_str() {
            "Int" => TypeRef::INT,
            "Int64" => "BigInt",
            "Float" => TypeRef::FLOAT,
            "Boolean" => TypeRef::BOOLEAN,
            _ => TypeRef::STRING,
        };
        let type_ref = if nullable {
            TypeRef::named(scalar)
        } else {
            TypeRef::named_nn(scalar)
        };
        input = input.field(InputValue::new(col.name.clone(), type_ref));
    }
    input
}

pub(crate) fn build_filter_input(name: &str, table_config: &TableConfig) -> InputObject {
    let mut input = InputObject::new(format!("{}FilterInput", name));
    for col in &table_config.columns {
        let scalar = match col.col_type.to_string().as_str() {
            "Int" => TypeRef::INT,
            "Int64" => "BigInt",
            "Float" => TypeRef::FLOAT,
            "Boolean" => TypeRef::BOOLEAN,
            _ => TypeRef::STRING,
        };
        input = input.field(InputValue::new(col.name.clone(), TypeRef::named(scalar)));
    }
    input
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FilterValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

pub(crate) fn build_filter_clauses(
    pairs: Vec<(String, FilterValue)>,
    allowed_columns: &[String],
) -> (String, Vec<String>) {
    let mut clauses = Vec::new();
    let mut params = Vec::new();

    for (key, val) in pairs {
        if !allowed_columns.contains(&key) {
            continue;
        }
        let param = match val {
            FilterValue::String(s) => s,
            FilterValue::Int(n) => n.to_string(),
            FilterValue::Float(f) => f.to_string(),
            FilterValue::Bool(b) => b.to_string(),
        };
        clauses.push(format!("{} = ${}", key, params.len() + 1));
        params.push(param);
    }

    (clauses.join(" AND "), params)
}

pub(crate) fn build_filter_sql(
    filter: ValueAccessor,
    allowed_columns: &[String],
) -> (String, Vec<String>) {
    let obj = match filter.object() {
        Ok(o) => o,
        Err(_) => return (String::new(), vec![]),
    };

    let mut pairs = Vec::new();
    for (key, val) in obj.iter() {
        if val.is_null() {
            continue;
        }
        let value = if let Ok(s) = val.string() {
            FilterValue::String(s.to_string())
        } else if let Ok(n) = val.i64() {
            FilterValue::Int(n)
        } else if let Ok(n) = val.f64() {
            FilterValue::Float(n)
        } else if let Ok(b) = val.boolean() {
            FilterValue::Bool(b)
        } else {
            continue;
        };
        pairs.push((key.to_string(), value));
    }

    build_filter_clauses(pairs, allowed_columns)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allowed(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_string_clause() {
        let pairs = vec![("name".to_string(), FilterValue::String("test".into()))];
        let (sql, params) = build_filter_clauses(pairs, &allowed(&["name"]));
        assert_eq!(sql, "name = $1");
        assert_eq!(params, vec!["test"]);
    }

    #[test]
    fn test_int_clause() {
        let pairs = vec![("feature_id".to_string(), FilterValue::Int(3))];
        let (sql, params) = build_filter_clauses(pairs, &allowed(&["feature_id"]));
        assert_eq!(sql, "feature_id = $1");
        assert_eq!(params, vec!["3"]);
    }

    #[test]
    fn test_float_clause() {
        let pairs = vec![("price".to_string(), FilterValue::Float(3.5))];
        let (sql, params) = build_filter_clauses(pairs, &allowed(&["price"]));
        assert_eq!(sql, "price = $1");
        assert_eq!(params, vec!["3.5"]);
    }

    #[test]
    fn test_bool_clause() {
        let pairs = vec![("active".to_string(), FilterValue::Bool(true))];
        let (sql, params) = build_filter_clauses(pairs, &allowed(&["active"]));
        assert_eq!(sql, "active = $1");
        assert_eq!(params, vec!["true"]);
    }

    #[test]
    fn test_bind_order_across_types() {
        let pairs = vec![
            ("name".to_string(), FilterValue::String("n".into())),
            ("feature_id".to_string(), FilterValue::Int(7)),
            ("flag".to_string(), FilterValue::Bool(false)),
        ];
        let (sql, params) = build_filter_clauses(pairs, &allowed(&["name", "feature_id", "flag"]));
        assert_eq!(sql, "name = $1 AND feature_id = $2 AND flag = $3");
        assert_eq!(params, vec!["n", "7", "false"]);
    }

    #[test]
    fn test_unknown_columns_skipped() {
        let pairs = vec![
            ("name".to_string(), FilterValue::String("n".into())),
            ("INJECTION".to_string(), FilterValue::String("evil".into())),
        ];
        let (sql, params) = build_filter_clauses(pairs, &allowed(&["name"]));
        assert_eq!(sql, "name = $1");
        assert_eq!(params, vec!["n"]);
    }

    #[test]
    fn test_empty_filter() {
        let (sql, params) = build_filter_clauses(vec![], &allowed(&["name"]));
        assert_eq!(sql, "");
        assert!(params.is_empty());
    }
}
