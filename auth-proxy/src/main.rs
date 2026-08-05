mod config;

use std::sync::Arc;

use identity_auth::AuthPolicy;
use pingora::proxy::{ProxyHttp, Session};
use pingora::server::Server;
use pingora::upstreams::peer::HttpPeer;
use tracing::{debug, info, trace, warn};

use crate::config::ProxyConfig;

struct AuthProxy {
    config: Arc<ProxyConfig>,
    authenticator: identity_auth::Authenticator,
}

#[async_trait::async_trait]
impl ProxyHttp for AuthProxy {
    type CTX = ();

    fn new_ctx(&self) {}

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut (),
    ) -> pingora::Result<Box<HttpPeer>> {
        let addr = self
            .config
            .upstream
            .trim_start_matches("http://")
            .trim_start_matches("https://");
        let addr = match addr.find('/') {
            Some(pos) => &addr[..pos],
            None => addr,
        };
        Ok(Box::new(HttpPeer::new(addr, false, "".to_string())))
    }

    async fn request_filter(&self, session: &mut Session, _ctx: &mut ()) -> pingora::Result<bool> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let span = tracing::info_span!("proxy_request", request_id = %request_id);
        let _guard = span.enter();

        // Skip JWT validation for MCP endpoints — MCP has its own auth
        if session.req_header().uri.path().starts_with("/mcp") {
            return Ok(false);
        }

        let auth_header = session
            .req_header()
            .headers
            .get(http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let token = match auth_header {
            Some(h) if h.starts_with("Bearer ") => h.trim_start_matches("Bearer ").trim(),
            _ => {
                if self.config.require_auth {
                    debug!("Missing or invalid Authorization header");
                    session.respond_error(401).await?;
                    return Ok(true);
                }
                return Ok(false);
            }
        };

        let identity = match self.authenticator.authenticate(token).await {
            Ok(identity) => identity,
            Err(e) => {
                warn!(error = %e, "JWT validation failed");
                session.respond_error(401).await?;
                return Ok(true);
            }
        };

        trace!("Auth proxy: request authenticated successfully");
        for (name, value) in identity.into_headers() {
            let _ = session.req_header_mut().insert_header(name, value);
        }

        Ok(false)
    }
}

fn main() -> anyhow::Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "auth_proxy=info".into());

    if std::env::var("LOG_FORMAT").as_deref() == Ok("json") {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .json()
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }

    let config_path =
        std::env::var("AUTH_PROXY_CONFIG").unwrap_or_else(|_| "config.yaml".to_string());
    let config = Arc::new(ProxyConfig::from_file(&config_path)?);

    info!(
        "Auth proxy starting on {} -> {}",
        config.listen_addr, config.upstream
    );

    let jwks_url = if config.jwt_jwks_url.is_empty() {
        None
    } else {
        Some(config.jwt_jwks_url.clone())
    };
    let jwt_secret = if config.jwt_secret.is_empty() {
        None
    } else {
        Some(config.jwt_secret.clone())
    };
    if jwks_url.is_none() && jwt_secret.is_none() {
        anyhow::bail!("Either jwt_secret or jwt_jwks_url must be configured");
    }
    let issuer = (!config.jwt_issuer.is_empty()).then(|| config.jwt_issuer.clone());

    let authenticator = identity_auth::Authenticator::new(AuthPolicy {
        jwt_secret,
        jwks_url,
        issuer,
        audience: None,
        // The frontend's self-signed HS256 tokens carry no `exp` — expiry is enforced
        // on the JWKS path only (Keycloak tokens always carry a valid `exp`).
        require_exp_secret: false,
        require_exp_jwks: true,
        // Keycloak tokens carry `aud: "account"` (the client id), not the proxy.
        validate_audience: false,
        identity_mappings: config
            .header_mappings
            .iter()
            .map(|m| identity_auth::IdentityMapping {
                claim: m.claim.clone(),
                header: m.header.clone(),
            })
            .collect(),
        jwks_circuit_breaker: config.jwks_circuit_breaker.clone().map(|c| {
            identity_auth::circuit_breaker::CircuitBreakerConfig {
                failure_threshold: c.failure_threshold,
                reset_timeout: std::time::Duration::from_secs(c.reset_timeout_secs),
                half_open_max_requests: c.half_open_max_requests,
            }
        }),
    });

    // Preserve the historical fail-fast behaviour: an unreachable JWKS endpoint aborts
    // startup instead of only surfacing on the first request. A throwaway runtime is used
    // because pingora owns the long-lived one inside `Server::run_forever`.
    {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(authenticator.warm_jwks())?;
    }

    let mut server = Server::new(None)?;
    server.bootstrap();

    let mut proxy_service = pingora::proxy::http_proxy_service(
        &server.configuration,
        AuthProxy {
            config: config.clone(),
            authenticator,
        },
    );
    proxy_service.add_tcp(&config.listen_addr);

    server.add_service(proxy_service);

    info!("Auth proxy ready");
    server.run_forever();
}
