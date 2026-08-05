//! Shared test helpers: RSA test keys, a tiny JWKS HTTP server, and JWK builders.
//!
//! `#[cfg(test)]`-only; imported by the test modules in `authenticator.rs` and `jwks.rs`
//! so the RSA key material and HTTP scaffolding live in exactly one place.

use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const TEST_KID: &str = "test-key-1";
pub const TEST_RSA_E: &str = "AQAB";
pub const TEST_RSA_N: &str = "nEoAGI2AatpUzPR_BIUMOmXsHItVwwPzzEZhRW2XszmlivqidBY3XxZA4mRY5kdg_rOC2NC8aWKmqJxHb3emwp1GgjPjvHCtmToqd-LnFjz0Yo5GaBc-MhJHCyBdFXn2IQnC16y2r9pz5ogRm9gRN1tf2DbLljn9x_RjtimkEOitEt-tV__yODxq-i1-yoPUq-f39mRW9AhmoSZozJW_ze1dSBiRMxShKWyVaSR8QmAHIGG_i_riywMxnFVwuCI6Lq2zyRn70vguVx9_A5V9eBIjzIHdGd1BzczqJo0WZ0Vn-Ffp_pX2u2yFaAbOAyPcKDYDiYw5CihfSvhKGFJvLw";
pub const TEST_RSA_PRIVATE_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCcSgAYjYBq2lTM
9H8EhQw6Zewci1XDA/PMRmFFbZezOaWK+qJ0FjdfFkDiZFjmR2D+s4LY0LxpYqao
nEdvd6bCnUaCM+O8cK2ZOip34ucWPPRijkZoFz4yEkcLIF0VefYhCcLXrLav2nPm
iBGb2BE3W1/YNsuWOf3H9GO2KaQQ6K0S361X//I4PGr6LX7Kg9Sr5/f2ZFb0CGah
JmjMlb/N7V1IGJEzFKEpbJVpJHxCYAcgYb+L+uLLAzGcVXC4IjourbPJGfvS+C5X
H38DlX14EiPMgd0Z3UHNzOomjRZnRWf4V+n+lfa7bIVoBs4DI9woNgOJjDkKKF9K
+EoYUm8vAgMBAAECggEAAKwo1/Iz7UHHP6KFsWVJKi8qFu1ajx5DPEvJO10/W9wR
pElzzYAS+OvFl7PK1iLUfgQTug8b4HA2O1+AxzACna/Dj+fdQQBTHuerKxzk1amp
e4sVLnl7IQgHGjsna2I89uNt3TO3DYapHQLU4JDLciuIfAuwUJMrTmL00uHW/OOh
skkYI5p81BA90Qwu/9j7KSwTrF369OA9Vp9eFrAa+TQ6FpZ1SFh71jPT+z//gq8e
VjbGSjA/hLfm7XUiTXb7uQ1WEy18ad2EL8VfSct6aE+5Om7Sq9gkVcRmpAjyjMmU
CzQAfNbYVxx3y+s8nHug2sCB/6WAaxYa0iHPwybLwQKBgQDLMJJz/eEpf9aJL2sv
yIAyduS/pDelkjnc9FoVNTbGvl5TJ4D/jUxb/Q+KqqlvOGYGOFirx2vw78gpUGAj
i+OK2r6SwBnfoQCQvXTE1KvGNX6vYVMW/xHp3/N71bcmhln0xh01t/CrZ07Xm5Ve
IUGj4WX/kHUvN6i/vFQMaxv5UQKBgQDE6Nij1tPzy2X9q+TDLZs6lB2vv2zaXRb3
pOUZcRE1mkU+Ck8QApcVWD4CgMTM36WPISA5BW6HrI0OUyor8ss5SBQ7zXvxNal/
YcihhV27FREmNKsMTrVkuIdbw5es8STLc4yv4ZPiEDR1KeqZ5UwgVBnGGjl52xoI
pauZZeXAfwKBgF94VgfMDSSbnWjd7+YGtj1/4aEt/rt8BlYMNdtrIm6ledpmYFUy
xeMe91N3Np88h6t6hCdKTyxo7cqDqnhpPSO7/fkj68RIeOSJMDlfl8pMzlaHSywt
8vPJtzTDSQf/7np1L7pSz/EpXEEwKDGPPLFMsckvze++njpgubkQBpfRAoGAEHJq
dfThq0FX+YI8D1ll19S7TgytKOgRnQm24RMintmN4wq1Y97zg6LlOwxKY9piV7wq
ltivTMHK3mFv6k/TTauJlR0qtxEGYU9nlKYxGAlAb3KCvvpsCEepdq61oopZymyS
WbZ7xawY1Zh0sfoHC8Q6iuNx3Y3BdOtxk9SBBj0CgYEAmMtUzN8Oz07KHaQL9A44
6eqk+J7YBAzA5Q89aAECNMi61VG9wDdcwD8eIqpbTyXJFLuiFnGTl2F947oyIU8W
x5DLdAwGT5DZzacCjprQ3LcAOcJRKEVVLpFZWYJw2Zf6ScVUVM6T3e9B/H5cqbbs
nk5Oc8VQZdFJV3nIb0Zbms0=
-----END PRIVATE KEY-----"#;

/// Minimal HTTP server that answers every request with the given body.
pub struct JwksTestServer {
    url: String,
}

impl JwksTestServer {
    pub fn url(&self) -> String {
        self.url.clone()
    }
}

pub async fn spawn_jwks_server(body: String) -> JwksTestServer {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let body = body.clone();
            tokio::spawn(async move {
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

pub fn test_jwk() -> jsonwebtoken::jwk::Jwk {
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

pub fn kid_only_jwk(kid: &str) -> jsonwebtoken::jwk::Jwk {
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

/// JWKS document containing the test RSA key under `TEST_KID`.
pub fn test_jwks_json() -> String {
    serde_json::json!({ "keys": [test_jwk()] }).to_string()
}
