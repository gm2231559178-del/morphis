# Spec: ES document contract — one shape, one producer

Status: ready-for-agent
Priority: 6
Feature: es-document-contract

## Problem Statement

For a developer (or an AI agent) trying to answer "how does a material get indexed and searched", the same fact is encoded five times in five places — `db/init.sql`, the search config's `join_fields`, `pgx/config.toml` resolvers, `pgx/queries/material.graphql`, and `pgx/schema/types.graphql` — behind a black-box binary (`ghcr.io/jyasuu/pg_x:main`) whose source is not in this repo. Worse:

- Two parallel producers run concurrently and write the same index: the direct `pgx-listen` path and the RabbitMQ-decoupled `pgx-listen-rabbitmq → queue → pgx-consume` path. A third shape is produced by `seed_es.sh`, which curls ES directly and writes documents that differ from the contract.
- **Child-table writes never re-index the parent.** Only `materials` has triggers. Editing a size, colorway, or feature attribute silently leaves the ES document stale.
- The read path compensates: `es_batch_enrich` re-fetches children from Postgres at query time and re-attaches them to search hits, so search results contain fresh child data only for the returned fields — filter fields still see the stale document.

## Solution

Make the ES document shape a single, fixture-tested contract module in this repo, have one producer write to each index, fire parent re-indexing when child tables change, and stop the read path from compensating for staleness. Because the producer binary is external, the contract is expressed as fixtures and validation this repo owns, and the two write pipelines are reduced to one.

## User Stories

1. As a developer, I want the material document shape defined once and tested, so that the five encodings stop drifting.
2. As a user searching materials, I want an edited size or feature attribute to be reflected in search, so that child-table writes re-index the parent material.
3. As an operator, I want one producer per index, so that two pipelines and a seed script cannot write conflicting shapes into the same index.
4. As a developer, I want `seed_es.sh` to produce the contract shape, so that seeded data matches what the pipeline produces.
5. As a tester, I want the contract verifiable in CI without the external binary, so that a shape change is caught by a fixture test.
6. As a user of search, I want filter fields and returned fields to agree, so that search does not return a document that contradicts the freshly-hydrated child data.
7. As a developer, I want to remove the query-time re-hydration once the contract is stable, so that the read path stops compensating for write-side staleness.

## Implementation Decisions

- A contract module in this repo: the canonical document shape (parent + sizes + colorways + features + feature_attributes) expressed as fixtures and a schema, with a validation test that the shape satisfies the search fields the read path depends on.
- Add triggers on the child tables (`sizes`, `colorways`, `material_features`, `feature_attributes`) so a child INSERT/UPDATE/DELETE fires the parent material's indexing — the existing trigger mechanism extended to fire on the same channel with the parent key.
- Reduce producers to one per index: the RabbitMQ-decoupled pipeline is the surviving producer; the direct `pgx-listen` path and the `seed_es.sh` direct curl are retired (seed data flows through the pipeline instead).
- Align `seed_es.sh` with the contract shape so seeded documents match pipeline output.
- `es_batch_enrich` is removed only after the contract is stable and child re-indexing is proven by tests; until then it stays as a compensation layer with a documented reason.
- The pipeline wiring (exchange, queue, routing key) moves from a shell one-liner in compose into a declared, testable configuration.

## Testing Decisions

- A good test asserts the contract: a document produced by the pipeline satisfies the fixture shape, and a child-table write causes the parent document to be re-indexed.
- Modules tested: the contract fixtures (schema validation without the external binary); the child-trigger behaviour (write to a child table, assert the parent re-indexes); the reduced pipeline (end-to-end in the integration suite).
- Prior art: `tests/search.hurl` and `tests/row_filters.hurl` (search over seeded ES); `tests/docker-entrypoint.sh` (side-effect orchestration).
- New tests: a fixture test validating the contract shape; an integration assertion that updating a `material_features` row changes the material's search document; a test that seeded documents match the contract.
- Blocked-until-change: this spec's tests require the pipeline to run in CI, which the backend integration script currently does not start (`integration-check.sh` omits rabbitmq/pgx).

## Out of Scope

- Bringing the producer binary's source into this repo — that is a prerequisite decision, not part of this spec. If the producer cannot be brought in-repo, the contract is enforced purely by fixtures and validation at the seam.
- Logical-decoding replication or any change to the LISTEN/NOTIFY mechanism itself.
- The read-path search module refactor (separate spec: `search-module-deepening`) — `es_batch_enrich` removal is coordinated with it, not part of it.

## Further Notes

This is the longest-pole spec: it is constrained by an external binary, two live pipelines, and the seed path. It is ranked last because its value depends on the producer question being settled (bring the binary in-repo, or enforce the contract at the seam). The child-table trigger gap is the one piece worth doing first on its own — it is small, inside the repo, and removes a real staleness bug.
