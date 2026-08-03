//! Single identity auth module shared by the morphis and auth-proxy binaries.
//!
//! All three entry points (morphis HTTP, morphis MCP, auth-proxy) validate JWTs and map
//! claims to request headers through this one module, so the security-critical logic is
//! audited and tested in exactly one place. Entry-point divergences (expiry enforcement,
//! audience checking) are expressed as [`AuthPolicy`], not as forked implementations.

pub mod authenticator;
pub mod circuit_breaker;
pub mod jwks;
pub mod policy;

mod identity;

pub use authenticator::{AuthError, Authenticator};
pub use identity::{Identity, IdentityMapping};
pub use policy::AuthPolicy;
