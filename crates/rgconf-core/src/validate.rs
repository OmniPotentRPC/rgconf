// Copyright 2023--present rgconf developers

use crate::catalog::{catalog, Bounds, ParamSpec};
use crate::error::Error;
use crate::resolve::{ResolvedField, ValueStatus};
use crate::value::{ParamValue, ScalarKind};
use std::collections::BTreeMap;

pub fn validate(fields: &BTreeMap<String, ResolvedField>) -> Result<(), Error> {
    // eOn [Lineb] wording first so shared keys keep ParametersINI.cpp messages.
    check_lineb(fields)?;
    let cat = catalog()?;
    for (key, field) in fields {
        let Some(spec) = cat.get(key) else {
            continue;
        };
        check_spec(spec, field)?;
    }
    Ok(())
}

/// Numerical constraints shared by the eOn job and the LI-NEB solver.
fn check_lineb(fields: &BTreeMap<String, ResolvedField>) -> Result<(), Error> {
    if let Some(job) = fields.get("runtime.job") {
        if job.value.to_string() == "lineb"
            && fields.get("lineb.energyEv").is_none_or(|energy| {
                matches!(
                    energy.status,
                    ValueStatus::DefaultValue | ValueStatus::DerivedFromUnset
                )
            })
        {
            return Err(located(
                job,
                "job = lineb requires an explicit [Lineb] energy in eV",
            ));
        }
    }
    if !fields.keys().any(|key| key.starts_with("lineb.")) {
        return Ok(());
    }
    if let (Some(opt), Some(perp)) = (
        fields.get("lineb.optTolSqrtEv"),
        fields.get("lineb.perpTolSqrtEv"),
    ) {
        let opt_ok = as_number(&opt.value).is_some_and(|n| n.is_finite() && n >= 0.0);
        let perp_ok = as_number(&perp.value).is_some_and(|n| n.is_finite() && n >= 0.0);
        if !opt_ok || !perp_ok {
            let field = if !opt_ok { opt } else { perp };
            return Err(located(
                field,
                "[Lineb] opt_tol and perp_tol must be finite and non-negative",
            ));
        }
    }
    if let (Some(spring), Some(endpoint)) = (
        fields.get("lineb.spring"),
        fields.get("lineb.endpointScale"),
    ) {
        let spring_ok = as_number(&spring.value).is_some_and(|n| n.is_finite() && n > 0.0);
        let endpoint_ok = as_number(&endpoint.value).is_some_and(|n| n.is_finite() && n > 0.0);
        if !spring_ok || !endpoint_ok {
            let field = if !spring_ok { spring } else { endpoint };
            return Err(located(
                field,
                "[Lineb] spring and endpoint_scale must be finite and positive",
            ));
        }
    }
    if let Some(field) = fields.get("lineb.endpointKappa") {
        if !as_number(&field.value).is_some_and(|n| n.is_finite() && n >= 0.0) {
            return Err(located(
                field,
                "[Lineb] endpoint_kappa must be finite and non-negative",
            ));
        }
    }
    if let (Some(time_step), Some(max_move)) =
        (fields.get("lineb.timeStep"), fields.get("lineb.maxMove"))
    {
        let dt_ok = as_number(&time_step.value).is_some_and(|n| n.is_finite() && n > 0.0);
        let move_ok = as_number(&max_move.value).is_some_and(|n| n.is_finite() && n >= 0.0);
        if !dt_ok || !move_ok {
            let field = if !dt_ok { time_step } else { max_move };
            return Err(located(
                field,
                "[Lineb] time_step must be positive and max_move non-negative",
            ));
        }
    }
    if let Some(turning) = fields.get("lineb.turningWidth") {
        if !as_number(&turning.value).is_some_and(|n| n > 0.0 && n < 1.0) {
            return Err(located(
                turning,
                "[Lineb] turning_width must be a fraction strictly between 0 and 1",
            ));
        }
    }
    if let (Some(hbar), Some(boltzmann)) = (fields.get("lineb.hbar"), fields.get("lineb.boltzmann"))
    {
        let hbar_ok = as_number(&hbar.value).is_some_and(|n| n.is_finite() && n > 0.0);
        let kb_ok = as_number(&boltzmann.value).is_some_and(|n| n.is_finite() && n > 0.0);
        if !hbar_ok || !kb_ok {
            let field = if !hbar_ok { hbar } else { boltzmann };
            return Err(located(
                field,
                "[Lineb] hbar and boltzmann must be finite and positive",
            ));
        }
    }
    if let Some(beads) = fields.get("lineb.beads") {
        if beads.value == ParamValue::Uint(1) {
            return Err(located(
                beads,
                "[Lineb] use 0 beads for a path or at least 2 for a rate",
            ));
        }
    }
    if let Some(pairs) = fields.get("lineb.quadraturePairs") {
        if !as_number(&pairs.value).is_some_and(|n| n >= 1.0) {
            return Err(located(
                pairs,
                "[Lineb] quadrature_pairs must be at least 1",
            ));
        }
    }
    if let Some(steps) = fields.get("lineb.maxSteps") {
        if !as_number(&steps.value).is_some_and(|n| n >= 1.0) {
            return Err(located(steps, "[Lineb] max_steps must be at least 1"));
        }
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
    match origin.rsplit_once(':') {
        Some((path, line)) => (path.to_string(), line.parse().unwrap_or(fallback)),
        None => (origin.to_string(), fallback),
    }
}
