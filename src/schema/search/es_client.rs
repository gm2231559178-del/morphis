use std::sync::Arc;

use serde_json::Value;

use identity_auth::circuit_breaker::CircuitBreaker;

/// Error raised by an ES client adapter.
#[derive(Debug)]
pub(crate) struct EsError(pub(crate) String);

impl std::fmt::Display for EsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Seam between the search module and Elasticsearch.
///
/// Two adapters implement the same behaviour: a live HTTP client in production
/// and an in-memory stub in tests. Both return the raw `_search` response body
/// so hit extraction stays in one place.
#[derive(Clone)]
pub(crate) enum EsClient {
    Live(LiveEsClient),
    #[cfg_attr(not(test), allow(dead_code))]
    Stub(StubEsClient),
    Unavailable,
}

impl EsClient {
    pub(crate) fn live(client: reqwest::Client, url: String, breaker: CircuitBreaker) -> Self {
        EsClient::Live(LiveEsClient {
            client,
            url,
            breaker,
        })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn stub(docs: Vec<Value>) -> Self {
        EsClient::Stub(StubEsClient {
            docs: Arc::new(docs),
        })
    }

    pub(crate) fn unavailable() -> Self {
        EsClient::Unavailable
    }

    pub(crate) async fn search(&self, index: &str, body: &Value) -> Result<Value, EsError> {
        match self {
            EsClient::Live(live) => live.search(index, body).await,
            EsClient::Stub(stub) => Ok(stub.search(body)),
            EsClient::Unavailable => Err(EsError("Elasticsearch not configured".into())),
        }
    }
}

#[derive(Clone)]
pub(crate) struct LiveEsClient {
    client: reqwest::Client,
    url: String,
    breaker: CircuitBreaker,
}

impl LiveEsClient {
    async fn search(&self, index: &str, body: &Value) -> Result<Value, EsError> {
        let url = format!("{}/{}/_search", self.url.trim_end_matches('/'), index);
        tracing::debug!(url = %url, body = %body, "ES request body");
        let client = self.client.clone();
        let resp = self
            .breaker
            .call(move || {
                let client = client.clone();
                async move { client.post(&url).json(body).send().await }
            })
            .await
            .map_err(|e| EsError(format!("ES request failed: {}", e)))?;
        resp.json::<Value>()
            .await
            .map_err(|e| EsError(format!("ES parse failed: {}", e)))
    }
}

#[derive(Clone)]
pub(crate) struct StubEsClient {
    docs: Arc<Vec<Value>>,
}

impl StubEsClient {
    /// Evaluates the generated bool query against a fixed document set and
    /// returns a response body shaped like a real `_search` result.
    fn search(&self, body: &Value) -> Value {
        let bool_body = &body["query"]["bool"];
        let size = body.get("size").and_then(Value::as_u64).unwrap_or(u64::MAX);
        let from = body.get("from").and_then(Value::as_u64).unwrap_or(0);
        let matched: Vec<Value> = self
            .docs
            .iter()
            .filter(|doc| bool_matches(bool_body, doc))
            .cloned()
            .collect();
        let hits: Vec<Value> = matched
            .iter()
            .skip(from as usize)
            .take(size as usize)
            .map(|doc| serde_json::json!({ "_source": doc }))
            .collect();
        serde_json::json!({ "hits": { "hits": hits } })
    }
}

fn bool_matches(body: &Value, doc: &Value) -> bool {
    if let Some(must) = body.get("must")
        && !clauses_match(must, doc)
    {
        return false;
    }
    if let Some(must_not) = body.get("must_not")
        && clauses_match(must_not, doc)
    {
        return false;
    }
    if let Some(should) = body.get("should") {
        let minimum = body
            .get("minimum_should_match")
            .and_then(Value::as_u64)
            .unwrap_or(1);
        if clauses_match_count(should, doc) < minimum as usize {
            return false;
        }
    }
    true
}

fn clauses_match(clauses: &Value, doc: &Value) -> bool {
    clauses
        .as_array()
        .is_some_and(|arr| arr.iter().all(|c| clause_matches(c, doc)))
}

fn clauses_match_count(clauses: &Value, doc: &Value) -> usize {
    clauses
        .as_array()
        .map(|arr| arr.iter().filter(|c| clause_matches(c, doc)).count())
        .unwrap_or(0)
}

fn clause_matches(clause: &Value, doc: &Value) -> bool {
    let Some(obj) = clause.as_object() else {
        return false;
    };
    if let Some(term) = obj.get("term").and_then(Value::as_object) {
        return term.iter().all(|(field, val)| {
            doc_at_path(doc, field.as_str()).is_some_and(|d| values_equal(d, val))
        });
    }
    if let Some(terms) = obj.get("terms").and_then(Value::as_object) {
        return terms.iter().all(|(field, val)| {
            let Some(hay) = val.as_array() else {
                return false;
            };
            doc_at_path(doc, field.as_str()).is_some_and(|d| hay.iter().any(|v| values_equal(d, v)))
        });
    }
    if let Some(match_phrase) = obj.get("match_phrase").and_then(Value::as_object) {
        return match_phrase.iter().all(|(field, val)| {
            let needle = val.as_str().unwrap_or("");
            doc_at_path(doc, field.as_str())
                .and_then(Value::as_str)
                .is_some_and(|s| s.contains(needle))
        });
    }
    if let Some(prefix) = obj.get("prefix").and_then(Value::as_object) {
        return prefix.iter().all(|(field, val)| {
            let needle = val.as_str().unwrap_or("");
            doc_at_path(doc, field.as_str())
                .and_then(Value::as_str)
                .is_some_and(|s| s.starts_with(needle))
        });
    }
    if let Some(wildcard) = obj.get("wildcard").and_then(Value::as_object) {
        return wildcard.iter().all(|(field, val)| {
            let pattern = val.as_str().unwrap_or("");
            let needle = pattern.trim_start_matches('*').trim_end_matches('*');
            doc_at_path(doc, field.as_str())
                .and_then(Value::as_str)
                .is_some_and(|s| {
                    if pattern.starts_with('*') && !pattern.ends_with('*') {
                        s.ends_with(needle)
                    } else if !pattern.starts_with('*') && pattern.ends_with('*') {
                        s.starts_with(needle)
                    } else {
                        s.contains(needle)
                    }
                })
        });
    }
    if let Some(range) = obj.get("range").and_then(Value::as_object) {
        return range.iter().all(|(field, bounds)| {
            let Some(doc_val) = doc_at_path(doc, field.as_str()).and_then(Value::as_f64) else {
                return false;
            };
            bounds.as_object().is_some_and(|b| {
                b.iter().all(|(op, bound)| {
                    let bound_val = bound.as_f64().unwrap_or(f64::MAX);
                    match op.as_str() {
                        "gt" => doc_val > bound_val,
                        "gte" => doc_val >= bound_val,
                        "lt" => doc_val < bound_val,
                        "lte" => doc_val <= bound_val,
                        _ => true,
                    }
                })
            })
        });
    }
    if let Some(inner) = obj.get("bool") {
        return bool_matches(inner, doc);
    }
    if let Some(multi_match) = obj.get("multi_match").and_then(Value::as_object) {
        let needle = multi_match.get("query").and_then(Value::as_str).unwrap_or("");
        let fields = multi_match
            .get("fields")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        return fields.iter().any(|field| {
            doc_at_path(doc, field)
                .and_then(Value::as_str)
                .is_some_and(|s| s.to_lowercase().contains(&needle.to_lowercase()))
        });
    }
    false
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(an), Value::Number(bn)) => an.as_f64() == bn.as_f64(),
        (Value::String(as_), Value::String(bs)) => as_ == bs,
        (Value::Bool(ab), Value::Bool(bb)) => ab == bb,
        _ => a == b,
    }
}

/// Traverses a dotted field path through nested objects and arrays. A trailing
/// `keyword` segment is skipped on scalars, mirroring how ES `_source` stores
/// the raw value while `field.keyword` addresses the mapped keyword sub-field.
fn doc_at_path<'a>(doc: &'a Value, path: &str) -> Option<&'a Value> {
    let (head, rest) = match path.split_once('.') {
        Some((h, r)) => (h, Some(r)),
        None => (path, None),
    };
    let node = match doc {
        Value::Object(map) => map.get(head),
        Value::Array(items) => items.iter().find_map(|item| doc_at_path(item, head)),
        Value::String(_) | Value::Number(_) | Value::Bool(_) if head == "keyword" => Some(doc),
        _ => None,
    };
    let node = node?;
    match rest {
        Some(r) => doc_at_path(node, r),
        None => Some(node),
    }
}
