// Copyright 2023--present rgconf developers

use crate::catalog::{catalog, Alias, Catalog};
use crate::error::Error;
use crate::value::{parse_loose_number, ParamValue};
use serde_json::Value as Json;
use std::collections::BTreeMap;
use toml::Value as Toml;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigLayer {
    SchemaDefault,
    CompatibilityProfile,
    UserGlobalToml,
    ProjectToml,
    JobConfigFile,
    EnvironmentVar,
    CliFlag,
    LiveRuntimeEdit,
}

impl ConfigLayer {
    pub fn name(self) -> &'static str {
        match self {
            Self::SchemaDefault => "schemaDefault",
            Self::CompatibilityProfile => "compatibilityProfile",
            Self::UserGlobalToml => "userGlobalToml",
            Self::ProjectToml => "projectToml",
            Self::JobConfigFile => "jobConfigFile",
            Self::EnvironmentVar => "environmentVar",
            Self::CliFlag => "cliFlag",
            Self::LiveRuntimeEdit => "liveRuntimeEdit",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub key: String,
    pub value: ParamValue,
    pub origin: String,
    pub line: u32,
    pub column: u32,
    pub via: String,
    pub deprecated: bool,
    pub deprecation_note: String,
    pub refine_suffix: Option<String>,
}

pub fn parse_ini(text: &str, path: &str) -> Result<Vec<Assignment>, Error> {
    let cat = catalog()?;
    let mut section = String::new();
    let mut out = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let line_no = (index + 1) as u32;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed[1..trimmed.len() - 1].trim().to_string();
            continue;
        }
        let Some((raw_key, raw_value)) = trimmed.split_once('=') else {
            return Err(Error::located(
                path,
                line_no,
                1,
                trimmed,
                "expected key = value",
            ));
        };
        let key = raw_key.trim();
        let value = parse_scalar(raw_value.trim());
        push_named(cat, "ini", &section, key, value, path, line_no, &mut out)?;
    }
    Ok(out)
}

pub fn parse_toml(text: &str, path: &str) -> Result<Vec<Assignment>, Error> {
    let cat = catalog()?;
    let parsed: Toml = toml::from_str(text).map_err(|err| {
        let line = match err.span() {
            Some(span) => line_of_offset(text, span.start),
            None => 1,
        };
        Error::located(path, line, 1, "", err.to_string())
    })?;
    let mut out = Vec::new();
    walk_toml(&parsed, "", text, path, cat, &mut out)?;
    Ok(out)
}

pub fn parse_json(text: &str, path: &str) -> Result<Vec<Assignment>, Error> {
    let cat = catalog()?;
    let parsed: Json = serde_json::from_str(text).map_err(|err| {
        Error::located(
            path,
            err.line() as u32,
            err.column() as u32,
            "",
            err.to_string(),
        )
    })?;
    let mut out = Vec::new();
    walk_json(&parsed, "", text, path, cat, &mut out)?;
    Ok(out)
}

pub fn parse_env_pairs<'a, I>(pairs: I) -> Result<Vec<Assignment>, Error>
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let cat = catalog()?;
    let mut wanted: BTreeMap<String, String> = BTreeMap::new();
    for spec in &cat.params {
        wanted.insert(Catalog::env_name(&spec.key), spec.key.clone());
    }
    let mut out = Vec::new();
    for (name, raw) in pairs {
        if let Some(key) = wanted.get(name) {
            let value = parse_scalar(raw);
            let spec = cat
                .get(key)
                .ok_or_else(|| Error::message(format!("missing spec {key}")))?;
            let coerced = coerce_for(spec, &value, &format!("ENV:{name}"), 1, name)?;
            out.push(Assignment {
                key: key.clone(),
                value: coerced,
                origin: format!("ENV:{name}"),
                line: 1,
                column: 1,
                via: name.to_string(),
                deprecated: spec.deprecated,
                deprecation_note: spec.deprecation_reason.clone(),
                refine_suffix: None,
            });
        }
    }
    Ok(out)
}

pub fn parse_cli(args: &[String]) -> Result<Vec<Assignment>, Error> {
    let cat = catalog()?;
    let mut out = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--set" {
            let Some(body) = args.get(index + 1) else {
                return Err(Error::located("CLI", 1, 1, "--set", "expected key=value"));
            };
            let Some((key, raw)) = body.split_once('=') else {
                return Err(Error::located("CLI", 1, 1, "--set", "expected key=value"));
            };
            assign_canonical(cat, key, parse_scalar(raw), "CLI:--set", &mut out)?;
            index += 2;
            continue;
        }
        if arg == "--profile"
            || arg == "--config"
            || arg == "--ini"
            || arg == "--toml"
            || arg == "--repro"
            || arg == "--trusted"
            || arg == "--lock"
            || arg == "--sbom"
        {
            index += 2;
            continue;
        }
        if let Some(keys) = cat.cli_index.get(arg) {
            let (raw, consumed) = if args
                .get(index + 1)
                .is_some_and(|next| !next.starts_with("--"))
            {
                (args[index + 1].clone(), 2)
            } else {
                ("true".to_string(), 1)
            };
            let parsed = parse_scalar(&raw);
            for key in keys {
                let Some(spec) = cat.get(key) else {
                    continue;
                };
                let transformed = apply_flag_transform(spec, parsed.clone())?;
                let coerced = coerce_for(spec, &transformed, &format!("CLI:{arg}"), 1, arg)?;
                out.push(Assignment {
                    key: key.clone(),
                    value: coerced,
                    origin: format!("CLI:{arg}"),
                    line: 1,
                    column: 1,
                    via: arg.clone(),
                    deprecated: spec.deprecated,
                    deprecation_note: spec.deprecation_reason.clone(),
                    refine_suffix: None,
                });
            }
            index += consumed;
            continue;
        }
        if arg.starts_with("--") {
            return Err(Error::located("CLI", 1, 1, arg, "unknown flag"));
        }
        index += 1;
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn push_named(
    cat: &Catalog,
    format: &str,
    section: &str,
    key: &str,
    value: ParamValue,
    path: &str,
    line: u32,
    out: &mut Vec<Assignment>,
) -> Result<(), Error> {
    let origin = format!("{path}:{line}");
    if let Some(targets) =
        cat.alias_index
            .get(&(format.to_string(), section.to_string(), key.to_string()))
    {
        for target in targets {
            let spec = cat
                .get(target)
                .ok_or_else(|| Error::message(format!("missing spec {target}")))?;
            let alias = spec
                .aliases
                .iter()
                .find(|item| item.format == format && item.section == section && item.key == key);
            let transformed = match alias {
                Some(alias) => apply_named_transform(alias, value.clone(), path, line, key)?,
                None => value.clone(),
            };
            let coerced = coerce_for(spec, &transformed, &origin, line, key)?;
            let deprecated = alias.is_some_and(|item| item.deprecated) || spec.deprecated;
            let note = alias
                .map(|item| item.deprecation_note.clone())
                .filter(|note| !note.is_empty())
                .unwrap_or_else(|| spec.deprecation_reason.clone());
            out.push(Assignment {
                key: target.clone(),
                value: coerced,
                origin: origin.clone(),
                line,
                column: 1,
                via: format!("[{section}] {key}"),
                deprecated,
                deprecation_note: note,
                refine_suffix: None,
            });
        }
        return Ok(());
    }
    if let Some((_, suffix)) = cat
        .refine_aliases
        .iter()
        .find(|(alias, _)| alias.format == format && alias.section == section && alias.key == key)
    {
        out.push(Assignment {
            key: format!("refine.pending.{suffix}"),
            value,
            origin,
            line,
            column: 1,
            via: format!("[{section}] {key}"),
            deprecated: false,
            deprecation_note: String::new(),
            refine_suffix: Some(suffix.clone()),
        });
        return Ok(());
    }
    let canonical = if section.is_empty() {
        key.to_string()
    } else {
        format!("{section}.{key}")
    };
    if cat.get(&canonical).is_some() {
        assign_at(
            cat,
            &canonical,
            value,
            &origin,
            line,
            &format!("[{section}] {key}"),
            out,
        )?;
        return Ok(());
    }
    if let Some(extension) = extension_key(section, key) {
        out.push(Assignment {
            key: extension,
            value,
            origin,
            line,
            column: 1,
            via: format!("[{section}] {key}"),
            deprecated: false,
            deprecation_note: String::new(),
            refine_suffix: None,
        });
        return Ok(());
    }
    Err(Error::located(
        path,
        line,
        1,
        format!("[{section}] {key}"),
        "unknown parameter",
    ))
}

fn assign_canonical(
    cat: &Catalog,
    key: &str,
    value: ParamValue,
    origin: &str,
    out: &mut Vec<Assignment>,
) -> Result<(), Error> {
    if cat.get(key).is_some() || is_extension_key(key) {
        assign_at(cat, key, value, origin, 1, key, out)?;
        return Ok(());
    }
    Err(Error::located("CLI", 1, 1, key, "unknown parameter"))
}

fn assign_at(
    cat: &Catalog,
    key: &str,
    value: ParamValue,
    origin: &str,
    line: u32,
    via: &str,
    out: &mut Vec<Assignment>,
) -> Result<(), Error> {
    if let Some(spec) = cat.get(key) {
        let coerced = coerce_for(spec, &value, origin, line, via)?;
        out.push(Assignment {
            key: key.to_string(),
            value: coerced,
            origin: origin.to_string(),
            line,
            column: 1,
            via: via.to_string(),
            deprecated: spec.deprecated,
            deprecation_note: spec.deprecation_reason.clone(),
            refine_suffix: None,
        });
        return Ok(());
    }
    if is_extension_key(key) {
        out.push(Assignment {
            key: key.to_string(),
            value,
            origin: origin.to_string(),
            line,
            column: 1,
            via: via.to_string(),
            deprecated: false,
            deprecation_note: String::new(),
            refine_suffix: None,
        });
        return Ok(());
    }
    Err(Error::located(origin, line, 1, key, "unknown parameter"))
}

fn extension_key(section: &str, key: &str) -> Option<String> {
    if section == "ext" {
        let mut parts = key.splitn(2, '.');
        let namespace = match parts.next() {
            Some(item) => item,
            None => "",
        };
        let name = match parts.next() {
            Some(item) => item,
            None => "",
        };
        if !namespace.is_empty() && !name.is_empty() {
            return Some(format!("ext.{namespace}.{name}"));
        }
    }
    if let Some(rest) = section.strip_prefix("ext.") {
        if !rest.is_empty() && !key.is_empty() && !key.contains('.') {
            return Some(format!("ext.{rest}.{key}"));
        }
    }
    None
}

fn is_extension_key(key: &str) -> bool {
    let mut parts = key.split('.');
    parts.next() == Some("ext")
        && parts.next().is_some_and(|ns| !ns.is_empty())
        && parts.next().is_some_and(|name| !name.is_empty())
        && parts.next().is_none()
}

pub fn apply_named_transform(
    alias: &Alias,
    value: ParamValue,
    path: &str,
    line: u32,
    key: &str,
) -> Result<ParamValue, Error> {
    apply_transform(&alias.transform, value)
        .map_err(|err| Error::located(path, line, 1, key, err.to_string()))
}

fn apply_flag_transform(
    spec: &crate::catalog::ParamSpec,
    value: ParamValue,
) -> Result<ParamValue, Error> {
    let transform = spec.aliases.iter().find_map(|alias| {
        if alias.transform.is_empty() {
            None
        } else {
            Some(alias.transform.as_str())
        }
    });
    match transform {
        Some(name) => apply_transform(name, value),
        None => Ok(value),
    }
}

pub fn apply_transform(name: &str, value: ParamValue) -> Result<ParamValue, Error> {
    if name.is_empty() {
        return Ok(value);
    }
    if name == "total_from_intermediate_plus_2" {
        let base = match &value {
            ParamValue::Int(number) => *number,
            ParamValue::Uint(number) => i64::try_from(*number)
                .map_err(|_| Error::message(format!("image count {number} is too large")))?,
            ParamValue::Float(number) => {
                if !number.is_finite() {
                    return Err(Error::message("image count must be finite"));
                }
                let whole = number.round();
                if (number - whole).abs() > crate::value::VALUE_AGREEMENT_ABS_TOLERANCE {
                    return Err(Error::message(format!(
                        "image count {number} is not an integer"
                    )));
                }
                whole as i64
            }
            ParamValue::Text(text) => {
                let parsed = parse_loose_number(text)?;
                return apply_transform(name, parsed);
            }
            other => return Err(Error::message(format!("image count cannot be {other}"))),
        };
        if base < 0 {
            return Err(Error::message(format!(
                "negative value `{base}` is not valid for an unsigned count"
            )));
        }
        let count = u64::try_from(base).map_err(|_| Error::message("image count does not fit"))?;
        return Ok(ParamValue::Uint(count + 2));
    }
    Err(Error::message(format!("unknown value transform `{name}`")))
}

fn coerce_for(
    spec: &crate::catalog::ParamSpec,
    value: &ParamValue,
    origin: &str,
    line: u32,
    via: &str,
) -> Result<ParamValue, Error> {
    if matches!(spec.kind, crate::value::ScalarKind::Uint) && value.is_negative_number() {
        return Err(Error::located(
            origin_path(origin),
            line,
            1,
            via,
            format!("negative value `{value}` is not valid for an unsigned count"),
        ));
    }
    spec.kind
        .coerce(value)
        .map_err(|err| Error::located(origin_path(origin), line, 1, via, err.to_string()))
}

fn origin_path(origin: &str) -> &str {
    match origin.rsplit_once(':') {
        Some((path, tail)) if tail.chars().all(|ch| ch.is_ascii_digit()) => path,
        _ => origin,
    }
}

fn parse_scalar(raw: &str) -> ParamValue {
    let trimmed = raw.trim().trim_matches('"').trim_matches('\'');
    match trimmed.to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" => return ParamValue::Bool(true),
        "false" | "no" | "off" => return ParamValue::Bool(false),
        _ => {}
    }
    if let Ok(value) = parse_loose_number(trimmed) {
        return value;
    }
    ParamValue::Text(trimmed.to_string())
}

fn walk_toml(
    value: &Toml,
    section: &str,
    text: &str,
    path: &str,
    cat: &Catalog,
    out: &mut Vec<Assignment>,
) -> Result<(), Error> {
    let Toml::Table(table) = value else {
        return Ok(());
    };
    for (key, child) in table {
        match child {
            Toml::Table(_) => {
                let next = if section.is_empty() {
                    key.clone()
                } else {
                    format!("{section}.{key}")
                };
                walk_toml(child, &next, text, path, cat, out)?;
            }
            _ => {
                let parsed = toml_scalar(child);
                let line = line_of_key(text, key);
                push_named(cat, "toml", section, key, parsed, path, line, out)?;
            }
        }
    }
    Ok(())
}

fn walk_json(
    value: &Json,
    section: &str,
    text: &str,
    path: &str,
    cat: &Catalog,
    out: &mut Vec<Assignment>,
) -> Result<(), Error> {
    let Json::Object(map) = value else {
        return Ok(());
    };
    for (key, child) in map {
        if child.is_object() {
            let next = if section.is_empty() {
                key.clone()
            } else {
                format!("{section}.{key}")
            };
            walk_json(child, &next, text, path, cat, out)?;
        } else {
            push_named(
                cat,
                "json",
                section,
                key,
                json_scalar(child),
                path,
                line_of_key(text, key),
                out,
            )?;
        }
    }
    Ok(())
}

fn toml_scalar(value: &Toml) -> ParamValue {
    match value {
        Toml::Boolean(bit) => ParamValue::Bool(*bit),
        Toml::Integer(number) => ParamValue::Int(*number),
        Toml::Float(number) => ParamValue::Float(*number),
        Toml::String(text) => parse_scalar(text),
        other => ParamValue::Text(other.to_string()),
    }
}

fn json_scalar(value: &Json) -> ParamValue {
    match value {
        Json::Bool(bit) => ParamValue::Bool(*bit),
        Json::Number(number) => {
            if let Some(integer) = number.as_i64() {
                ParamValue::Int(integer)
            } else if let Some(float) = number.as_f64() {
                ParamValue::Float(float)
            } else {
                ParamValue::Text(number.to_string())
            }
        }
        Json::String(text) => parse_scalar(text),
        other => ParamValue::Text(other.to_string()),
    }
}

fn line_of_key(text: &str, key: &str) -> u32 {
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(key) {
            let rest = rest.trim_start();
            if rest.starts_with('=') || rest.starts_with(':') {
                return (index + 1) as u32;
            }
        }
    }
    1
}

fn line_of_offset(text: &str, offset: usize) -> u32 {
    text[..offset.min(text.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count() as u32
        + 1
}
