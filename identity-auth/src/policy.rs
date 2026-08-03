use crate::circuit_breaker::CircuitBreakerConfig;
use crate::identity::IdentityMapping;

/// Validation policy for one entry point.
///
/// The security-relevant divergences between the entry points (expiry enforcement, audience
/// checking) are expressed here as policy rather than as separate JWT implementations.
/// This is the only place where "is expiry enforced" is answered.
#[derive(Debug, Clone)]
pub struct AuthPolicy {
    /// Symmetric key used to validate HS256/384/512 tokens.
    pub jwt_secret: Option<String>,
    /// JWKS endpoint used to validate RS/ES/EdDSA tokens.
    pub jwks_url: Option<String>,
    /// Optional `iss` claim to require.
    pub issuer: Option<String>,
    /// Optional `aud` claim to require (only checked when `validate_audience` is true).
    pub audience: Option<String>,
    /// Enforce `exp` for HS* (symmetric) tokens. auth-proxy disables this for its
    /// self-signed frontend tokens, which carry no `exp`.
    pub require_exp_secret: bool,
    /// Enforce `exp` for JWKS (asymmetric) tokens.
    pub require_exp_jwks: bool,
    /// Check the `aud` claim (auth-proxy disables this for Keycloak tokens whose audience
    /// is the client id, not the proxy).
    pub validate_audience: bool,
    /// Claim → header mappings applied when building the resulting identity.
    pub identity_mappings: Vec<IdentityMapping>,
    /// Circuit breaker config for JWKS fetches. `None` uses a default.
    pub jwks_circuit_breaker: Option<CircuitBreakerConfig>,
}

impl AuthPolicy {
    /// Whether any key source is configured. Entry points use this to fail fast at startup.
    pub fn has_key_source(&self) -> bool {
        self.jwt_secret.is_some() || self.jwks_url.is_some()
    }
}
