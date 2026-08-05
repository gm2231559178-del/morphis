use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;

use axum::http::{HeaderValue, Request, header};
use axum::middleware;
use axum::response::Response;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::service::RequestContext;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler, tool, tool_router};

use crate::config::Config;
use crate::schema::Identity;

/// Cache for `graphql_schema` results — schema is static for the server lifetime.
static SCHEMA_CACHE: OnceLock<String> = OnceLock::new();

// ── Shared state for auth middleware ────────────────────────────

#[derive(Clone)]
pub struct MCPState {
    pub authenticator: Option<identity_auth::Authenticator>,
}

// ── MCP Server ─────────────────────────────────────────────────

pub struct MorphisMCPServer {
    config: Arc<Config>,
    schema: async_graphql::dynamic::Schema,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl MorphisMCPServer {
    pub fn new(config: Arc<Config>, schema: async_graphql::dynamic::Schema) -> Self {
        Self {
            config,
            schema,
            tool_router: Self::tool_router(),
        }
    }

    fn col_info(&self, table_name: &str) -> Option<TableSchema> {
        let cfg = self.config.tables.get(table_name)?;
        let mut columns = Vec::new();
        for col in &cfg.columns {
            columns.push(ColumnSchema {
                name: col.name.clone(),
                col_type: col.col_type.to_string(),
                nullable: col.nullable,
                prompt: col.prompt.clone(),
                examples: col.examples.clone(),
                is_pk: cfg.primary_key.contains(&col.name),
            });
        }
        let search_indexes: Vec<String> = self
            .config
            .search_indexes
            .iter()
            .filter(|si| si.graphql_type == *table_name)
            .map(|si| si.name.clone())
            .collect();
        let relations: Vec<RelationSchema> = cfg
            .relations
            .iter()
            .map(|r| RelationSchema {
                name: r.name.clone(),
                rel_type: format!("{:?}", r.rel_type).to_lowercase(),
                table: r.table.clone(),
                local_field: r.local_field.clone(),
                foreign_field: r.foreign_field.clone(),
            })
            .collect();
        Some(TableSchema {
            db_table: cfg.table.clone(),
            prompt: cfg.prompt.clone(),
            columns,
            search_indexes,
            common_queries: cfg.common_queries.clone(),
            relations,
        })
    }

    /// Discover available tables with progressive detail.
    /// Call with no args (or detail: false) for a lightweight overview — table names, prompts,
    /// relations, and search indexes. Call with detail: true to get full column info
    /// (types, prompts, examples, nullable, primary key flags).
    #[tool(
        description = "Discover available tables, their prompts, relations, and search indexes. Always call this first. Pass detail:true to also get full column types, prompts, and examples for every table."
    )]
    async fn discover_tables(
        &self,
        Parameters(args): Parameters<DiscoverTablesArgs>,
    ) -> Result<CallToolResult, McpError> {
        tracing::debug!(detail = args.detail, "MCP discover_tables called");
        let mut tables = serde_json::Map::new();
        for name in self.config.tables.keys() {
            if let Some(info) = self.col_info(name) {
                let mut obj = serde_json::Map::new();
                obj.insert("db_table".into(), serde_json::json!(info.db_table));
                obj.insert("prompt".into(), serde_json::json!(info.prompt));

                if args.detail {
                    let cols: Vec<serde_json::Value> = info
                        .columns
                        .iter()
                        .map(|c| {
                            let mut m = serde_json::Map::new();
                            m.insert("name".into(), serde_json::json!(c.name));
                            m.insert("type".into(), serde_json::json!(c.col_type));
                            m.insert("nullable".into(), serde_json::json!(c.nullable));
                            m.insert("primary_key".into(), serde_json::json!(c.is_pk));
                            if let Some(p) = &c.prompt {
                                m.insert("prompt".into(), serde_json::json!(p));
                            }
                            if let Some(ex) = &c.examples {
                                m.insert("examples".into(), serde_json::json!(ex));
                            }
                            serde_json::Value::Object(m)
                        })
                        .collect();
                    obj.insert("columns".into(), serde_json::Value::Array(cols));
                } else {
                    let col_names: Vec<String> =
                        info.columns.iter().map(|c| c.name.clone()).collect();
                    obj.insert("columns".into(), serde_json::json!(col_names));
                }

                obj.insert(
                    "search_indexes".into(),
                    serde_json::json!(info.search_indexes),
                );
                let rels: Vec<serde_json::Value> = info
                    .relations
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "name": r.name,
                            "type": r.rel_type,
                            "table": r.table,
                            "local_field": r.local_field,
                            "foreign_field": r.foreign_field,
                        })
                    })
                    .collect();
                if !rels.is_empty() {
                    obj.insert("relations".into(), serde_json::Value::Array(rels));
                }
                let cqs: Vec<serde_json::Value> = info
                    .common_queries
                    .iter()
                    .map(|cq| {
                        serde_json::json!({
                            "description": cq.description,
                            "tool": cq.tool,
                            "params": cq.params,
                        })
                    })
                    .collect();
                if !cqs.is_empty() {
                    obj.insert("common_queries".into(), serde_json::Value::Array(cqs));
                }
                tables.insert(name.clone(), serde_json::Value::Object(obj));
            }
        }
        let mut result = serde_json::Map::new();
        result.insert("tables".into(), serde_json::Value::Object(tables));
        let system_prompt = self
            .config
            .mcp
            .as_ref()
            .and_then(|m| m.prompts.as_ref())
            .and_then(|p| p.system.clone());
        let query_guidance = self
            .config
            .mcp
            .as_ref()
            .and_then(|m| m.prompts.as_ref())
            .and_then(|p| p.query_guidance.clone());
        if let Some(sp) = system_prompt {
            result.insert("system_prompt".into(), serde_json::json!(sp));
        }
        if let Some(qg) = query_guidance {
            result.insert("query_guidance".into(), serde_json::json!(qg));
        }
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).map_err(|e| {
                McpError::internal_error(
                    format!("Failed to serialize response: {}", e),
                    None::<serde_json::Value>,
                )
            })?,
        )]))
    }

    /// Execute a GraphQL query against the built-in endpoint.
    /// Supports nested relations, filtering, ordering, pagination, and mutations.
    ///
    /// Examples:
    ///   { materials(limit: 3) { mat_no name status } }
    ///   { materials(id: "M001") { mat_no name sizes { size_code } colorways { hex } } }
    ///   { materialsList(filter: { status: "active" }) { mat_no name material_features { feature_name } } }
    ///   mutation { createMaterials(input: { mat_no: "NEW01", name: "New", status: "active" }) { mat_no } }
    #[tool(
        description = "Execute any GraphQL query against the API. Supports nested relations, filtering, pagination, and mutations. Example: { materialsList(limit: 3) { mat_no name sizes { size_code } } }"
    )]
    #[tracing::instrument(skip(self, ctx), fields(query_preview = %args.query.chars().take(80).collect::<String>()))]
    async fn graphql(
        &self,
        Parameters(args): Parameters<GraphqlArgs>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let mut request = async_graphql::Request::new(args.query);
        if let Some(vars) = args.variables {
            request = request.variables(async_graphql::Variables::from_json(vars));
        }

        let identity = identity_from_extensions(&ctx.extensions);
        let resp = crate::schema::execute(&self.schema, request, identity).await;

        let value = serde_json::to_value(&resp).map_err(|e| {
            McpError::internal_error(
                format!("Failed to serialize GraphQL response: {}", e),
                None::<serde_json::Value>,
            )
        })?;

        // Surface GraphQL errors as tool errors so the LLM gets clear feedback
        if let Some(errors) = value.get("errors") {
            tracing::warn!(
                errors = %serde_json::to_string_pretty(errors).unwrap_or_default(),
                "MCP graphql returned errors"
            );
            return Err(McpError::internal_error(
                format!(
                    "GraphQL errors: {}",
                    serde_json::to_string_pretty(errors).unwrap_or_default()
                ),
                None::<serde_json::Value>,
            ));
        }
        let formatted = serde_json::to_string_pretty(&value).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    /// Introspect the GraphQL schema and return every available query with its arguments, return type, and nested fields.
    /// Use this to learn the exact query names, filter inputs, and relation fields before calling graphql.
    /// Returns JSON with query name, description, arguments (name/type/description), return type, and nested fields.
    /// Example response for a query: { "query": "materialsList", "arguments": [ { "name": "filter", "type": "MaterialsFilterInput" }, ... ], "return_type": "[Materials!]!", "nested_fields": ["mat_no", "name", "sizes", ...] }
    #[tool(
        description = "Get the GraphQL schema: all query names, filter arguments, return types, and nested fields. Call this before graphql to learn the exact query syntax."
    )]
    async fn graphql_schema(&self) -> Result<CallToolResult, McpError> {
        // Return cached result — schema is static at runtime
        if let Some(cached) = SCHEMA_CACHE.get() {
            tracing::trace!("MCP graphql_schema cache hit");
            return Ok(CallToolResult::success(vec![Content::text(cached.clone())]));
        }
        tracing::debug!("MCP graphql_schema: building schema (cache miss)");

        let introspect_query = r#"
        {
          __schema {
            queryType {
              fields {
                name
                description
                args {
                  name
                  description
                  type { name kind ofType { name kind ofType { name kind } } }
                }
                type { name kind ofType { name kind ofType { name kind ofType { name kind } } } }
              }
            }
            types { name kind fields { name } }
          }
        }
        "#;

        let request = async_graphql::Request::new(introspect_query);
        let resp = crate::schema::execute(&self.schema, request, Identity::default()).await;
        let data = serde_json::to_value(&resp.data).map_err(|e| {
            McpError::internal_error(
                format!("Failed to serialize introspection response: {}", e),
                None::<serde_json::Value>,
            )
        })?;

        let fields = data["__schema"]["queryType"]["fields"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let mut type_fields: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        if let Some(types) = data["__schema"]["types"].as_array() {
            for t in types {
                let tname = t["name"].as_str().unwrap_or("");
                if tname.starts_with("__") {
                    continue;
                }
                let names: Vec<String> = t["fields"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|f| f["name"].as_str().map(|s| s.to_string()))
                            .filter(|n| !n.starts_with("__"))
                            .collect()
                    })
                    .unwrap_or_default();
                type_fields.insert(tname.to_string(), names);
            }
        }

        let mut result = Vec::new();
        for field in &fields {
            let name = field["name"].as_str().unwrap_or("");
            let desc = field["description"].as_str().unwrap_or("").to_string();
            let type_name = extract_type_name(field);

            let args: Vec<serde_json::Value> = field["args"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|a| {
                            let aname = a["name"].as_str().unwrap_or("");
                            let atype = extract_type_name(a);
                            let adesc = a["description"].as_str().unwrap_or("");
                            serde_json::json!({
                                "name": aname,
                                "type": atype,
                                "description": adesc,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            let type_name_clean = type_name
                .trim_end_matches('!')
                .trim_start_matches('[')
                .trim_end_matches(']')
                .trim_end_matches('!')
                .to_string();
            let nested: Vec<String> = type_fields
                .get(&type_name_clean)
                .cloned()
                .unwrap_or_default();

            result.push(serde_json::json!({
                "query": name,
                "description": if desc.is_empty() { serde_json::Value::Null } else { serde_json::json!(desc) },
                "return_type": type_name,
                "arguments": args,
                "nested_fields": if nested.is_empty() { serde_json::Value::Null } else { serde_json::json!(nested) },
            }));
        }

        let output = serde_json::json!({
            "graphql_queries": result,
            "note": "Use these query names and arguments in the graphql tool. Nested fields can be included in the selection set.",
        });

        let schema_json = serde_json::to_string_pretty(&output).map_err(|e| {
            McpError::internal_error(
                format!("Failed to format schema: {}", e),
                None::<serde_json::Value>,
            )
        })?;

        // Cache for subsequent calls — schema is static at runtime
        let _ = SCHEMA_CACHE.set(schema_json.clone());

        Ok(CallToolResult::success(vec![Content::text(schema_json)]))
    }
}

impl ServerHandler for MorphisMCPServer {
    fn get_info(&self) -> ServerInfo {
        let cfg = self.config.mcp.as_ref();
        let instructions = cfg
            .and_then(|m| m.prompts.as_ref().and_then(|p| p.query_guidance.clone()))
            .unwrap_or_else(|| {
                "Morphis Data MCP Server. Use discover_tables to explore available tables, \
                 graphql_schema to learn the GraphQL query syntax, \
                 and graphql to execute queries with nested relations."
                    .to_string()
            });
        InitializeResult::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(instructions)
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tool_router.get(name).cloned()
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: self.tool_router.list_all(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        if self.get_tool(&request.name).is_none() {
            tracing::warn!(tool = %request.name, "MCP call_tool: unknown tool");
            return Err(McpError::invalid_params(
                format!("Tool '{}' not found", request.name),
                None::<serde_json::Value>,
            ));
        }
        let tcc = ToolCallContext::new(self, request, context);
        self.tool_router.call(tcc).await
    }
}

// ── Parameter Structs ───────────────────────────────────────────

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DiscoverTablesArgs {
    /// When true, includes full column details (types, prompts, examples, nullable, primary key flags).
    /// When false (default), returns overview with column names only — call with detail:true to drill in.
    #[serde(default)]
    pub detail: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GraphqlArgs {
    /// GraphQL query string. Supports nested relations.
    /// Example: { materialsList(limit: 3) { mat_no name status sizes { size_code } colorways { hex } } }
    pub query: String,
    /// Optional variables for the query
    #[serde(default)]
    pub variables: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
struct RelationSchema {
    name: String,
    rel_type: String,
    table: String,
    local_field: String,
    foreign_field: String,
}

#[derive(Debug, Clone)]
struct TableSchema {
    db_table: String,
    prompt: Option<String>,
    columns: Vec<ColumnSchema>,
    search_indexes: Vec<String>,
    common_queries: Vec<crate::config::CommonQueryConfig>,
    relations: Vec<RelationSchema>,
}

#[derive(Debug, Clone)]
struct ColumnSchema {
    name: String,
    col_type: String,
    nullable: bool,
    prompt: Option<String>,
    examples: Option<Vec<String>>,
    is_pk: bool,
}

// ── Auth Middleware ─────────────────────────────────────────────

async fn mcp_auth_middleware(
    axum::extract::State(state): axum::extract::State<MCPState>,
    mut req: Request<axum::body::Body>,
    next: middleware::Next,
) -> Response {
    let identity = match &state.authenticator {
        Some(authenticator) => {
            let token = req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(|s| s.to_string());

            match token {
                Some(token) => match crate::auth::validate_identity(authenticator, &token).await {
                    Ok(identity) => {
                        tracing::trace!("MCP auth succeeded");
                        identity
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "MCP JWT validation failed");
                        return Response::builder()
                            .status(401)
                            .body(axum::body::Body::from("Unauthorized"))
                            .unwrap();
                    }
                },
                None => {
                    tracing::warn!("MCP request without Bearer token (rejected)");
                    return Response::builder()
                        .status(401)
                        .body(axum::body::Body::from("Unauthorized"))
                        .unwrap();
                }
            }
        }
        None => {
            let headers = req
                .headers()
                .iter()
                .filter_map(|(name, value)| {
                    value
                        .to_str()
                        .ok()
                        .map(|v| (name.as_str().to_lowercase(), v.to_string()))
                })
                .collect();
            Identity::from_raw(headers)
        }
    };

    // Ensure MCP requests have proper Accept header
    if req.uri().path().starts_with("/mcp") && !req.headers().contains_key(header::ACCEPT) {
        req.headers_mut().insert(
            header::ACCEPT,
            HeaderValue::from_static("application/json, text/event-stream"),
        );
    }
    // Carry the identity to the tool via the request extensions — rmcp surfaces the original
    // HTTP request parts to tools, so the tool reads it back from `ctx.extensions`.
    req.extensions_mut().insert(identity);
    next.run(req).await
}

// ── Axum Router Builder ────────────────────────────────────────

pub fn build_mcp_router(
    config: Arc<Config>,
    schema: async_graphql::dynamic::Schema,
) -> Option<axum::Router> {
    let mcp_cfg = config.mcp.as_ref()?;
    if !mcp_cfg.enabled {
        return None;
    }

    let authenticator = mcp_cfg
        .auth
        .as_ref()
        .filter(|a| a.enabled)
        .map(|a| {
            crate::auth::authenticator(
                a.jwt_secret.as_deref(),
                a.jwks_url.as_deref(),
                a.issuer.as_deref(),
                a.audience.as_deref(),
                true,
                &a.identity_mappings,
                a.jwks_url
                    .as_ref()
                    .map(|_| config.circuit_breakers.jwks.clone()),
            )
        });

    let mcp_state = MCPState { authenticator };

    let service = StreamableHttpService::new(
        {
            let config = config.clone();
            let schema = schema.clone();
            move || Ok(MorphisMCPServer::new(config.clone(), schema.clone()))
        },
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default()
            .with_stateful_mode(true)
            .with_sse_keep_alive(Some(Duration::from_secs(15)))
            .with_allowed_hosts(["localhost", "127.0.0.1", "0.0.0.0", "auth-proxy"]),
    );

    let router =
        axum::Router::new()
            .nest_service("/mcp", service)
            .layer(middleware::from_fn_with_state(
                mcp_state,
                mcp_auth_middleware,
            ));

    tracing::info!("MCP server enabled at /mcp (Streamable HTTP)");

    Some(router)
}

// ── Identity threading ─────────────────────────────────────────

/// Extract the request identity injected by the auth middleware from the tool's
/// request context. rmcp injects the original `http::request::Parts` (which carries
/// the axum extensions) into every tool request's `RequestContext`.
fn identity_from_extensions(extensions: &rmcp::model::Extensions) -> Identity {
    extensions
        .get::<axum::http::request::Parts>()
        .and_then(|parts| parts.extensions.get::<Identity>())
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_extracted_from_request_extensions() {
        let (mut parts, _) = axum::http::Request::builder()
            .uri("/mcp")
            .body(axum::body::Body::empty())
            .unwrap()
            .into_parts();
        let mut headers = std::collections::HashMap::new();
        headers.insert("x-tenant-id".to_string(), "tenant-a".to_string());
        parts.extensions.insert(Identity::from_raw(headers));

        let mut extensions = rmcp::model::Extensions::new();
        extensions.insert(parts);

        let identity = identity_from_extensions(&extensions);
        assert_eq!(identity.header_value("x-tenant-id"), Some("tenant-a"));
    }

    #[test]
    fn identity_defaults_to_anonymous_when_missing() {
        let identity = identity_from_extensions(&rmcp::model::Extensions::new());
        assert_eq!(identity.header_value("x-tenant-id"), None);
    }
}

// ── Filter Parsing ──────────────────────────────────────────────

// ── GraphQL introspection helpers ────────────────────────────────

fn extract_type_name(field: &serde_json::Value) -> String {
    let t = &field["type"];
    let kind = t["kind"].as_str().unwrap_or("");
    if kind == "NON_NULL" {
        if let Some(of) = t["ofType"].as_object() {
            let inner_kind = of.get("kind").and_then(|k| k.as_str()).unwrap_or("");
            if inner_kind == "LIST" {
                let inner_name = resolve_named_type(&t["ofType"]["ofType"]);
                format!("[{}]!", inner_name)
            } else {
                let name = of.get("name").and_then(|n| n.as_str()).unwrap_or("");
                format!("{}!", name)
            }
        } else {
            "unknown".to_string()
        }
    } else if kind == "LIST" {
        let inner_name = resolve_named_type(&t["ofType"]);
        format!("[{}]", inner_name)
    } else {
        t.get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string()
    }
}

fn resolve_named_type(t: &serde_json::Value) -> String {
    if let Some(name) = t["name"].as_str()
        && !name.is_empty()
    {
        return name.to_string();
    }
    if let Some(of) = t["ofType"].as_object() {
        return resolve_named_type(&serde_json::Value::Object(of.clone()));
    }
    "unknown".to_string()
}
