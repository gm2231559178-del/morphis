# Spec: Batch relation resolvers to kill the N+1 query storm

Status: ready-for-agent
Priority: 7
Feature: n1-relation-batching

## Problem Statement

A GraphQL query that returns a list of parent rows (e.g. a `searchMaterials` hit list of 10 materials) and selects a nested relation on each (sizes, colorways, material_features, feature_attributes) fires one SQL query per parent row per relation level. A single 10-hit search produced ~180 Postgres queries (per-row `WHERE matnr = $1` and `WHERE attrib_id = $1 LIMIT 1` storms), turning every browse/list call into an N+1 cascade that grows with page size and nesting depth.

## Solution

Relation resolution happens once per relation per request level, not once per parent row. All parent FKs at one depth are gathered and fetched in a single batched query (`WHERE foreign_field::text = ANY($1)`), then distributed back to their parents. Nested relations (material_features → feature_attributes) repeat the pattern one level at a time, so total SQL is O(relation levels), not O(parent rows × relation levels). Row filters, ordering, and belongs_to semantics are preserved exactly.

## User Stories

1. As a user querying `searchMaterials { sizes }`, I want all 10 hits' sizes fetched in one SQL query, so that search responses scale with result size instead of exploding into N queries.
2. As a user querying `materialsList { material_features { feature_attributes } }`, I want each relation level to cost one query, so that nested relations stay O(depth) not O(rows × depth).
3. As a user, I want the values returned by batched relations to be byte-identical to today's per-row results, so that no API response changes.
4. As a user of a has_many relation, I want children returned in the current `ORDER BY primary_key` order per parent, so that deterministic ordering is preserved.
5. As a user of a belongs_to relation, I want at most one row returned per parent, so that single-object semantics are preserved.
6. As a tenant-restricted user, I want row filters (X-Tenant-ID column filter, X-User-ID subquery filter) applied inside the batched query, so that batching never leaks rows across tenants.
7. As a user querying composite-key relations, I want batching to work for multi-column FKs, so that no existing relation shape is degraded.
8. As a developer, I want the batch query builder and result distribution to be pure functions I can unit test without a database, so that the "one query for N keys" invariant is provable in CI without Docker.
9. As a developer, I want the loader to emit a trace line with the number of keys served per query, so that the integration runner can assert the N+1 storm is gone end-to-end.

## Implementation Decisions

- Enable the `dataloader` feature of `async-graphql` (pulls in `futures-channel` + `lru`), and build the batched relation resolver on top of async-graphql's `Loader` trait and `DataLoader` (1ms coalescing window, `max_batch_size` default). The mechanism is sound in this codebase: async-graphql's dynamic executor resolves list items concurrently (`try_join_all`), so all parent relation loads at one depth land inside the same coalescing window and collapse into one `load()` call.
- One `RelationLoader` implementing `Loader<RelKey>` is registered on the schema builder once, alongside the existing app context, and relation resolvers read it from the GraphQL context. No change to the HTTP/MCP request path is needed — batching is entirely inside the schema.
- The load key `RelKey` carries everything the loader needs and nothing shared across requests: (a) a relation identity — related table, FK field(s) with int-cast flags, list-vs-single, and the has_many ORDER BY; (b) the row-filter SQL suffix plus its bound params, precomputed by the resolver from the request `Identity` and the relation's `row_filters` config (the same compiler the per-row path uses today); and (c) the FK value for that parent row. Because row filters are baked into the key, a schema-global loader cannot leak rows across tenants or requests, and needs no per-request state or cache.
- `load()` groups the received keys by (relation identity, filter suffix) and issues exactly one statement per group. The query shape mirrors the existing batch-enrichment dialect: `SELECT ... FROM (SELECT * FROM {table} WHERE {foreign_field}::text = ANY($1) ... ) t` with the FK array bound via the existing array bind, followed by the precomputed filter params. Composite FKs use the `(f1, f2) IN (SELECT * FROM unnest($1::text[], $2::text[]))` form.
- has_many: children are grouped per parent FK, preserving the current `json_agg(... ORDER BY primary_key)` order. belongs_to: the batch keeps at most one row per parent FK (DISTINCT ON the foreign field), matching today's `LIMIT 1`.
- The per-row SQL execution in the relation resolver is deleted; the resolver instead extracts the FK from the parent JSON, builds the `RelKey`, awaits `load_many`, and maps the returned children to a `FieldValue`. Error handling and the `Identity`-driven filter compilation move unchanged into the key builder.
- The batch enrichment compensation layer stays untouched and out of scope — its job (fresh child data on search hits) is now covered more cheaply by the batched resolver, but removing it is a separate change with its own risk.

## Testing Decisions

- A good test asserts external behaviour with no behavioural change: same query, same results, but relation resolution is O(levels) not O(rows). The core invariant — "N FK keys produce exactly 1 SQL statement" — is asserted at the pure-function level, because no live Postgres is available in local development (no Docker).
- Module tested: the new relation-loader module, plus the existing relation/schema path unchanged as a regression net.
- Prior art: `key_value_normalises_strings_and_numbers` in the search module (pure JSON-to-JSON helpers tested without a DB); `tests/relations.hurl`, `tests/search.hurl`, `tests/queries.hurl` for the live path; the search `stub_service` + `build_schema_with_search` + `run()` harness for schema-level assertions without a live ES/DB.
- New unit tests (no DB): the batch query builder emits one `ANY($1)` statement for N keys; grouping preserves has_many ORDER BY per parent; belongs_to yields at most one row per parent; composite FK keys produce the unnest form; filter suffix + params are appended after the FK array bind and differ per identity; empty FK set short-circuits without a query.
- New schema-level unit tests: a list query selecting a nested relation still resolves through the existing seam with identical shape (stub service), guarding against the resolver returning malformed values.
- New hurl tests (CI/Docker only): seed a page of materials with children, assert `searchMaterials { sizes { ... } material_features { feature_attributes { ... } } }` returns full nested data; existing relations/search suites stay green as the batched-path regression.
- New integration assertion (CI/Docker only): the loader logs `relation_batch keys=N queries=1` per level; the test runner greps container output to prove the storm is gone end-to-end. Skipped when Docker is unavailable.
- Regression: the existing suite (`health` → `mutations` → cleanup → `queries` → `relations` → `search` → `row_filters` → `auth_proxy` → `mcp`) stays green.

## Out of Scope

- Removing or rewiring the batch enrichment compensation layer — the batched resolver makes its relation work redundant, but deprecation is a follow-up.
- Changing the GraphQL API surface, response shapes, relation semantics, or row-filter behaviour.
- Caching relation results across requests — a schema-global `NoCache` loader guarantees fresh reads.
- Per-relation pagination, filtering, or lazy loading of relation children.

## Further Notes

The production incident this spec fixes was a 10-hit `_search` causing ~180 SQL statements. After this change the same query costs 1 ES search + O(levels) batched SQL. Because the loader is keyed by relation identity + FK value with filters baked in, it is safe to share across requests, and nested relations batch one level at a time without any query planner.
