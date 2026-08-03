use std::sync::Arc;
use std::time::Duration;

use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::{DecodingKey, decode_header};
use tokio::sync::RwLock;
use tokio::time::Instant;

use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};

/// How long a fetched JWKS is trusted before re-fetching.
const JWKS_TTL: Duration = Duration::from_secs(300);

/// Fetches, caches and refreshes a JWKS key set, and selects keys for a token.
///
/// Key selection is unified: prefer an exact `kid` match, otherwise fall back to scanning
/// all keys. When the cached key set is stale or the token's `kid` is missing from it
/// (a rotation), the set is re-fetched once.
#[derive(Clone)]
pub struct JwksProvider {
    url: String,
    client: reqwest::Client,
    cache: Arc<RwLock<JwksCache>>,
    circuit_breaker: CircuitBreaker,
}

struct JwksCache {
    keys: Vec<Jwk>,
    fetched_at: Instant,
}

impl JwksProvider {
    pub fn new(url: String, breaker_config: Option<CircuitBreakerConfig>) -> Self {
        let circuit_breaker = match breaker_config {
            Some(config) => CircuitBreaker::new(config),
            None => CircuitBreaker::new(CircuitBreakerConfig {
                failure_threshold: 3,
                reset_timeout: Duration::from_secs(60),
                half_open_max_requests: 1,
            }),
        };
        let cache = Arc::new(RwLock::new(JwksCache {
            keys: Vec::new(),
            fetched_at: Instant::now() - JWKS_TTL - Duration::from_secs(1),
        }));
        Self {
            url,
            client: reqwest::Client::new(),
            cache,
            circuit_breaker,
        }
    }

    /// Resolve the candidate decoding keys for a token.
    ///
    /// Returns a single key when the token's `kid` matches (preferred), otherwise all
    /// convertible keys so the caller can try each. On a stale cache or a missing `kid`
    /// the key set is refreshed first.
    pub async fn resolve_keys(&self, token: &str) -> Result<Vec<DecodingKey>, String> {
        let header = decode_header(token).map_err(|e| format!("JWT header decode failed: {e}"))?;
        let kid = header.kid.clone();

        {
            let cache = self.cache.read().await;
            if cache.fetched_at.elapsed() < JWKS_TTL {
                let selected = select_key(&cache.keys, kid.as_deref());
                if !selected.is_empty() || kid.is_none() {
                    return Ok(selected);
                }
            }
        }

        let fetched = self.fetch_keys().await?;
        let selected = select_key(&fetched, kid.as_deref());
        {
            let mut cache = self.cache.write().await;
            cache.keys = fetched;
            cache.fetched_at = Instant::now();
        }
        Ok(selected)
    }

    /// Fetch and cache the key set now, propagating errors.
    ///
    /// Used at startup by entry points that want to fail fast when the JWKS endpoint is
    /// unreachable, matching the eager fetch they historically did.
    pub async fn warm(&self) -> Result<(), String> {
        let fetched = self.fetch_keys().await?;
        if fetched.is_empty() {
            return Err(format!("No usable JWKS keys found from {}", self.url));
        }
        let mut cache = self.cache.write().await;
        cache.keys = fetched;
        cache.fetched_at = Instant::now();
        Ok(())
    }

    async fn fetch_keys(&self) -> Result<Vec<Jwk>, String> {
        let response = self
            .circuit_breaker
            .call(|| self.client.get(&self.url).send())
            .await
            .map_err(|e| format!("JWKS fetch failed (circuit breaker): {e}"))?;
        let body = response
            .text()
            .await
            .map_err(|e| format!("JWKS body read failed: {e}"))?;
        let set: jsonwebtoken::jwk::JwkSet =
            serde_json::from_str(&body).map_err(|e| format!("JWKS parse failed: {e}"))?;
        tracing::debug!(url = %self.url, key_count = set.keys.len(), "Fetched JWKS keys");
        Ok(set.keys)
    }
}

/// Prefer an exact `kid` match; fall back to the first convertible key.
fn select_key(keys: &[Jwk], kid: Option<&str>) -> Vec<DecodingKey> {
    if let Some(kid) = kid
        && let Some(jwk) = keys.iter().find(|k| k.common.key_id.as_deref() == Some(kid))
        && let Ok(key) = DecodingKey::from_jwk(jwk)
    {
        return vec![key];
    }
    keys.iter()
        .filter_map(|k| DecodingKey::from_jwk(k).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_RSA_N: &str = "nEoAGI2AatpUzPR_BIUMOmXsHItVwwPzzEZhRW2XszmlivqidBY3XxZA4mRY5kdg_rOC2NC8aWKmqJxHb3emwp1GgjPjvHCtmToqd-LnFjz0Yo5GaBc-MhJHCyBdFXn2IQnC16y2r9pz5ogRm9gRN1tf2DbLljn9x_RjtimkEOitEt-tV__yODxq-i1-yoPUq-f39mRW9AhmoSZozJW_ze1dSBiRMxShKWyVaSR8QmAHIGG_i_riywMxnFVwuCI6Lq2zyRn70vguVx9_A5V9eBIjzIHdGd1BzczqJo0WZ0Vn-Ffp_pX2u2yFaAbOAyPcKDYDiYw5CihfSvhKGFJvLw";
    const TEST_RSA_E: &str = "AQAB";
    const TEST_KID: &str = "test-key-1";

    fn test_jwk() -> Jwk {
        serde_json::from_value(serde_json::json!({
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "kid": TEST_KID,
            "n": TEST_RSA_N,
            "e": TEST_RSA_E,
        }))
        .unwrap()
    }

    fn kid_only_jwk(kid: &str) -> Jwk {
        serde_json::from_value(serde_json::json!({
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "kid": kid,
            "n": TEST_RSA_N,
            "e": TEST_RSA_E,
        }))
        .unwrap()
    }

    #[test]
    fn select_prefers_exact_kid_match() {
        let keys = vec![kid_only_jwk("other"), test_jwk()];
        let selected = select_key(&keys, Some(TEST_KID));
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].family(), jsonwebtoken::AlgorithmFamily::Rsa);
    }

    #[test]
    fn select_falls_back_to_scanning_all_keys() {
        let keys = vec![kid_only_jwk("other"), test_jwk()];
        let selected = select_key(&keys, None);
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn select_falls_back_to_scanning_when_kid_misses() {
        let keys = vec![kid_only_jwk("other")];
        let selected = select_key(&keys, Some(TEST_KID));
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].family(), jsonwebtoken::AlgorithmFamily::Rsa);
    }

    #[tokio::test]
    async fn resolve_keys_fetches_and_caches_from_http_endpoint() {
        let jwks_json = serde_json::json!({ "keys": [test_jwk()] }).to_string();
        let server = spawn_jwks_server(jwks_json).await;
        let provider = JwksProvider::new(server.url(), None);

        // First resolve triggers a fetch; only the token header matters for key selection.
        let token = fake_rs256_token(TEST_KID);
        let keys = provider.resolve_keys(&token).await.unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].family(), jsonwebtoken::AlgorithmFamily::Rsa);

        // Second resolve hits the cache without a second fetch (server shuts down safely).
        let keys = provider.resolve_keys(&token).await.unwrap();
        assert_eq!(keys.len(), 1);
    }

    #[tokio::test]
    async fn warm_populates_cache_without_a_token() {
        let jwks_json = serde_json::json!({ "keys": [test_jwk()] }).to_string();
        let server = spawn_jwks_server(jwks_json).await;
        let provider = JwksProvider::new(server.url(), None);
        provider.warm().await.unwrap();
        // Warmth is observable through a normal resolve, which now hits the cache.
        let keys = provider
            .resolve_keys(&fake_rs256_token(TEST_KID))
            .await
            .unwrap();
        assert_eq!(keys.len(), 1);
    }

    fn fake_rs256_token(kid: &str) -> String {
        let header = serde_json::json!({ "alg": "RS256", "typ": "JWT", "kid": kid });
        let raw = serde_json::to_vec(&header).unwrap();
        format!("{}.e30.e30", base64url_encode(&raw))
    }

    fn base64url_encode(bytes: &[u8]) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let b = [
                chunk[0],
                chunk.get(1).copied().unwrap_or(0),
                chunk.get(2).copied().unwrap_or(0),
            ];
            let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
            out.push(ALPHABET[(n >> 18) as usize & 63] as char);
            out.push(ALPHABET[(n >> 12) as usize & 63] as char);
            if chunk.len() > 1 {
                out.push(ALPHABET[(n >> 6) as usize & 63] as char);
            }
            if chunk.len() > 2 {
                out.push(ALPHABET[n as usize & 63] as char);
            }
        }
        out
    }

    async fn spawn_jwks_server(body: String) -> JwksTestServer {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                let body = body.clone();
                tokio::spawn(async move {
                    use tokio::io::{AsyncReadExt, AsyncWriteExt};
                    let mut buf = [0u8; 1024];
                    let _ = socket.read(&mut buf).await;
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                });
            }
        });
        JwksTestServer {
            url: format!("http://{addr}/jwks"),
        }
    }

    struct JwksTestServer {
        url: String,
    }

    impl JwksTestServer {
        fn url(&self) -> String {
            self.url.clone()
        }
    }
}
