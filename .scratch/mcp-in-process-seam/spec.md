# Spec: MCP in-process query-execution seam

Status: ready-for-agent
Priority: 1
Feature: mcp-in-process-seam

## Problem Statement

The MCP server lives in the same process as the GraphQL server, but its `graphql` tool re-enters the application through the network: it POSTs a query to `http://localhost:{port}/graphql`. This has three consequences for a user of the MCP server:

1. **Row filters are bypassed.** The MCP auth middleware builds an `Identity` from the caller's JWT and scopes it into a task-local, but nothing ever reads it. The tool's HTTP call carries no headers, so the query executes with zero identity — a caller authenticated as tenant A can read tenant B's rows through the MCP server.
2. **MCP depends on the HTTP stack.** The tool fails if the listener is down or the port is wrong, and it breaks when app-level auth is enabled (its internal call sends no Bearer token).
3. **The MCP server rebuilds an `AppContext` it never uses.** A second construction site for the same context that only adds dead weight and a second place for wiring to drift.

## Solution

Extract one in-process query-execution interface — `execute(query, variables, identity)` — used by both the HTTP GraphQL handler and the MCP `graphql` / `graphql_schema` tools. The `Identity` travels explicitly from the MCP auth middleware into the execution, so row filters and RBAC apply to MCP calls exactly as they do to HTTP calls.

## User Stories

1. As a tenant-isolated database operator, I want MCP tool calls to honor row filters, so that a caller cannot read another tenant's rows through the MCP server.
2. As a tenant-isolated database operator, I want MCP tool calls to honor column-level RBAC, so that restricted tables are as restricted via MCP as via GraphQL.
3. As a user of the MCP `graphql` tool, I want it to return the same rows as the `/graphql` endpoint for the same identity, so that results are consistent between the two entry points.
4. As an operator, I want MCP and app-level auth to be enableable together, so that the MCP internal call no longer depends on app auth being disabled.
5. As a developer, I want the MCP `graphql` tool to stop using the network for an in-process call, so that it works even if the listener or port is misconfigured.
6. As a developer, I want exactly one query-execution interface, so that HTTP and MCP cross the same seam and behaviour cannot diverge.
7. As a developer, I want the dead MCP `AppContext` rebuild removed, so that there is exactly one construction site for the app context.
8. As a developer, I want the dead `MCP_IDENTITY` task-local scoping removed, so that identity flows as an explicit parameter instead of a value that is scoped but never read.
9. As a tester, I want MCP tool behaviour testable without the HTTP server and without an auth configuration, so that identity → row-filter behaviour is verified in a unit test.
10. As a user of the MCP `graphql_schema` tool, I want introspection to read the in-process schema, so that it does not depend on an HTTP round-trip and its cache stays valid.

## Implementation Decisions

- Introduce a query-execution module with a single interface `execute(query, variables, identity) -> Value`. It owns schema execution, error shaping, and variable handling.
- The HTTP `graphql_handler` keeps building `Identity` from headers (unchanged behaviour) and calls `execute`.
- The MCP `graphql` tool builds nothing itself; the MCP auth middleware produces the `Identity` and the tool passes it into `execute`. No task-local.
- The MCP `graphql_schema` tool runs its introspection against the same in-process schema, keeping the existing static cache.
- Delete the MCP-side `AppContext` construction; the shared app context is passed into the execution module.
- The execution module is the seam for both callers and tests — behaviour is identical regardless of entry point.

## Testing Decisions

- A good test asserts external behaviour only: the same query + identity yields the same filtered rows whether invoked through the HTTP handler or the MCP interface.
- Modules tested: the query-execution module (new), exercised through its interface; existing HTTP behaviour via the hurl suite unchanged.
- Prior art: `circuit_breaker.rs` unit tests (pure module tested directly); `tests/*.hurl` for black-box HTTP behaviour.
- New tests: unit/integration tests that build the schema in-process, call `execute` with identities for two tenants, and assert tenant isolation (the current bypass would fail them).
- Existing `tests/mcp.hurl` (endpoint reachability, 401 without JWT) stays green; it cannot assert tool bodies, so the new unit seam is the primary coverage for tool behaviour.

## Out of Scope

- Consolidating the three JWT validation implementations (separate spec: `single-identity-auth-module`).
- Numeric/typed filter and null-write fixes (separate spec: `typed-numeric-and-null-fields`).
- Any change to the `discover_tables` DTO layering.

## Further Notes

This is the highest-leverage deepening in the review: it closes a live security hole, deletes dead construction, and makes the MCP tools testable for the first time. It also stages the auth work — the `Identity` it threads through is the same object the auth module will produce.
