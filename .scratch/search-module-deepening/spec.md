# Spec: Search module deepening

Status: ready-for-agent
Priority: 5
Feature: search-module-deepening

## Problem Statement

Search behaviour is split by concern rather than by feature, so a developer changing search must hold three files in their head:

- Operators are *declared* as input types in the schema assembly module and *interpreted* independently in the search module (a hardcoded `OPERATOR_KEYS` list), with no shared notion of an operator.
- Row filters — the same concept that compiles to SQL in `apply_row_filters` — are compiled *again* into ES JSON by the search module, with a different subquery strategy and its own permission cache. Two compilers, no shared interpretation.
- The search module reads app-context internals field-by-field (ES client, ES URL, circuit breaker, pool) instead of going through an ES-client seam, and it owns the only `PermissionCache` instance, which lives in the config module.

The consequence: adding one operator touches three files (`mod`, `search`, `input`); row-filter semantics can silently diverge between the PG and ES paths; and the ES client cannot be substituted in tests.

## Solution

Fold operator schema, operator interpretation, row-filter compilation, the permission cache, and the ES client adapter into one search module behind a single interface — `search(index, args, identity) -> hits`. The module owns one operator registry, one row-filter compiler (producing both SQL and ES output), and its ES client behind a seam that has two adapters (live client in production, stub in tests). The permission cache moves out of the config module into the search module.

## User Stories

1. As a developer adding an operator, I want to edit one module, so that schema and interpretation cannot drift.
2. As a developer, I want one row-filter compiler for both PG and ES, so that filter semantics are identical across the two search paths.
3. As a tester, I want the ES client at a seam, so that the search module is testable against a stub instead of a live cluster.
4. As a developer, I want the permission cache owned by the module that uses it, so that runtime state stops living in the config module.
5. As a developer, I want the search module to stop reading app-context internals field-by-field, so that it consumes its dependencies through a seam.
6. As a developer, I want cache-hit and TTL behaviour testable, so that the subquery permission cache is verified.
7. As a user, I want PG-list and ES-search filters to interpret the same filter config the same way, so that behaviour is consistent across entry points.

## Implementation Decisions

- One search module owning: operator input-type generation, operator interpretation, row-filter compilation, the permission cache, and the ES client adapter.
- An internal operator registry: one table of operator metadata (name, input shape, ES clause builder) consulted by both schema generation and interpretation. The schema assembly module stops defining operator inputs; the `OPERATOR_KEYS` hardcoding in the search module is deleted.
- One row-filter compiler driven by the row-filter config, emitting either SQL fragments (for PG resolvers) or ES `should`/`term` clauses (for the search path), so the two paths share the same interpretation and subquery strategy.
- The ES client sits behind an internal seam with two adapters: `reqwest` in production, an in-memory stub in tests — the two-adapter rule makes the seam real.
- The permission cache moves from the config module into the search module; its TTL and hit behaviour become unit-testable.
- The search module receives its dependencies (client, URL, breaker, pool) explicitly at construction rather than reaching into the app context.

## Testing Decisions

- A good test asserts external behaviour: a search query returns the hits the filter semantics say it should.
- Modules tested: the search module through its interface, with a stub ES adapter; the row-filter compiler (both SQL and ES output for the same config); the permission cache (hit, miss, TTL).
- Prior art: `circuit_breaker.rs` unit tests (module tested directly); `tests/search.hurl` and `tests/row_filters.hurl` for end-to-end ES behaviour.
- New tests: operator interpretation for each operator type against a stub ES; row-filter config compiled to SQL and to ES produce equivalent semantics; cache TTL expiry.
- Regression: `tests/search.hurl` and `tests/row_filters.hurl` stay green unchanged — they are the external contract for search behaviour.

## Out of Scope

- Typed numeric/null list-filter fixes (separate spec: `typed-numeric-and-null-fields`) — the PG filter compiler stays in the schema layer until that lands.
- The per-table generation collapse (separate spec: `per-table-schema-collapse`) — keeps the operator inputs' current home until the search module is ready to take them.
- The ES document-contract work (separate spec: `es-document-contract`) — write-side mapping is untouched.

## Further Notes

This spec depends on the operator inputs moving out of the schema assembly module, so it should be sequenced after the per-table collapse (which keeps them put) or coordinated with it. The permission-cache relocation is safe to do first on its own — it is a small, self-contained move with an immediate testability win.
