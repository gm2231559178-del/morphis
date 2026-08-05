use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ProxyConfig {
    pub listen_addr: String,
    pub upstream: String,
    #[serde(default)]
    pub jwt_secret: String,
    #[serde(default)]
    pub jwt_jwks_url: String,
    #[serde(default)]
    pub jwt_issuer: String,
    #[serde(default = "default_require_auth")]
    pub require_auth: bool,
    #[serde(default)]
    pub jwks_circuit_breaker: Option<CircuitBreakerConfig>,
    pub header_mappings: Vec<HeaderMapping>,
}

fn default_require_auth() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct CircuitBreakerConfig {
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u64,
    #[serde(default = "default_reset_timeout_secs")]
    pub reset_timeout_secs: u64,
    #[serde(default = "default_half_open_max")]
    pub half_open_max_requests: u64,
}

fn default_failure_threshold() -> u64 {
    5
}

fn default_reset_timeout_secs() -> u64 {
    30
}

fn default_half_open_max() -> u64 {
    3
}

#[derive(Debug, Clone, Deserialize)]
pub struct HeaderMapping {
    pub claim: String,
    pub header: String,
}

impl ProxyConfig {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: ProxyConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}
