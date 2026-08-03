mod auth;
mod circuit_breaker;
mod config;
mod db;
mod mcp;
mod schema;

use std::sync::Arc;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{Router, extract::Extension, middleware, response::Response, routing::get};
use tower_http::cors::CorsLayer;

use config::AuthConfig;
use schema::Identity;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "morphis=info".into());

    if std::env::var("LOG_FORMAT").as_deref() == Ok("json") {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .json()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .init();
    }

    let config_path = std::env::var("MORPHIS_CONFIG").unwrap_or_else(|_| "config.yaml".to_string());
    let config = Arc::new(config::Config::from_file(&config_path)?);

    tracing::info!("Loaded config with {} tables", config.tables.len());

    let pool = db::connect(&config.database).await?;

    let schema = schema::build_schema(config.clone(), pool.clone()).await;

    let auth_config = config.auth.clone().unwrap_or(AuthConfig {
        enabled: false,
        jwks_url: None,
        issuer: None,
        audience: None,
        jwt_secret: None,
        identity_mappings: vec![],
    });

    let mut app = Router::new()
        .route("/graphql", get(graphql_handler).post(graphql_handler))
        .route("/playground", get(graphql_playground))
        .route("/health", get(health))
        .layer(CorsLayer::permissive())
        .layer(Extension(schema.clone()));

    if auth_config.enabled {
        let auth = Arc::new(auth_config);
        let authenticator = auth::authenticator(
            auth.jwt_secret.as_deref(),
            auth.jwks_url.as_deref(),
            auth.issuer.as_deref(),
            auth.audience.as_deref(),
            true,
            &auth.identity_mappings,
            auth.jwks_url
                .as_ref()
                .map(|_| config.circuit_breakers.jwks.clone()),
        );
        app = app.layer(middleware::from_fn(move |req, next: middleware::Next| {
            let authenticator = authenticator.clone();
            async move { auth_middleware(req, next, authenticator).await }
        }));
    }

    // Mount MCP sub-router if enabled
    if let Some(mcp_router) = mcp::build_mcp_router(config.clone(), schema) {
        app = app.merge(mcp_router);
    }

    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!("Morphis server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn graphql_handler(
    Extension(schema): Extension<async_graphql::dynamic::Schema>,
    headers: axum::http::HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let identity = Identity::from_raw(
        headers
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|v| (name.as_str().to_lowercase(), v.to_string()))
            })
            .collect(),
    );
    schema::execute(&schema, req.into_inner(), identity)
        .await
        .into()
}

async fn graphql_playground() -> axum::response::Html<&'static str> {
    axum::response::Html(GRAPHQL_PLAYGROUND_HTML)
}

async fn health() -> &'static str {
    "ok"
}

#[tracing::instrument(skip_all, fields(method = %req.method(), uri = %req.uri(), request_id = %uuid::Uuid::new_v4()))]
async fn auth_middleware(
    mut req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
    authenticator: identity_auth::Authenticator,
) -> Response {
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.trim().to_string());

    if let Some(token) = auth_header {
        match auth::validate_identity(&authenticator, &token).await {
            Ok(identity) => {
                let headers = req.headers_mut();
                for (name, value) in identity.into_headers() {
                    if let Ok(name) = axum::http::header::HeaderName::from_bytes(name.as_bytes())
                        && let Ok(value) = axum::http::HeaderValue::from_str(&value)
                    {
                        headers.insert(name, value);
                    }
                }
                tracing::trace!("Request authenticated successfully");
            }
            Err(e) => {
                tracing::warn!(error = %e, "JWT validation failed");
            }
        }
    } else {
        tracing::debug!("Request without Bearer token");
    }

    next.run(req).await
}

const GRAPHQL_PLAYGROUND_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Morphis GraphQL Playground</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/css/index.css" />
  <link rel="shortcut icon" href="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/favicon.png" />
  <script src="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/js/middleware.js"></script>
</head>
<body>
  <div id="root"></div>
  <script>window.addEventListener('load', function () { GraphQLPlayground.init(document.getElementById('root'), { endpoint: '/graphql' }); });</script>
</body>
</html>"#;
