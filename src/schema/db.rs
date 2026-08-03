use sqlx::{Pool, Postgres, Row};

/// A single SQL parameter, in the shape the SQL expects.
pub(crate) enum Bind<'a> {
    /// A scalar bound as text.
    Text(&'a str),
    /// Bound as a Postgres array (`= ANY($n)`).
    Array(&'a [String]),
}

pub(crate) fn text_binds(params: &[String]) -> Vec<Bind<'_>> {
    params.iter().map(|p| Bind::Text(p.as_str())).collect()
}

/// Run a query whose single output column holds a JSON document, and parse it.
///
/// Returns `Ok(None)` when the query matched no rows. Row-list queries come
/// back via the `COALESCE(json_agg(row_to_json(t) ORDER BY pk), '[]'::json)`
/// dialect as a JSON array.
pub(crate) async fn fetch_json(
    pool: &Pool<Postgres>,
    sql: &str,
    binds: &[Bind<'_>],
) -> Result<Option<serde_json::Value>, async_graphql::Error> {
    let mut query = sqlx::query(sql);
    for bind in binds {
        match bind {
            Bind::Text(s) => {
                query = query.bind(s);
            }
            Bind::Array(v) => {
                query = query.bind(v);
            }
        }
    }
    let row = query.fetch_optional(pool).await.map_err(|e| {
        let msg = e.to_string();
        tracing::error!(error = %msg, sql_preview = %sql.chars().take(200).collect::<String>(), "DB query failed");
        async_graphql::Error::new(msg)
    })?;
    let Some(row) = row else {
        return Ok(None);
    };
    let json_str: String = row
        .try_get(0)
        .map_err(|e| async_graphql::Error::new(e.to_string()))?;
    serde_json::from_str(&json_str)
        .map(Some)
        .map_err(|e| async_graphql::Error::new(e.to_string()))
}

/// `fetch_json` normalised into a row list (the `json_agg` / `COALESCE` shape).
pub(crate) async fn fetch_rows(
    pool: &Pool<Postgres>,
    sql: &str,
    binds: &[Bind<'_>],
) -> Result<Vec<serde_json::Value>, async_graphql::Error> {
    Ok(match fetch_json(pool, sql, binds).await? {
        Some(serde_json::Value::Array(arr)) => arr,
        Some(val) => vec![val],
        None => vec![],
    })
}
