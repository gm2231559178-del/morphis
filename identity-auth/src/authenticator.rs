use std::collections::HashMap;

use jsonwebtoken::{AlgorithmFamily, Algorithm as JwtAlgorithm, DecodingKey, Validation, decode};

use crate::Identity;
use crate::jwks::JwksProvider;
use crate::policy::AuthPolicy;

/// Why authentication failed. The message is safe to log (no token material).
#[derive(Debug)]
pub struct AuthError(pub String);

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for AuthError {}

/// Validates JWTs against an [`AuthPolicy`] and maps claims to an [`Identity`].
///
/// One instance per entry point. JWKS keys are fetched lazily and cached with a TTL, so no
/// network call happens on the first request when only the shared secret is configured.
#[derive(Clone)]
pub struct Authenticator {
    policy: AuthPolicy,
    jwks: Option<JwksProvider>,
}

impl Authenticator {
    pub fn new(policy: AuthPolicy) -> Self {
        let jwks = policy
            .jwks_url
            .clone()
            .map(|url| JwksProvider::new(url, policy.jwks_circuit_breaker.clone()));
        Self { policy, jwks }
    }

    /// Validate the token and produce the claim-derived identity.
    ///
    /// On failure the returned error message describes the rejection without echoing
    /// token or secret material.
    pub async fn authenticate(&self, token: &str) -> Result<Identity, AuthError> {
        if let Some(secret) = &self.policy.jwt_secret {
            let key = DecodingKey::from_secret(secret.as_bytes());
            let validation = self.validation(AlgorithmFamily::Hmac);
            if let Ok(data) = decode::<serde_json::Value>(token, &key, &validation) {
                return Ok(self.identity_from_claims(&data.claims));
            }
        }

        if let Some(jwks) = &self.jwks {
            let keys = jwks
                .resolve_keys(token)
                .await
                .map_err(|e| AuthError(format!("JWKS validation failed: {e}")))?;
            for key in &keys {
                let validation = self.validation(key.family());
                if let Ok(data) = decode::<serde_json::Value>(token, key, &validation) {
                    return Ok(self.identity_from_claims(&data.claims));
                }
            }
        }

        Err(AuthError("JWT validation failed: token rejected by all key sources".into()))
    }

    /// Eagerly fetch and cache the JWKS key set, failing on error. Entry points that want
    /// to fail fast at startup (rather than on the first request) call this after [`new`].
    pub async fn warm_jwks(&self) -> Result<(), AuthError> {
        match &self.jwks {
            Some(jwks) => jwks.warm().await.map_err(AuthError),
            None => Ok(()),
        }
    }

    /// The `Validation` for a given key family, derived from the policy.
    ///
    /// The algorithm allow-list must contain only algorithms of `family`: jsonwebtoken
    /// rejects any token whose key family does not cover every entry in the allow-list, so
    /// a mixed list would fail even for valid tokens.
    fn validation(&self, family: AlgorithmFamily) -> Validation {
        let mut validation = Validation::new(JwtAlgorithm::RS256);
        validation.algorithms = family.algorithms().to_vec();
        validation.validate_aud = self.policy.validate_audience;
        validation.validate_exp = true;
        validation.required_spec_claims = std::collections::HashSet::new();
        if let Some(audience) = &self.policy.audience {
            validation.set_audience(std::slice::from_ref(audience));
        }
        if let Some(issuer) = &self.policy.issuer {
            validation.set_issuer(std::slice::from_ref(issuer));
        }
        // Expiry enforcement: JWKS tokens must always be unexpired; HS* tokens follow the
        // policy flag (auth-proxy's self-signed frontend tokens carry no `exp`). When
        // enforcement is off, `exp` is neither required nor checked.
        let enforce_exp = match family {
            AlgorithmFamily::Hmac => self.policy.require_exp_secret,
            _ => self.policy.require_exp_jwks,
        };
        if !enforce_exp {
            validation.validate_exp = false;
        } else {
            validation.required_spec_claims.insert("exp".to_string());
        }
        validation
    }

    fn identity_from_claims(&self, claims: &serde_json::Value) -> Identity {
        let mut headers = HashMap::new();
        for mapping in &self.policy.identity_mappings {
            if let Some(value) = resolve_claim(claims, &mapping.claim)
                && let Some(value) = stringify_claim(value)
            {
                headers.insert(mapping.header.clone(), value);
            }
        }
        Identity::from_raw(headers)
    }
}

/// Resolve a possibly-nested claim path (`"user.tenant_id"`) to a JSON value.
fn resolve_claim<'a>(claims: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut current = claims;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

/// Convert a claim value to a header string. Scalars become their JSON text; arrays and
/// objects are skipped (a header can only carry one string).
fn stringify_claim(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        serde_json::Value::Null | serde_json::Value::Array(_) | serde_json::Value::Object(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{spawn_jwks_server, JwksTestServer, TEST_KID, TEST_RSA_PRIVATE_KEY};
    use std::time::{SystemTime, UNIX_EPOCH};

    const TEST_SECRET: &str = "test-secret-that-is-long-enough";

    fn secret_policy(require_exp: bool) -> AuthPolicy {
        AuthPolicy {
            jwt_secret: Some(TEST_SECRET.to_string()),
            jwks_url: None,
            issuer: None,
            audience: None,
            require_exp_secret: require_exp,
            require_exp_jwks: true,
            validate_audience: false,
            identity_mappings: vec![
                crate::IdentityMapping {
                    claim: "sub".to_string(),
                    header: "x-user-id".to_string(),
                },
                crate::IdentityMapping {
                    claim: "tenant_id".to_string(),
                    header: "x-tenant-id".to_string(),
                },
                crate::IdentityMapping {
                    claim: "role".to_string(),
                    header: "x-role".to_string(),
                },
            ],
            jwks_circuit_breaker: None,
        }
    }

    fn jwks_policy(require_exp: bool) -> AuthPolicy {
        AuthPolicy {
            jwt_secret: None,
            jwks_url: Some("http://127.0.0.1:1/jwks".to_string()),
            issuer: None,
            audience: None,
            require_exp_secret: true,
            require_exp_jwks: require_exp,
            validate_audience: false,
            identity_mappings: secret_policy(false).identity_mappings,
            jwks_circuit_breaker: None,
        }
    }

    fn now() -> i64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
    }

    fn sign_hs256(claims: serde_json::Value) -> String {
        let header = jsonwebtoken::Header::new(JwtAlgorithm::HS256);
        jsonwebtoken::encode(
            &header,
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(TEST_SECRET.as_bytes()),
        )
        .unwrap()
    }

    fn sign_rs256(claims: serde_json::Value) -> String {
        let mut header = jsonwebtoken::Header::new(JwtAlgorithm::RS256);
        header.kid = Some(TEST_KID.to_string());
        jsonwebtoken::encode(
            &header,
            &claims,
            &jsonwebtoken::EncodingKey::from_rsa_pem(TEST_RSA_PRIVATE_KEY.as_bytes()).unwrap(),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn accepts_valid_hs256_token_with_claims() {
        let token = sign_hs256(serde_json::json!({
            "sub": "user-1",
            "tenant_id": "tenant-a",
            "role": "admin",
            "exp": now() + 3600,
        }));
        let authenticator = Authenticator::new(secret_policy(true));
        let identity = authenticator.authenticate(&token).await.unwrap();
        assert_eq!(identity.header_value("x-user-id"), Some("user-1"));
        assert_eq!(identity.header_value("x-tenant-id"), Some("tenant-a"));
        assert_eq!(identity.header_value("x-role"), Some("admin"));
    }

    #[tokio::test]
    async fn skips_unknown_claim_paths() {
        let token = sign_hs256(serde_json::json!({
            "sub": "user-1",
            "tenant_id": "tenant-a",
            "role": "admin",
            "exp": now() + 3600,
        }));
        let mut policy = secret_policy(true);
        policy.identity_mappings.push(crate::IdentityMapping {
            claim: "user.tenant_id".to_string(),
            header: "x-nested-tenant".to_string(),
        });
        policy.identity_mappings.push(crate::IdentityMapping {
            claim: "missing_claim".to_string(),
            header: "x-missing".to_string(),
        });
        let authenticator = Authenticator::new(policy);
        let identity = authenticator.authenticate(&token).await.unwrap();
        assert_eq!(identity.header_value("x-nested-tenant"), None);
        assert_eq!(identity.header_value("x-missing"), None);
        assert_eq!(identity.header_value("x-tenant-id"), Some("tenant-a"));
    }

    #[tokio::test]
    async fn resolves_nested_claim_paths() {
        let token = sign_hs256(serde_json::json!({
            "sub": "user-1",
            "user": { "tenant_id": "tenant-nested" },
            "exp": now() + 3600,
        }));
        let mut policy = secret_policy(true);
        policy.identity_mappings = vec![crate::IdentityMapping {
            claim: "user.tenant_id".to_string(),
            header: "x-tenant-id".to_string(),
        }];
        let identity = Authenticator::new(policy).authenticate(&token).await.unwrap();
        assert_eq!(identity.header_value("x-tenant-id"), Some("tenant-nested"));
    }

    #[tokio::test]
    async fn expired_token_rejected_when_exp_enforced() {
        // Expired well beyond the 60s clock-skew leeway.
        let token = sign_hs256(serde_json::json!({ "sub": "u", "exp": now() - 3600 }));
        let result = Authenticator::new(secret_policy(true)).authenticate(&token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn expired_token_accepted_when_exp_not_enforced() {
        let token = sign_hs256(serde_json::json!({ "sub": "u", "exp": now() - 60 }));
        let identity = Authenticator::new(secret_policy(false)).authenticate(&token).await;
        assert!(identity.is_ok());
        assert_eq!(identity.unwrap().header_value("x-user-id"), Some("u"));
    }

    #[tokio::test]
    async fn token_without_exp_accepted_when_not_enforced() {
        let token = sign_hs256(serde_json::json!({ "sub": "u", "tenant_id": "t" }));
        let authenticator = Authenticator::new(secret_policy(false));
        let identity = authenticator.authenticate(&token).await;
        assert!(identity.is_ok());
    }

    #[tokio::test]
    async fn token_without_exp_rejected_when_enforced() {
        let token = sign_hs256(serde_json::json!({ "sub": "u", "tenant_id": "t" }));
        let result = Authenticator::new(secret_policy(true)).authenticate(&token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn wrong_secret_rejected() {
        let token = sign_hs256(serde_json::json!({ "sub": "u", "exp": now() + 3600 }));
        let mut policy = secret_policy(true);
        policy.jwt_secret = Some("another-secret-value".to_string());
        let result = Authenticator::new(policy).authenticate(&token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn rejects_malformed_token() {
        let result = Authenticator::new(secret_policy(true))
            .authenticate("not.a.jwt")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn accepts_valid_rs256_token_via_jwks() {
        let token = sign_rs256(serde_json::json!({
            "sub": "user-rsa",
            "tenant_id": "tenant-b",
            "role": "editor",
            "exp": now() + 3600,
        }));

        // JWKS URL points at nothing, but the JWKS key is injected into the shared test
        // keyring via the circuit-breaker-free JwksProvider cache. Use a real HTTP server.
        let jwks_json = serde_json::json!({
            "keys": [{
                "kty": "RSA",
                "use": "sig",
                "alg": "RS256",
                "kid": TEST_KID,
                "n": "nEoAGI2AatpUzPR_BIUMOmXsHItVwwPzzEZhRW2XszmlivqidBY3XxZA4mRY5kdg_rOC2NC8aWKmqJxHb3emwp1GgjPjvHCtmToqd-LnFjz0Yo5GaBc-MhJHCyBdFXn2IQnC16y2r9pz5ogRm9gRN1tf2DbLljn9x_RjtimkEOitEt-tV__yODxq-i1-yoPUq-f39mRW9AhmoSZozJW_ze1dSBiRMxShKWyVaSR8QmAHIGG_i_riywMxnFVwuCI6Lq2zyRn70vguVx9_A5V9eBIjzIHdGd1BzczqJo0WZ0Vn-Ffp_pX2u2yFaAbOAyPcKDYDiYw5CihfSvhKGFJvLw",
                "e": "AQAB",
            }]
        })
        .to_string();

        let server = spawn_jwks_server(jwks_json).await;
        let policy = AuthPolicy {
            jwks_url: Some(server.url()),
            ..jwks_policy(true)
        };
        let identity = Authenticator::new(policy).authenticate(&token).await.unwrap();
        assert_eq!(identity.header_value("x-user-id"), Some("user-rsa"));
        assert_eq!(identity.header_value("x-tenant-id"), Some("tenant-b"));
        assert_eq!(identity.header_value("x-role"), Some("editor"));
    }

    #[tokio::test]
    async fn expired_rs256_token_rejected_when_jwks_exp_enforced() {
        let token = sign_rs256(serde_json::json!({ "sub": "u", "exp": now() - 3600 }));
        let server = spawn_jwks_server(crate::test_support::test_jwks_json()).await;
        let policy = AuthPolicy {
            jwks_url: Some(server.url()),
            ..jwks_policy(true)
        };
        let result = Authenticator::new(policy).authenticate(&token).await;
        assert!(result.is_err());
    }
}
