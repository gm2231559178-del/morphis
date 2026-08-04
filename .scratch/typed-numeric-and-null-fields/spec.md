# Spec: Typed numeric and null field handling

Status: ready-for-agent
Priority: 3
Feature: typed-numeric-and-null-fields

## Problem Statement

A user of the GraphQL endpoint gets silently wrong results for non-string data, and writes data it did not intend:

- **Numeric and boolean list filters are dropped.** The filter input schema advertises `Int`, `Float`, and `Boolean` fields (e.g. `sizesList(filter: { id: 3 })`), but the filter compiler only emits clauses for string values. Every numeric/boolean filter is silently ignored, so `feature_attributesList(filter: { feature_id: 1 })` returns attributes from every feature instead of just feature 1.
- **Nulls are written as empty strings.** Creating or updating a row with a nullable field set to `null` stores `''` instead of SQL `NULL`. For text columns this corrupts semantics (`IS NULL` and default-value checks no longer hold); for an integer column the write fails outright.
- **Int64 columns are exposed as GraphQL `Int` (i32).** A `bigint` value larger than 2^31 loses precision or errors. No `Int64` column exists in the current config, so this is latent but wrong.
- **The real filter logic is untested.** The unit tests for the filter compiler re-implement its loop instead of calling it, because its interface takes a value type that cannot be constructed in a test.

## Solution

Make the GraphQL layer honour the types it advertises: the filter compiler handles string, integer, float, and boolean values; writes distinguish "column absent" from "column null" and emit SQL `NULL`; and column scalar mapping is corrected so integer columns round-trip without loss. Along the way, make the filter compiler's core a pure, testable function so the numeric clauses are actually exercised.

## User Stories

1. As a user querying `sizesList`, I want `filter: { id: 3 }` to return only size 3, so that numeric list filters work like string filters.
2. As a user querying `feature_attributesList`, I want `filter: { feature_id: 1 }` to return only feature 1's attributes, so that filtering by a foreign-key integer works.
3. As a user, I want float and boolean filters to work, so that the schema does not advertise filters it ignores.
4. As a user creating a material, I want to leave a nullable column unset or null without storing an empty string, so that `NULL` stays `NULL`.
5. As a user updating a row, I want to clear a field by setting it to null, so that the column becomes `NULL` rather than `''`.
6. As a database operator, I want `NULL` written as SQL `NULL`, so that `IS NULL` queries, not-null constraints, and default values behave correctly.
7. As a user of large integer identifiers, I want `bigint` values read back without precision loss, so that IDs larger than 2^31 are not corrupted.
8. As a developer, I want the filter compiler's core to be a pure testable function, so that numeric/boolean clauses are unit-tested instead of re-implemented in the tests.

## Implementation Decisions

- The filter compiler matches string, integer, float, and boolean values and emits `column = $n` for each, with the value bound as text (Postgres coerces to the column type). Null values and unknown columns are still skipped.
- Split the compiler into a pure core — a mapping from (column, typed value) pairs to SQL clauses — and a thin adapter that extracts those pairs from the GraphQL value type. The pure core is the unit-test surface; the current tests that re-implement the loop are deleted and replaced by tests that call it.
- Writes: in `create`, a column explicitly set to `null` is skipped entirely (the DB default applies); in `update`, a column set to `null` emits an SQL `NULL` clause. A column simply absent from the input is untouched. This requires distinguishing "key present with null" from "key absent" when reading input objects.
- Int64 columns map to a `BigInt` scalar in object types, inputs, and search operator types; `Int` stays `Int`. `Float` and `Boolean` mappings are unchanged (already correct).
- Nullable columns: either the object field type is made nullable for columns declared `nullable: true`, or the mismatch is documented deliberately — the spec's decision is to make field nullability follow column nullability so the schema stops lying.
- No config change is needed to ship this: `sizes.id`, `colorways.id`, `material_features.id`, `feature_attributes.id`, `feature_attributes.feature_id`, and `user_permissions.id` are the int columns the new filters and null-writes apply to.

## Testing Decisions

- A good test asserts external behaviour: a filter returns the filtered rows; a null write stores `NULL`.
- Modules tested: the filter compiler's pure core (unit); the mutation create/update null paths (unit + hurl); the scalar mapping (unit).
- Prior art: `util.rs` unit tests (pure functions tested directly); `tests/queries.hurl` and `tests/mutations.hurl` for end-to-end behaviour.
- New hurl tests: `feature_attributesList(filter: { feature_id: 1 })` returns only feature 1's attributes; `sizesList(filter: { id: ... })` filters by id; create a row with a nullable column omitted and with it `null`, then read back and assert `NULL`; update a row's nullable column to `null` and assert `NULL`.
- New unit tests: numeric/boolean filter clauses are generated with correct bind order; null value produces no clause (create) / an SQL `NULL` clause (update); the current re-implemented tests are removed.
- Regression: the existing suite (`health` → `mutations` → cleanup → `queries` → `relations` → `search` → `row_filters` → `auth_proxy` → `mcp`) must stay green.

## Out of Scope

- Full operator support in PG list filters (e.g. `gt`, `lt`, `in`) — this spec covers exact-equality on typed values only. Operators are a search-module concern.
- Search-side numeric behaviour — the ES operator path already handles integers/floats/booleans correctly; only schema typing for `Int64` is touched.
- The nullability of relation fields (only scalar column fields are in scope).

## Further Notes

This is the cheapest spec to ship and the most user-visible: the schema advertises filters it silently drops. The `filter: { feature_id: 1 }` case is a realistic query (finding a material's attributes) that returns incorrect data today. The test-surface fix is part of the value — today the filter logic is provably untested, which is how the string-only bug survived.
