use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use async_graphql::dynamic::FieldValue;
use serde_json::Value;
use sqlx::{Pool, Postgres};

use crate::config::RowFilterConfig;

use super::db;
use super::search::apply_row_filters;
use super::util::gql_val;
use super::Identity;

/// Everything the loader needs to serve one relation, shared by every parent
/// row in the same request and identity scope.
///
/// Row filters are compiled into the spec (as a SQL suffix plus bound params)
/// so a single schema-wide loader can safely batch keys from different
/// identities and tenants: keys with different filters form different groups
/// and never share a query.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RelSpec {
    /// Related table name.
    pub table: String,
    /// Foreign-key columns on the related table, in join order.
    pub foreign_fields: Vec<String>,
    /// `true` for has_many, `false` for belongs_to / has_one.
    pub is_list: bool,
    /// `t.pk1, t.pk2` ordering for has_many; empty otherwise.
    pub order_by: String,
    /// Precompiled row-filter clauses (` AND col = $2 ...`), empty when no
    /// filters apply to the current identity.
    pub filter_suffix: String,
    /// Bound params for `filter_suffix`, already offset past the FK binds.
    pub filter_params: Vec<String>,
}

/// A single parent row's request for the children of one relation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RelKey {
    pub spec: RelSpec,
    /// The FK value(s) of this parent row, one per `spec.foreign_fields`.
    pub fk: Vec<String>,
}

/// Resolves relation children for many parent rows in one batched query.
///
/// Registered once on the schema; `async-graphql`'s `DataLoader` coalesces the
/// concurrent per-row loads from one resolution level into a single `load`
/// call, so N parents cost one query per relation level instead of one query
/// per parent.
#[derive(Clone)]
pub(crate) struct RelationLoader {
    pool: Pool<Postgres>,
}

impl RelationLoader {
    pub(crate) fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

/// Compile the row-filter suffix for a relation, numbered so the FK binds
/// occupy `$1..=$fk_param_count` and the filters follow. Mirrors the per-row
/// compiler's output so batching does not change filtering behaviour.
pub(crate) fn build_filter_suffix(
    identity: &Identity,
    row_filters: &[RowFilterConfig],
    fk_param_count: usize,
) -> (String, Vec<String>) {
    let mut suffix = String::new();
    let mut params: Vec<String> = vec![String::new(); fk_param_count];
    apply_row_filters(&mut suffix, &mut params, identity, row_filters);
    let filter_params = params.split_off(fk_param_count);
    (suffix, filter_params)
}

/// Build the single SQL statement that fetches children for every key in a
/// group. FK columns are compared as text (`::text = ANY($1)`), matching the
/// batch-enrichment dialect, so int and string FKs need no cast. has_many uses
/// `json_agg` ordered by the relation's primary key; belongs_to returns the
/// same shape and the caller keeps at most one row per parent.
pub(crate) fn build_batch_query(spec: &RelSpec) -> String {
    let fk_count = spec.foreign_fields.len();
    let fk_exprs: Vec<String> = spec
        .foreign_fields
        .iter()
        .map(|f| format!("{}::text", f))
        .collect();
    let where_clause = if fk_count == 1 {
        format!("{}::text = ANY($1)", spec.foreign_fields[0])
    } else {
        let array_params = (1..=fk_count)
            .map(|i| format!("${}::text[]", i))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "({}) IN (SELECT * FROM unnest({}))",
            fk_exprs.join(", "),
            array_params
        )
    };
    let order = if spec.is_list {
        format!(" ORDER BY {}", spec.order_by)
    } else {
        String::new()
    };
    format!(
        "SELECT ARRAY[{fk_list}] AS __fk, COALESCE(json_agg(row_to_json(t){order}), '[]'::json)::text AS __children FROM (SELECT * FROM {table} WHERE {where_clause}{filter_suffix}) t GROUP BY ARRAY[{fk_list}]",
        fk_list = fk_exprs.join(", "),
        table = spec.table,
        where_clause = where_clause,
        filter_suffix = spec.filter_suffix,
        order = order,
    )
}

/// Shape loaded children into the GraphQL field value: a list for has_many, the
/// first row (or null) for belongs_to / has_one.
pub(crate) fn relation_field_value(items: Vec<Value>, is_list: bool) -> Option<FieldValue<'static>> {
    if is_list {
        let items: Vec<FieldValue> = items
            .into_iter()
            .map(|r| FieldValue::value(gql_val(r)))
            .collect();
        Some(FieldValue::list(items))
    } else {
        match items.into_iter().next() {
            Some(row) => Some(FieldValue::value(gql_val(row))),
            None => FieldValue::NONE,
        }
    }
}

impl Loader<RelKey> for RelationLoader {
    type Value = Vec<Value>;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[RelKey]) -> Result<HashMap<RelKey, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut groups: Vec<(&RelSpec, Vec<&RelKey>)> = Vec::new();
        for key in keys {
            match groups.iter_mut().find(|(spec, _)| **spec == key.spec) {
                Some((_, keys_in_group)) => keys_in_group.push(key),
                None => groups.push((&key.spec, vec![key])),
            }
        }

        let mut out = HashMap::new();
        for (spec, group_keys) in &groups {
            let sql = build_batch_query(spec);
            let fk_count = spec.foreign_fields.len();
            let mut fk_cols: Vec<Vec<String>> = vec![Vec::new(); fk_count];
            for key in group_keys {
                for (i, fk) in key.fk.iter().enumerate() {
                    fk_cols[i].push(fk.clone());
                }
            }

            let mut binds: Vec<db::Bind<'_>> = fk_cols
                .iter()
                .map(|col| db::Bind::Array(col.as_slice()))
                .collect();
            binds.extend(
                spec.filter_params
                    .iter()
                    .map(|p| db::Bind::Text(p.as_str())),
            );

            tracing::debug!(
                keys = group_keys.len(),
                queries = groups.len(),
                "relation_batch"
            );

            let rows = db::fetch_fk_groups(&self.pool, &sql, &binds).await?;
            let mut by_fk: HashMap<Vec<String>, Vec<Value>> = HashMap::new();
            for (fk, children) in rows {
                by_fk.insert(fk, children);
            }
            for key in group_keys {
                let children = by_fk.remove(&key.fk).unwrap_or_default();
                out.insert((*key).clone(), children);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(table: &str, foreign: &str, is_list: bool) -> RelSpec {
        RelSpec {
            table: table.into(),
            foreign_fields: vec![foreign.into()],
            is_list,
            order_by: if is_list { "t.pk1, t.pk2".into() } else { String::new() },
            filter_suffix: String::new(),
            filter_params: vec![],
        }
    }

    #[test]
    fn batch_query_single_fk_uses_any() {
        let s = spec("sizes", "mat_no", true);
        let sql = build_batch_query(&s);
        assert!(sql.contains("FROM (SELECT * FROM sizes WHERE mat_no::text = ANY($1)) t"));
        assert!(sql.contains("AS __fk"));
        assert!(sql.contains("json_agg(row_to_json(t) ORDER BY t.pk1, t.pk2)"));
    }

    #[test]
    fn batch_query_belongs_to_has_no_order() {
        let s = spec("materials", "mat_no", false);
        let sql = build_batch_query(&s);
        assert!(sql.contains("WHERE mat_no::text = ANY($1)"));
        assert!(!sql.contains("ORDER BY"), "belongs_to must not order");
    }

    #[test]
    fn batch_query_composite_fk_uses_unnest() {
        let s = RelSpec {
            table: "items".into(),
            foreign_fields: vec!["a".into(), "b".into()],
            is_list: true,
            order_by: "t.pk1".into(),
            filter_suffix: String::new(),
            filter_params: vec![],
        };
        let sql = build_batch_query(&s);
        assert!(sql.contains(
            "(a::text, b::text) IN (SELECT * FROM unnest($1::text[], $2::text[]))"
        ));
        assert!(sql.contains("GROUP BY ARRAY[a::text, b::text]"));
    }

    #[test]
    fn batch_query_embeds_filter_suffix_after_fk() {
        let mut s = spec("sizes", "mat_no", true);
        s.filter_suffix = " AND tenant_id = $2".into();
        s.filter_params = vec!["tenant-a".into()];
        let sql = build_batch_query(&s);
        assert!(sql.contains("WHERE mat_no::text = ANY($1) AND tenant_id = $2"));
    }

    #[test]
    fn batch_query_emits_one_statement_for_many_keys() {
        let s = spec("sizes", "mat_no", true);
        let sql = build_batch_query(&s);
        assert_eq!(sql.matches("ANY(").count(), 1, "one batch query, one ANY");
        assert_eq!(sql.matches("WHERE").count(), 1, "one WHERE clause");
    }

    fn identity_with(header: &str, value: &str) -> Identity {
        let mut headers = std::collections::HashMap::new();
        headers.insert(header.to_string(), value.to_string());
        Identity::from_raw(headers)
    }

    fn column_filter() -> RowFilterConfig {
        RowFilterConfig::ColumnFilter {
            column: "tenant_id".into(),
            from_header: "X-Tenant-ID".into(),
            auto_set: false,
        }
    }

    #[test]
    fn filter_suffix_offsets_params_after_fk_binds() {
        let identity = identity_with("x-tenant-id", "tenant-a");
        let (suffix, params) = build_filter_suffix(&identity, &[column_filter()], 1);
        assert_eq!(suffix, " AND tenant_id = $2");
        assert_eq!(params, vec!["tenant-a"]);
    }

    #[test]
    fn filter_suffix_offsets_composite_fk() {
        let identity = identity_with("x-tenant-id", "tenant-b");
        let (suffix, params) = build_filter_suffix(&identity, &[column_filter()], 2);
        assert_eq!(suffix, " AND tenant_id = $3");
        assert_eq!(params, vec!["tenant-b"]);
    }

    #[test]
    fn filter_suffix_noop_without_identity_match() {
        let (suffix, params) = build_filter_suffix(&Identity::default(), &[column_filter()], 1);
        assert_eq!(suffix, "");
        assert!(params.is_empty());
    }

    #[test]
    fn field_value_list_wraps_rows() {
        let items = vec![serde_json::json!({"a": 1}), serde_json::json!({"a": 2})];
        let fv = relation_field_value(items, true);
        assert!(fv.is_some(), "list stays a list");
    }

    #[test]
    fn field_value_belongs_to_first_row_only() {
        let items = vec![serde_json::json!({"mat_no": "M001"})];
        let fv = relation_field_value(items, false);
        assert!(fv.is_some(), "single row kept");
    }

    #[test]
    fn field_value_belongs_to_none_when_empty() {
        let fv = relation_field_value(vec![], false);
        assert!(fv.is_none(), "no row -> null");
    }

    #[test]
    fn field_value_list_empty_stays_empty_list() {
        let fv = relation_field_value(vec![], true);
        assert!(fv.is_some(), "empty list stays list");
    }

    #[tokio::test]
    async fn load_with_empty_keys_short_circuits_without_db() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://user:pass@localhost:5432/morphis")
            .unwrap();
        let loader = RelationLoader::new(pool);
        let result = loader.load(&[]).await;
        assert_eq!(result.unwrap(), HashMap::new());
    }
}
