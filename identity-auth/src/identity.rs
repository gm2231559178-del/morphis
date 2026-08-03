use std::collections::HashMap;

use serde::Deserialize;

/// One-to-one claim → request-header mapping applied after a successful token validation.
#[derive(Debug, Clone, Deserialize)]
pub struct IdentityMapping {
    pub claim: String,
    pub header: String,
}

/// The authenticated request identity: a set of claim-derived headers that row filters and
/// RBAC read (e.g. `x-tenant-id`). Produced by [`Authenticator::authenticate`](crate::Authenticator).
#[derive(Debug, Clone, Default)]
pub struct Identity {
    headers: HashMap<String, String>,
}

impl Identity {
    /// Build an identity from raw claim-derived headers. Header names are normalized to
    /// lowercase so lookups are case-insensitive.
    pub fn from_raw(headers: HashMap<String, String>) -> Self {
        Self {
            headers: headers
                .into_iter()
                .map(|(name, value)| (name.to_lowercase(), value))
                .collect(),
        }
    }

    pub fn header_value(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(String::as_str)
    }

    /// Consume the identity into its claim-derived headers.
    pub fn into_headers(self) -> HashMap<String, String> {
        self.headers
    }

    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_lookup_is_case_insensitive() {
        let mut headers = HashMap::new();
        headers.insert("X-Tenant-Id".to_string(), "tenant-a".to_string());
        let identity = Identity::from_raw(headers);
        assert_eq!(identity.header_value("x-tenant-id"), Some("tenant-a"));
        assert_eq!(identity.header_value("X-TENANT-ID"), Some("tenant-a"));
        assert_eq!(identity.header_value("other"), None);
    }

    #[test]
    fn empty_identity_is_anonymous() {
        let identity = Identity::default();
        assert!(identity.is_empty());
        assert_eq!(identity.header_value("x-tenant-id"), None);
    }
}
