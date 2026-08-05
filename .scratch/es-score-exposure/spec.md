# Spec: Expose ES relevance score on search hits

Status: ready-for-agent
Priority: 5
Feature: es-score-exposure

## Problem Statement

A search consumer cannot tell *why* the hits came back in the order they did. Every `search{Index}` query asks Elasticsearch for relevance-ranked results, but the per-hit `_score` that ES computed is thrown away before the response reaches the caller — only the `_source` document is surfaced. A UI that wants to badge a "best match" or render results as a relevance-ranked list has no signal to work with, so it cannot distinguish a strong match from a weak one, and cannot do anything with relevance ordering beyond trusting ES's sort.

## Solution

Expose each search hit's Elasticsearch relevance `_score` alongside the document, so callers can see how relevant each hit is and render or reason about the ordering ES produced. The `search{Index}` field returns a per-index hit type carrying the matched document and its score; the `_score` value ES returned is preserved end-to-end and never re-derived. Ordering and pagination stay entirely Elasticsearch's responsibility — this change adds no `sort` argument and no client-side reordering.

## User Stories

1. As a user of a `search{Index}` query, I want each hit to carry the relevance score Elasticsearch assigned it, so that I can see how relevant each result is.
2. As a UI developer, I want to read a hit's score alongside its fields in the same query response, so that I can badge or visually rank "best match" results.
3. As a consumer of search results, I want the score to reflect Elasticsearch's own relevance computation, so that I can trust the ordering and explain it to end users.
4. As a user querying with an empty or filter-only `query`, I want a score field present on each hit even when ES assigns a constant score, so that the response shape is uniform across query modes.
5. As a consumer, I want the documents I select today (`searchMaterials { mat_no name }`) to still resolve unchanged through a `node` field, so that my field selections keep working after the response shape change.
6. As a developer, I want the score path to be testable against the in-memory ES stub, so that the wiring from ES response to GraphQL response is verified without a live cluster.
7. As a consumer of the existing suite, I want the real-ES hurl tests updated to the new hit shape and kept green, so that the live path is regression-tested in CI.
8. As a developer, I want no change to pagination, ordering, or filtering semantics, so that the risk of this change is limited to the response shape.

## Implementation Decisions

- Introduce one hit object type per search index, named after the index's `graphql_type` (e.g. `MaterialsSearchHit` for the materials index), with exactly two fields: `node` (the existing table object type, non-null) and `score` (a non-null Float).
- Change the `search{Index}` field return type from `[<graphql_type>!]!` to `[<graphql_type>SearchHit!]!`. Each element maps `_source` to `node` and `_score` to `score`.
- The hit extraction in the search path reads both `hit["_source"]` and `hit["_score"]` from the raw ES response and keeps them paired per hit; the score is passed through as-is (ES emits `null` when it does not compute a score, in which case the field resolves to `0.0` to keep the non-null contract — confirm against live ES behaviour before finalising this default).
- The batched relation enrichment path continues to operate on the document payloads (`node` values) only; it must not disturb the paired `score`.
- The in-memory ES stub gains the ability to return a `_score` per hit so the seam reproduces the live response shape (deterministic per-document values derived from the stub's matching logic, documented in the stub).
- No `sort` argument is added, and no client-side reordering is introduced — ES ordering and `from`/`size` pagination are unchanged and authoritative.

## Testing Decisions

- A good test asserts external behaviour: a `search{Index}` query returns each hit with the score ES assigned, and the documents previously returned are now reachable unchanged under `node`.
- Module tested: the search module end-to-end through the schema (`build_schema_with_search` + the stub ES adapter + `run()`), asserting both `score` values and `node` fields in one response.
- Prior art: `query_operator_and_spans_top_level_and_joined_fields` and `filter_operators_are_interpreted_against_stub_es` in the search module — stub service + `build_schema_with_search` + `run()` against a three-document fixture; `tests/search.hurl` and `tests/row_filters.hurl` for the live end-to-end path.
- New unit tests (extend the existing search-module tests): a text query returns per-hit scores in the response; an empty/filter-only query returns the same hit shape with a score; `node` resolves the document fields identically to today's flat selection; the stub returns a score per hit.
- New hurl tests in `tests/search.hurl`: assert `score` is present on each hit and `node.mat_no` matches the previously asserted flat values; update the existing assertions to the `node` shape (the response-shape change touches every search assertion).
- Regression: the existing suite (`health` → `mutations` → cleanup → `queries` → `relations` → `search` → `row_filters` → `auth_proxy` → `mcp`) stays green after the shape update.

## Out of Scope

- A `sort` argument or any form of ordering control — ordering and pagination remain Elasticsearch's job.
- Client-side or server-side re-ranking of hits by score.
- Exposing `_score` on list (`*List`) or single-object queries — the score exists only on search hits.
- Ties, score normalisation, or explain-API output (`_explanation`).
- Changing how ES computes scores (boosts, function scoring, per-field weights).

## Further Notes

The response-shape change is breaking for any consumer that selects fields directly off `search{Index}` hits: the hurl suites and the MCP `graphql` tool must switch to `node { ... }`. The frontend introspects search field arguments from the schema rather than hard-coding hit selections, so it needs no query rewrites. This is a deliberate trade — a synthetic `score` field on the shared table object type would leak relevance metadata into non-search queries and collide with real columns; the wrapper hit type keeps the score scoped to search. Seeding the stub with deterministic `_score` values is what makes the score-exposure invariant provable without Docker.
