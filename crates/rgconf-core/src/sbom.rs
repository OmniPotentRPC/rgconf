// Copyright 2023--present rgconf developers

use crate::error::Error;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub name: String,
    pub version: String,
    pub purl: String,
}

/// Read a CycloneDX JSON document. Every component with a purl is kept,
/// including cargo and generic purls.
pub fn parse_cyclonedx(text: &str) -> Result<Vec<Component>, Error> {
    let value: Value =
        serde_json::from_str(text).map_err(|err| Error::message(format!("cyclonedx: {err}")))?;
    let Some(components) = value.get("components").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for component in components {
        let Some(purl) = component.get("purl").and_then(Value::as_str) else {
            continue;
        };
        if purl.is_empty() {
            continue;
        }
        let name = component
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let version = component
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        out.push(Component {
            name,
            version,
            purl: purl.to_string(),
        });
    }
    Ok(out)
}
