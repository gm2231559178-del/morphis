#!/usr/bin/env bash
set -euo pipefail

# Seeds the materials index from the contract fixtures (contract/materials/*.json).
# Every document is the canonical shape produced by the indexing pipeline, so the
# seeded data never drifts from what pgx emits. The contract is validated by the
# fixture test in src/schema/search/contract.rs.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONTRACT_DIR="${CONTRACT_DIR:-$SCRIPT_DIR/contract/materials}"

ES_URL="${ES_URL:-http://localhost:9200}"
ES_AUTH="${ES_AUTH:--u elastic:morphis_es_pass}"
INDEX="${1:-materials}"

echo "=== Creating index: $INDEX ==="
curl -s $ES_AUTH -X PUT "$ES_URL/$INDEX" -H "Content-Type: application/json" -d '{
  "settings": { "number_of_shards": 1, "number_of_replicas": 0 }
}' | python3 -m json.tool

echo ""
echo "=== Indexing documents from $CONTRACT_DIR ==="

for fixture in "$CONTRACT_DIR"/*.json; do
  [ -e "$fixture" ] || { echo "  ERROR: no fixtures found in $CONTRACT_DIR"; exit 1; }
  mat_no="$(basename "$fixture" .json)"
  echo "  indexing $mat_no"
  curl -s $ES_AUTH -X POST "$ES_URL/$INDEX/_doc/$mat_no" \
    -H "Content-Type: application/json" \
    --data-binary "@$fixture" | python3 -m json.tool
done

echo ""
echo "=== Refreshing index ==="
curl -s $ES_AUTH -X POST "$ES_URL/$INDEX/_refresh" | python3 -m json.tool

echo ""
echo "=== Verifying count ==="
curl -s $ES_AUTH "$ES_URL/$INDEX/_count" | python3 -m json.tool

echo ""
echo "=== Done ==="
