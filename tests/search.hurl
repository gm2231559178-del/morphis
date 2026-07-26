POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"cotton\") { mat_no name status } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M001"
jsonpath "$.data.searchMaterials[0].name" == "Premium Cotton Canvas"
jsonpath "$.data.searchMaterials[0].status" == "active"

POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\") { mat_no name status } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 3

# eq operator: filter by single status
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", filter: { status: { eq: \"active\" } }) { mat_no name status } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 2
jsonpath "$.data.searchMaterials[*].mat_no" contains "M001"
jsonpath "$.data.searchMaterials[*].mat_no" contains "M002"

# ne operator: exclude active materials
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", filter: { status: { ne: \"active\" } }) { mat_no name status } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M003"
jsonpath "$.data.searchMaterials[0].status" == "discontinued"

# in operator: match multiple statuses (OR)
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", filter: { status: { in: [\"active\", \"discontinued\"] } }) { mat_no name status } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 3

# in operator: match subset of statuses
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", filter: { status: { in: [\"discontinued\"] } }) { mat_no name status } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M003"

# contains operator: substring match on name
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", filter: { name: { contains: \"Cotton\" } }) { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M001"

# starts_with operator
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", filter: { name: { starts_with: \"Merino\" } }) { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M002"

# ends_with operator
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", filter: { name: { ends_with: \"Polyester\" } }) { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M003"

# Multiple operators on same field: name contains AND status eq
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", filter: { name: { contains: \"Wool\" }, status: { eq: \"active\" } }) { mat_no name status } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M002"

# Nested filter: eq on feature_name
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", filter: { material_features: { feature_name: { eq: \"Eco\" } } }) { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M003"

# Nested filter: in on feature_name (OR) — Construction + Eco
# M001 has Construction+Care, M002 has Construction+Certification, M003 has Construction+Eco
# All 3 have Construction, so all match
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", filter: { material_features: { feature_name: { in: [\"Eco\", \"Construction\"] } } }) { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 3

# Nested filter: in on feature_name — Certification only (only M002)
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", filter: { material_features: { feature_name: { in: [\"Certification\"] } } }) { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M002"

# Nested filter: all on feature_name (AND) — material must have BOTH Construction AND Eco
# M001: Construction+Care (no Eco) → no match
# M002: Construction+Certification (no Eco) → no match
# M003: Construction+Eco → match
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", filter: { material_features: { feature_name: { all: [\"Construction\", \"Eco\"] } } }) { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M003"

# Nested filter: all on feature_name — Construction AND Certification
# Only M002 has both
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", filter: { material_features: { feature_name: { all: [\"Construction\", \"Certification\"] } } }) { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M002"

# Deeply nested filter: eq on attr_value
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", filter: { material_features: { feature_attributes: { attr_value: { eq: \"OEKO-TEX\" } } } }) { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M002"

# Combined: nested eq + top-level eq
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", filter: { status: { eq: \"active\" }, material_features: { feature_name: { eq: \"Construction\" } } }) { mat_no name status } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 2
jsonpath "$.data.searchMaterials[*].mat_no" contains "M001"
jsonpath "$.data.searchMaterials[*].mat_no" contains "M002"
jsonpath "$.data.searchMaterials[*].status" contains "active"

# Combined: text query + operator filter
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"wool\", filter: { material_features: { feature_name: { eq: \"Construction\" } } }) { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M002"

POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"wool\") { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M002"

POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"plain\") { mat_no name material_features { feature_name description } } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M001"
jsonpath "$.data.searchMaterials[0].material_features" count == 3

# Paging: limit returns fewer results
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", limit: 1) { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1

# Paging: limit of 2 on all results
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", limit: 2) { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 2

# Paging: first page with limit 2, offset 0 = items 0-1
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", limit: 2, offset: 0) { mat_no } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 2

# Paging: second page with offset past first page
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", limit: 2, offset: 2) { mat_no } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1

# Paging: offset beyond total returns empty
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(query: \"\", limit: 10, offset: 100) { mat_no } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 0

# Raw ES query: filter by nested feature_name = "Eco"
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(esQuery: \"{\\\"bool\\\":{\\\"must\\\":[{\\\"term\\\":{\\\"material_features.feature_name.keyword\\\":\\\"Eco\\\"}}]}}\") { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M003"

# Raw ES query: filter by feature_name = "Certification"
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(esQuery: \"{\\\"bool\\\":{\\\"must\\\":[{\\\"term\\\":{\\\"material_features.feature_name.keyword\\\":\\\"Certification\\\"}}]}}\") { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M002"

# Raw ES query: filter by feature_name = "Construction" (all 3 have it)
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(esQuery: \"{\\\"bool\\\":{\\\"must\\\":[{\\\"term\\\":{\\\"material_features.feature_name.keyword\\\":\\\"Construction\\\"}}]}}\") { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 3

# Raw ES query: bool must + should (feature=Construction AND name contains Wool)
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(esQuery: \"{\\\"bool\\\":{\\\"must\\\":[{\\\"term\\\":{\\\"material_features.feature_name.keyword\\\":\\\"Construction\\\"}}],\\\"should\\\":[{\\\"match\\\":{\\\"name\\\":\\\"Wool\\\"}}],\\\"minimum_should_match\\\":1}}\") { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M002"

# Raw ES query: nested attribute match (attr_value = "OEKO-TEX")
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(esQuery: \"{\\\"term\\\":{\\\"material_features.feature_attributes.attr_value.keyword\\\":\\\"OEKO-TEX\\\"}}\") { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M002"

# Raw ES query: with limit
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(esQuery: \"{\\\"match_all\\\":{}}\", limit: 2) { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 2

# Raw ES query: terms filter (OR on feature_name)
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(esQuery: \"{\\\"terms\\\":{\\\"material_features.feature_name.keyword\\\":[\\\"Eco\\\",\\\"Certification\\\"]}}\") { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 2
jsonpath "$.data.searchMaterials[*].mat_no" contains "M002"
jsonpath "$.data.searchMaterials[*].mat_no" contains "M003"

# Raw ES query: bool must with terms (AND across multiple feature_names)
POST http://localhost:4000/graphql
Content-Type: application/json
{
  "query": "{ searchMaterials(esQuery: \"{\\\"bool\\\":{\\\"must\\\":[{\\\"term\\\":{\\\"material_features.feature_name.keyword\\\":\\\"Construction\\\"}},{\\\"term\\\":{\\\"material_features.feature_name.keyword\\\":\\\"Eco\\\"}}]}}\") { mat_no name } }"
}
HTTP 200
[Asserts]
jsonpath "$.data.searchMaterials" count == 1
jsonpath "$.data.searchMaterials[0].mat_no" == "M003"
