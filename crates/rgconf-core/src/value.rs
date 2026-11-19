// Copyright 2023--present rgconf developers

use crate::error::Error;
use std::fmt;

/// Absolute gap below which two finite parameter numbers agree.
pub const VALUE_AGREEMENT_ABS_TOLERANCE: f64 = 0.000_000_000_001;

/// A typed parameter value. `Unset` is the absence of a user assignment.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    Unset,
    Bool(bool),
    Int(i64),
    Uint(u64),
    Float(f64),
    Text(String),
    IntList(Vec<i64>),
    FloatList(Vec<f64>),
    TextList(Vec<String>),
}

impl ParamValue {
    pub fn agrees(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Float(left), Self::Float(right)) => {
                left.is_finite()
                    && right.is_finite()
                    && (left - right).abs() <= VALUE_AGREEMENT_ABS_TOLERANCE
            }
            (Self::Float(left), Self::Int(right)) => finite_agrees(*left, *right as f64),
            (Self::Int(left), Self::Float(right)) => finite_agrees(*left as f64, *right),
            (Self::Float(left), Self::Uint(right)) => finite_agrees(*left, *right as f64),
            (Self::Uint(left), Self::Float(right)) => finite_agrees(*left as f64, *right),
            (Self::Int(left), Self::Uint(right)) => *left >= 0 && *left as u64 == *right,
            (Self::Uint(left), Self::Int(right)) => *right >= 0 && *left == *right as u64,
            _ => self == other,
        }
    }

    pub fn is_negative_number(&self) -> bool {
        match self {
            Self::Int(value) => *value < 0,
            Self::Float(value) => value.is_sign_negative() && *value != 0.0,
            _ => false,
        }
    }
}

fn finite_agrees(left: f64, right: f64) -> bool {
    left.is_finite() && right.is_finite() && (left - right).abs() <= VALUE_AGREEMENT_ABS_TOLERANCE
}

impl fmt::Display for ParamValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unset => write!(f, "unset"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::Int(value) => write!(f, "{value}"),
            Self::Uint(value) => write!(f, "{value}"),
            Self::Float(value) => write!(f, "{value}"),
            Self::Text(value) => write!(f, "{value}"),
            Self::IntList(values) => write!(f, "{values:?}"),
            Self::FloatList(values) => write!(f, "{values:?}"),
            Self::TextList(values) => write!(f, "{values:?}"),
        }
    }
}

/// Scalar kinds the catalog attaches to a canonical key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalarKind {
    Bool,
    Int,
    Uint,
    Float,
    Text,
    Enum(Vec<String>),
    IntList,
    FloatList,
    TextList,
}

impl ScalarKind {
    pub fn coerce(&self, value: &ParamValue) -> Result<ParamValue, Error> {
        match self {
            Self::Bool => coerce_bool(value),
            Self::Int => coerce_int(value),
            Self::Uint => coerce_uint(value),
            Self::Float => coerce_float(value),
            Self::Text => match value {
                ParamValue::Text(text) => Ok(ParamValue::Text(text.clone())),
                other => Ok(ParamValue::Text(other.to_string())),
            },
            Self::Enum(names) => coerce_enum(names, value),
            Self::IntList | Self::FloatList | Self::TextList => Ok(value.clone()),
        }
    }
}

fn coerce_bool(value: &ParamValue) -> Result<ParamValue, Error> {
    match value {
        ParamValue::Bool(bit) => Ok(ParamValue::Bool(*bit)),
        ParamValue::Int(0) => Ok(ParamValue::Bool(false)),
        ParamValue::Int(1) => Ok(ParamValue::Bool(true)),
        ParamValue::Text(text) => match text.trim().to_ascii_lowercase().as_str() {
            "true" | "yes" | "on" => Ok(ParamValue::Bool(true)),
            "false" | "no" | "off" => Ok(ParamValue::Bool(false)),
            _ => Err(Error::message(format!(
                "expected a boolean, found `{text}`"
            ))),
        },
        other => Err(Error::message(format!("expected a boolean, found {other}"))),
    }
}

fn coerce_int(value: &ParamValue) -> Result<ParamValue, Error> {
    match value {
        ParamValue::Int(number) => Ok(ParamValue::Int(*number)),
        ParamValue::Uint(number) => i64::try_from(*number).map(ParamValue::Int).map_err(|_| {
            Error::message(format!(
                "unsigned value {number} does not fit a signed parameter"
            ))
        }),
        ParamValue::Float(number) => integer_float(*number).map(ParamValue::Int),
        ParamValue::Text(text) => parse_loose_number(text).and_then(|parsed| coerce_int(&parsed)),
        other => Err(Error::message(format!(
            "expected an integer, found {other}"
        ))),
    }
}

fn coerce_uint(value: &ParamValue) -> Result<ParamValue, Error> {
    if value.is_negative_number() {
        return Err(Error::message(format!(
            "negative value `{value}` is not valid for an unsigned count"
        )));
    }
    match value {
        ParamValue::Uint(number) => Ok(ParamValue::Uint(*number)),
        ParamValue::Int(number) => u64::try_from(*number)
            .map(ParamValue::Uint)
            .map_err(|_| Error::message(format!("value {number} is not an unsigned count"))),
        ParamValue::Float(number) => {
            let whole = integer_float(*number)?;
            if whole < 0 {
                return Err(Error::message(format!(
                    "negative value `{value}` is not valid for an unsigned count"
                )));
            }
            Ok(ParamValue::Uint(whole as u64))
        }
        ParamValue::Text(text) => parse_loose_number(text).and_then(|parsed| coerce_uint(&parsed)),
        other => Err(Error::message(format!(
            "expected an unsigned count, found {other}"
        ))),
    }
}

fn coerce_float(value: &ParamValue) -> Result<ParamValue, Error> {
    match value {
        ParamValue::Float(number) => Ok(ParamValue::Float(*number)),
        ParamValue::Int(number) => Ok(ParamValue::Float(*number as f64)),
        ParamValue::Uint(number) => Ok(ParamValue::Float(*number as f64)),
        ParamValue::Text(text) => parse_loose_number(text).and_then(|parsed| coerce_float(&parsed)),
        other => Err(Error::message(format!("expected a number, found {other}"))),
    }
}

fn integer_float(number: f64) -> Result<i64, Error> {
    if !number.is_finite() {
        return Err(Error::message(format!("non finite number `{number}`")));
    }
    let whole = number.round();
    if (number - whole).abs() > VALUE_AGREEMENT_ABS_TOLERANCE {
        return Err(Error::message(format!(
            "expected an integer number, found {number}"
        )));
    }
    if whole < i64::MIN as f64 || whole > i64::MAX as f64 {
        return Err(Error::message(format!(
            "number {number} does not fit an integer parameter"
        )));
    }
    Ok(whole as i64)
}

pub fn parse_loose_number(text: &str) -> Result<ParamValue, Error> {
    let trimmed = text.trim();
    if trimmed.eq_ignore_ascii_case("inf")
        || trimmed.eq_ignore_ascii_case("+inf")
        || trimmed.eq_ignore_ascii_case("infinity")
    {
        return Ok(ParamValue::Float(f64::INFINITY));
    }
    if let Ok(integer) = trimmed.parse::<i64>() {
        return Ok(ParamValue::Int(integer));
    }
    if let Ok(integer) = trimmed.parse::<u64>() {
        return Ok(ParamValue::Uint(integer));
    }
    match trimmed.parse::<f64>() {
        Ok(number) => Ok(ParamValue::Float(number)),
        Err(_) => Err(Error::message(format!(
            "expected a number, found `{trimmed}`"
        ))),
    }
}

fn coerce_enum(names: &[String], value: &ParamValue) -> Result<ParamValue, Error> {
    let raw = match value {
        ParamValue::Text(text) => text.clone(),
        other => other.to_string(),
    };
    if let Some(name) = lookup_enum(names, &raw) {
        return Ok(ParamValue::Text(name));
    }
    Err(Error::message(format!(
        "value `{raw}` is not one of {}",
        names.join(", ")
    )))
}

pub fn lookup_enum(names: &[String], raw: &str) -> Option<String> {
    let folded = raw.trim().to_ascii_lowercase().replace('-', "_");
    if let Some(found) = names
        .iter()
        .find(|name| name.eq_ignore_ascii_case(raw.trim()))
    {
        return Some(found.clone());
    }
    let legacy = legacy_enum(&folded);
    names
        .iter()
        .find(|name| name.eq_ignore_ascii_case(legacy))
        .cloned()
}

fn legacy_enum(folded: &str) -> &str {
    match folded {
        "norm" => "l2Euclidean",
        "max_atom" | "max_atom_l2" | "fmax" => "maxAtomL2",
        "max_component" | "inf" | "linf" => "maxComponentLinf",
        "rms" => "rmsAtom",
        "nudged_elastic_band" | "neb" => "nudgedElasticBand",
        "saddle_search" | "saddle" => "saddleSearch",
        "akmc" => "akmcProcessSearch",
        "molecular_dynamics" | "dynamics" | "md" => "molecularDynamics",
        "basin_hopping" => "basinHopping",
        "steepest_descent" | "sd" => "steepestDescent",
        "quickmin" | "qm" => "quickMin",
        "fire2" => "fire2",
        "lbfgs" | "cg" | "fire" | "none" => folded,
        "minimization" => "minimization",
        other => other,
    }
}
