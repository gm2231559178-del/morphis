//! Canonical shape of a material search document — the P6 contract.
//!
//! The document shape is defined once here as a typed schema and enforced
//! against the fixtures in `contract/materials/*.json`. Every producer of the
//! `materials` index (the pgx pipeline and `seed_es.sh`) must emit documents
//! that deserialize into [`MaterialDocument`]; this gate keeps the five
//! encodings (db schema, search config join_fields, pgx resolvers, pgx
//! queries/schema, seed fixtures) from drifting.

#![allow(dead_code)] // fields are schema-enforced via serde, not read in Rust

use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterialDocument {
    mat_no: String,
    name: String,
    status: String,
    tenant_id: Option<String>,
    sizes: Vec<Size>,
    colorways: Vec<Colorway>,
    material_features: Vec<MaterialFeature>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Size {
    id: i64,
    size_code: String,
    mat_no: String,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Colorway {
    id: i64,
    colorway_code: String,
    mat_no: String,
    name: String,
    hex: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterialFeature {
    id: i64,
    feature_id: i64,
    mat_no: String,
    feature_name: String,
    description: Option<String>,
    feature_attributes: Vec<FeatureAttribute>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FeatureAttribute {
    id: i64,
    feature_id: i64,
    attr_name: String,
    attr_value: String,
}

impl MaterialDocument {
    /// The search fields the read path depends on (config `search_indexes`
    /// searchable_fields/join_fields): multi_match targets and filter paths.
    /// A document satisfies the contract only if every such path is present
    /// with non-null values somewhere in the document.
    fn satisfies_read_path(&self) -> Result<(), String> {
        for (val, name) in [
            (&self.mat_no, "mat_no"),
            (&self.name, "name"),
            (&self.status, "status"),
        ] {
            if val.is_empty() {
                return Err(format!("top-level field {name} is empty"));
            }
        }
        if self.material_features.is_empty() {
            return Err("material_features must not be empty".into());
        }
        let mut has_description = false;
        for feature in &self.material_features {
            if feature.feature_name.is_empty() {
                return Err(format!("feature {} has an empty feature_name", feature.id));
            }
            if feature.description.as_deref().is_some_and(|d| !d.is_empty()) {
                has_description = true;
            }
            if feature.feature_attributes.is_empty() {
                return Err(format!("feature {} has no feature_attributes", feature.id));
            }
            for attr in &feature.feature_attributes {
                if attr.attr_name.is_empty() || attr.attr_value.is_empty() {
                    return Err(format!(
                        "attribute {} has empty attr_name/attr_value",
                        attr.id
                    ));
                }
            }
        }
        if !has_description {
            return Err("no material_feature carries a non-null description".into());
        }
        Ok(())
    }
}

#[test]
fn contract_fixtures_deserialize_and_satisfy_the_read_path() {
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("contract/materials");
    let mut fixtures: Vec<_> = std::fs::read_dir(&fixtures_dir)
        .expect("contract/materials must exist")
        .map(|entry| entry.expect("read dir entry").path())
        .filter(|path| path.extension().map(|ext| ext == "json").unwrap_or(false))
        .collect();
    assert!(!fixtures.is_empty(), "contract/materials must contain fixtures");
    fixtures.sort();

    for path in fixtures {
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        let doc: MaterialDocument = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("{} does not match the contract: {e}", path.display()));
        doc.satisfies_read_path()
            .unwrap_or_else(|reason| panic!("{} violates the read-path contract: {reason}", path.display()));
    }
}
