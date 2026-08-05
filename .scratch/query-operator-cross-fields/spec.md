# Spec: Query operator for cross-field search

Status: ready-for-agent
Priority: 7
Feature: query-operator-cross-fields

## Problem Statement

A user of the search endpoint cannot express how the free-text `query` argument combines its terms. Today a non-empty `query` on a `search{Index}` field always becomes an Elasticsearch `multi_match` with `type: cross_fields` and the default term operator (OR) — a document matches if *any* query term appears in *any* searchable field. There is no way to require *all* terms to appear, so a multi-word query like `"flame retardant"` returns every document containing either word, diluting precision.

## Solution

Add a `queryOperator` argument to every `search{Index}` field accepting `AND` or `OR` (default `OR`, preserving current behaviour). With `AND`, every term in the `query` must appear somewhere in the document's searchable fields (top-level and joined); with `OR` (current default), any term suffices. The full-text clause is the same cross-fields `multi_match`; only the term operator changes. Empty `query` stays a no-op regardless of the argument.

## User Stories

1. As a user searching materials, I want `searchMaterials(query: "cotton canvas", queryOperator: AND)` to return only materials containing both "cotton" and "canvas", so that multi-word searches are precise.
2. As a user, I want `searchMaterials(query: "cotton canvas", queryOperator: OR)` to behave exactly as it does today, so that existing queries keep their meaning.
3. As a user omitting `queryOperator`, I want the query to use OR semantics, so that current callers need no changes.
4. As a user, I want the `queryOperator` argument to apply across joined fields too (e.g. a term in the material name and a term in a feature description), so that cross-field AND matching spans the whole searchable document.
5. As a user passing an empty `query`, I want `queryOperator` to have no effect, so that result sets without a text query are unchanged.
6. As a developer, I want the term-combination logic to be verifiable against the stub ES client, so that AND semantics are tested without a live cluster.
7. As an API consumer, I want the schema to document the new argument with an enum type, so that GraphQL tooling validates values client-side.

## Implementation Decisions

- Add an `ENUM` input argument `queryOperator` (values `AND`, `OR`) to the `search{Index}` field in the search schema assembly; it is optional and defaults to `OR`.
- When `query` is non-empty, the full-text clause remains a single `multi_match` over the collected searchable fields with `type: cross_fields`. With `queryOperator: AND`, set the multi_match `operator` to `and`; otherwise omit it (ES default `or`).
- The existing `minimum_should_match: 1` should clause is unchanged in both modes.
- The stub ES client's `multi_match` matcher gains term-aware semantics: it tokenises the query on whitespace and, when the body carries `operator: and`, requires every term to match some searchable field; otherwise it matches when any term does. Both modes keep case-insensitive substring matching so the stub mirrors the live client's analysis.
- The raw `esQuery` escape hatch is untouched — `queryOperator` only shapes the generated full-text clause.

## Testing Decisions

- A good test asserts external behaviour: a `search{Index}` query with `queryOperator` returns exactly the hits whose searchable fields satisfy the term combination.
- Module tested: the search module end-to-end through its schema, with the stub ES adapter.
- Prior art: `filter_operators_are_interpreted_against_stub_es` in `src/schema/search/mod.rs` — stub service + `build_schema_with_search` + `run()` against a three-document fixture; `tests/search.hurl` for the live end-to-end path.
- New unit tests (extend the existing search-module test): OR returns docs matching any term; AND returns only docs matching every term; AND requires the terms to be satisfied across the whole document (top-level and joined fields); empty query ignores the argument; default (no argument) equals OR.
- New hurl tests in `tests/search.hurl` using the seeded materials: `query: "wool canvas"` with OR returns M001 + M002; with AND returns empty; `query: "cotton canvas"` with AND returns M001; omit the argument and assert OR behaviour is unchanged.
- Regression: the existing suite (`health` → `mutations` → cleanup → `queries` → `relations` → `search` → `row_filters` → `auth_proxy` → `mcp`) stays green.

## Out of Scope

- Fuzzy, phrase, or per-field weighting in the free-text query — only the AND/OR term combination is in scope.
- Operator selection for the `filter` argument (already handled by the operator registry).
- Making the operator configurable per index or globally — it is a per-call argument.

## Further Notes

The seeded materials give a clean discriminator: "Premium Cotton Canvas", "Merino Wool Blend", "Recycled Polyester". `"wool canvas"` is the key test — OR matches M001 (canvas) and M002 (wool); AND matches neither, since no single document contains both terms.
