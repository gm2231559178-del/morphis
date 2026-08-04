# Spec: Single identity auth module

Status: ready-for-agent
Priority: 2
Feature: single-identity-auth-module

## Problem Statement

JWT validation is implemented three times, in two crates, with semantics that quietly diverge:

- the morphis HTTP auth middleware (`validate_jwt` + JWKS fetch),
- the morphis MCP auth middleware (hand-rolled JWKS parsing with kid-based key selection),
- the auth-proxy binary (HS256 expiry disabled, audience validation off, JWKS fetched once at startup with no refresh).

The claim → header mapping is also triplicated. For an operator, this means the same token can be accepted by one entry point and rejected (or treated differently) by another, and a security-relevant decision like "is expiry enforced on HS256" has three answers. None of the three implementations has a unit test.

## Solution

One Identity auth module behind a single interface — `authenticate(token, policy) -> Result<Identity>` — with two adapters: an HS256 adapter and a JWKS adapter. Divergent behaviours (expiry enforcement, audience checking, key selection, JWKS refresh) become policy configuration rather than forked implementations. All three entry points (HTTP, MCP, auth-proxy) call the same module.

## User Stories

1. As a security reviewer, I want exactly one JWT validation implementation, so that I can audit the security-critical logic once instead of three times.
2. As a developer, I want HS256 expiry enforced consistently across entry points, so that an expired token cannot be accepted somewhere the other entry points reject.
3. As a developer, I want key selection semantics identical everywhere, so that a token signed by a key not present in JWKS is rejected consistently.
4. As an operator of auth-proxy, I want its looser or stricter policy to be a configuration choice of the shared module, so that policy divergence stops being accidental.
5. As a developer, I want JWKS fetching and refresh handled in one place, so that a JWKS outage or rotation is handled consistently.
6. As a developer, I want the claim → header mapping written once, so that adding a claim header is a single edit instead of three.
7. As a tester, I want validation and identity mapping unit-testable through the module interface, so that expiry, kid matching, and claim mapping are verified without a network.
8. As a user of auth-proxy, I want it to keep working after the change, so that the migration is behaviour-preserving where policy is unchanged.

## Implementation Decisions

- New module: `Identity` auth with interface `authenticate(token, policy) -> Result<Identity>`; `policy` carries issuer, audience, expiry handling, allowed algorithms, and identity mappings.
- Two adapters behind an internal seam: HS256 secret and JWKS. Key selection is unified (prefer `kid`, fall back to scanning keys) and documented.
- The module ships in a small library crate that both the morphis binary and the auth-proxy binary depend on. The current workspace has no shared library crate, so this introduces the first one.
- Claim → header mapping is folded into the module as part of `Identity` construction; the three copies (`AuthConfig.identity_mappings`, `MCPAuthConfig.identity_mappings`, auth-proxy `header_mappings`) are replaced by one config type.
- JWKS refresh moves into the JWKS adapter with the circuit breaker already used by morphis.
- The two dormant copies inside morphis (HTTP + MCP) are deleted; auth-proxy's inline validation is replaced by the module. Where auth-proxy intentionally diverges (expiry off, audience off), that is expressed as policy, not as new code.

## Testing Decisions

- A good test asserts external behaviour: a token accepted/rejected by the interface, and the resulting `Identity`, for a given policy.
- Modules tested: the auth module (new), through `authenticate`. Adapters tested against a stub JWKS server (local HTTP endpoint serving a key set) and against a secret.
- Prior art: `circuit_breaker.rs` unit tests; `tests/auth_proxy.hurl` covers the auth-proxy entry point end-to-end.
- New tests: HS256 expiry on/off per policy; JWKS key matching with and without `kid`; claim → header mapping; missing-key rejection. Existing `auth_proxy.hurl` and `mcp.hurl` (401 paths) stay green as regression.
- The auth module's tests must not require a live Keycloak — a stub JWKS server is sufficient.

## Out of Scope

- The MCP in-process execution seam (separate spec: `mcp-in-process-seam`) — it consumes the `Identity` this module produces but is otherwise independent.
- The frontend's self-signed JWT (`frontend/app/api/graphql/route.ts`) — noted as a fourth producer of the claim contract, but outside this change.
- Changing which claims Keycloak emits (that lives in `scripts/keycloak-setup.py`).

## Further Notes

The three copies differ in real ways today (exp on/off, kid vs all-keys, aud on/off, fail-closed vs degrade-to-anonymous). Normalizing on one module means picking semantics for each of those axes and expressing the exceptions as policy. The degrade-to-anonymous behaviour of the morphis HTTP middleware is the one deliberate divergence worth keeping and encoding explicitly.
