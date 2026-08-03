//! Adapter between the app config and the shared `identity-auth` crate.
//!
//! Both the HTTP server and the MCP server express their auth configs as
//! [`identity_auth::AuthPolicy`] here, so all JWT validation and claim→header mapping
//! happens in the shared, tested module.

use identity_auth::{AuthPolicy, Authenticator, Identity, IdentityMapping};

use crate::config::{CircuitBreakerInstanceConfig, MCPIdentityMapping};

/// Build the shared [`Authenticator`] for an entry point whose auth is enabled.
///
/// `require_exp_secret` mirrors the entry point's expiry enforcement for symmetric
/// tokens: morphis requires `exp` on its HS* tokens, auth-proxy (frontend hop) does not.
pub fn authenticator(
    jwt_secret: Option<&str>,
    jwks_url: Option<&str>,
    issuer: Option<&str>,
    audience: Option<&str>,
    require_exp_secret: bool,
    identity_mappings: &[MCPIdentityMapping],
    jwks_breaker: Option<CircuitBreakerInstanceConfig>,
) -> Authenticator {
    Authenticator::new(AuthPolicy {
        jwt_secret: jwt_secret.map(str::to_string),
        jwks_url: jwks_url.map(str::to_string),
        issuer: issuer.map(str::to_string),
        audience: audience.map(str::to_string),
        require_exp_secret,
        require_exp_jwks: true,
        validate_audience: false,
        identity_mappings: identity_mappings
            .iter()
            .map(|m| IdentityMapping {
                claim: m.claim.clone(),
                header: m.header.clone(),
            })
            .collect(),
        jwks_circuit_breaker: jwks_breaker.map(|c| {
            identity_auth::circuit_breaker::CircuitBreakerConfig {
                failure_threshold: c.failure_threshold,
                reset_timeout: std::time::Duration::from_secs(c.reset_timeout_secs),
                half_open_max_requests: c.half_open_max_requests,
            }
        }),
    })
}

/// Validate a token and return the claim-derived identity, or a loggable error string.
pub async fn validate_identity(
    authenticator: &Authenticator,
    token: &str,
) -> Result<Identity, String> {
    authenticator
        .authenticate(token)
        .await
        .map_err(|e| e.0)
}
