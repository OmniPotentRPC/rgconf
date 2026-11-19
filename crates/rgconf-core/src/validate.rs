// Copyright 2023--present rgconf developers

use crate::catalog::{catalog, Bounds, ParamSpec};
use crate::error::Error;
use crate::resolve::ResolvedField;
use crate::value::{ParamValue, ScalarKind};
use std::collections::BTreeMap;

pub fn validate(fields: &BTreeMap<String, ResolvedField>) -> Result<(), Error> {
    let cat = catalog()?;
    for (key, field) in fields {
        let Some(spec) = cat.get(key) else {
            continue;
        };
        check_spec(spec, field)?;
    }
    Ok(())
}

fn check_spec(spec: &ParamSpec, field: &ResolvedField) -> Result<(), Error> {
    if let ScalarKind::Enum(names) = &spec.kind {
        if !names.iter().any(|name| name == &field.value.to_string()) {
            return Err(located(
                field,
                format!("`{}` is not a member of {}", field.value, spec.type_name),
            ));
        }
    }
    if spec.kind == ScalarKind::Uint && field.value.is_negative_number() {
        return Err(located(
            field,
            format!(
                "negative value `{}` is not valid for an unsigned count",
                field.value
            ),
        ));
    }
    if let Some(number) = as_number(&field.value) {
        check_bounds(&spec.bounds, number, field)?;
    }
    Ok(())
}

fn check_bounds(bounds: &Bounds, number: f64, field: &ResolvedField) -> Result<(), Error> {
    if bounds.require_finite && !number.is_finite() {
        return Err(located(field, "value must be finite"));
    }
    if let Some(limit) = bounds.min_inclusive {
        if number < limit {
            return Err(located(
                field,
                format!("value {number} is below the inclusive minimum {limit}"),
            ));
        }
    }
    if let Some(limit) = bounds.min_exclusive {
        if number <= limit {
            return Err(located(
                field,
                format!("value {number} is not above the exclusive minimum {limit}"),
            ));
        }
    }
    if let Some(limit) = bounds.max_inclusive {
        if number > limit {
            return Err(located(
                field,
                format!("value {number} is above the inclusive maximum {limit}"),
            ));
        }
    }
    if let Some(limit) = bounds.max_exclusive {
        if number >= limit {
            return Err(located(
                field,
                format!("value {number} is not below the exclusive maximum {limit}"),
            ));
        }
    }
    Ok(())
}

fn as_number(value: &ParamValue) -> Option<f64> {
    match value {
        ParamValue::Int(number) => Some(*number as f64),
        ParamValue::Uint(number) => Some(*number as f64),
        ParamValue::Float(number) => Some(*number),
        _ => None,
    }
}

fn located(field: &ResolvedField, message: impl Into<String>) -> Error {
    let (path, line) = split_origin(&field.origin, field.line);
    Error::located(path, line, 1, &field.key, message)
}

fn split_origin(origin: &str, fallback: u32) -> (String, u32) {
    if let Some((path, tail)) = origin.rsplit_once(':') {
        if let Ok(line) = tail.parse::<u32>() {
            return (path.to_string(), line);
        }
    }
    (origin.to_string(), fallback)
}
