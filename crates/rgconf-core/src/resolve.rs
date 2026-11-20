// Copyright 2023--present rgconf developers

use crate::catalog::{catalog, Mutability, ParamSpec};
use crate::edit::ParamEdit;
use crate::error::Error;
use crate::input::{self, Assignment, ConfigLayer};
use crate::units::gpridge_paper_force_ev_per_angstrom;
use crate::validate;
use crate::value::ParamValue;
use std::collections::BTreeMap;

const EON_DEFAULT_FORCE_EV_PER_ANGSTROM: f64 = 0.01;
const GPRIDGE_LEGACY_FORCE_EV_PER_ANGSTROM: f64 = 0.05;
const ASE_DROPIN_FORCE_EV_PER_ANGSTROM: f64 = 0.05;
const EW_K_MIN_FRACTION_OF_SPRING: f64 = 0.1;
const ZOOM_FORMULA_PASSES: usize = 8;
/// eOn `lineb_options_t.opt_tol` / `perp_tol` (feat/fixed-energy-lineb 904ad4f27).
const LINEB_EON_FORCE_TOL_SQRT_EV: f64 = 1e-3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    RgconfCanonical,
    EonLegacyIni,
    EonServerYaml,
    GpridgePaper,
    GpridgeLegacyToml,
    AseDropin,
}

impl Profile {
    pub fn parse(text: &str) -> Result<Self, Error> {
        match text {
            "rgconfCanonical" | "canonical" => Ok(Self::RgconfCanonical),
            "eonLegacyIni" | "eon-legacy-ini" => Ok(Self::EonLegacyIni),
            "eonServerYaml" | "eon-server-yaml" => Ok(Self::EonServerYaml),
            "gpridgePaper" | "gpridge-paper" => Ok(Self::GpridgePaper),
            "gpridgeLegacyToml" | "gpridge-legacy-toml" => Ok(Self::GpridgeLegacyToml),
            "aseDropin" | "ase-dropin" => Ok(Self::AseDropin),
            other => Err(Error::message(format!(
                "unknown compatibility profile `{other}`"
            ))),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::RgconfCanonical => "rgconfCanonical",
            Self::EonLegacyIni => "eonLegacyIni",
            Self::EonServerYaml => "eonServerYaml",
            Self::GpridgePaper => "gpridgePaper",
            Self::GpridgeLegacyToml => "gpridgeLegacyToml",
            Self::AseDropin => "aseDropin",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueStatus {
    DefaultValue,
    ProfileDefault,
    DerivedFromUnset,
    ExplicitlySet,
    LayeredOverride,
    DeprecatedAlias,
    MutatedMidRun,
}

impl ValueStatus {
    pub fn name(self) -> &'static str {
        match self {
            Self::DefaultValue => "defaultValue",
            Self::ProfileDefault => "profileDefault",
            Self::DerivedFromUnset => "derivedFromUnset",
            Self::ExplicitlySet => "explicitlySet",
            Self::LayeredOverride => "layeredOverride",
            Self::DeprecatedAlias => "deprecatedAlias",
            Self::MutatedMidRun => "mutatedMidRun",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedField {
    pub key: String,
    pub value: ParamValue,
    pub status: ValueStatus,
    pub layer: ConfigLayer,
    pub origin: String,
    pub line: u32,
    pub via: String,
    pub history: Vec<String>,
    pub doc: String,
    pub unit: String,
    pub mutability: Mutability,
    pub deprecated: bool,
    pub deprecation_note: String,
    pub type_name: String,
}

#[derive(Debug, Clone)]
pub struct Resolution {
    pub profile: Profile,
    pub fields: BTreeMap<String, ResolvedField>,
    pub edits: Vec<ParamEdit>,
}

impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "profile {}", self.profile.name())?;
        for (key, field) in &self.fields {
            write!(f, "\n{key}={} {}", field.value, field.status.name())?;
        }
        Ok(())
    }
}

impl Resolution {
    pub fn get(&self, key: &str) -> Option<&ResolvedField> {
        self.fields.get(key)
    }

    pub fn require(&self, key: &str) -> Result<&ResolvedField, Error> {
        self.fields
            .get(key)
            .ok_or_else(|| Error::message(format!("unknown parameter `{key}`")))
    }
}

#[derive(Debug, Clone)]
pub struct Resolver {
    profile: Profile,
    layers: BTreeMap<ConfigLayer, Vec<Assignment>>,
}

impl Resolver {
    pub fn new(profile: Profile) -> Self {
        Self {
            profile,
            layers: BTreeMap::new(),
        }
    }

    pub fn profile(&self) -> Profile {
        self.profile
    }

    pub fn merge_ini(&mut self, layer: ConfigLayer, text: &str, path: &str) -> Result<(), Error> {
        self.absorb(layer, input::parse_ini(text, path)?)
    }

    pub fn merge_toml(&mut self, layer: ConfigLayer, text: &str, path: &str) -> Result<(), Error> {
        self.absorb(layer, input::parse_toml(text, path)?)
    }

    pub fn merge_json(&mut self, layer: ConfigLayer, text: &str, path: &str) -> Result<(), Error> {
        self.absorb(layer, input::parse_json(text, path)?)
    }

    pub fn merge_env<'a, I>(&mut self, pairs: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        self.absorb(ConfigLayer::EnvironmentVar, input::parse_env_pairs(pairs)?)
    }

    pub fn merge_args(&mut self, args: &[String]) -> Result<(), Error> {
        self.absorb(ConfigLayer::CliFlag, input::parse_cli(args)?)
    }

    pub fn merge_assignments(
        &mut self,
        layer: ConfigLayer,
        hits: Vec<Assignment>,
    ) -> Result<(), Error> {
        self.absorb(layer, hits)
    }

    fn absorb(&mut self, layer: ConfigLayer, incoming: Vec<Assignment>) -> Result<(), Error> {
        let bucket = self.layers.entry(layer).or_default();
        for item in incoming {
            if let Some(previous) = bucket.iter().find(|have| have.key == item.key) {
                if !previous.value.agrees(&item.value) {
                    return Err(Error::message(format!(
                        "contradictory values for `{}`: `{}` from {} and `{}` from {}",
                        item.key, previous.value, previous.via, item.value, item.via
                    )));
                }
            }
            bucket.push(item);
        }
        Ok(())
    }

    pub fn apply_assignment(resolution: &mut Resolution, hit: &Assignment) -> Result<(), Error> {
        let backup = resolution.fields.get(&hit.key).cloned();
        apply_user(&mut resolution.fields, ConfigLayer::LiveRuntimeEdit, hit)?;
        if let Err(err) = validate::validate(&resolution.fields) {
            if let Some(previous) = backup {
                resolution.fields.insert(hit.key.clone(), previous);
            }
            return Err(err);
        }
        Ok(())
    }

    pub fn resolve(&self) -> Result<Resolution, Error> {
        let cat = catalog()?;
        let mut fields = BTreeMap::new();
        for spec in &cat.params {
            fields.insert(spec.key.clone(), field_from_spec(spec));
        }
        for (layer, hits) in &self.layers {
            for hit in hits {
                if hit.refine_suffix.is_some() {
                    continue;
                }
                apply_user(&mut fields, *layer, hit)?;
            }
        }
        apply_refine(&mut fields, &self.layers, self.profile)?;
        apply_formulas(&mut fields, self.profile)?;
        validate::validate(&fields)?;
        Ok(Resolution {
            profile: self.profile,
            fields,
            edits: Vec::new(),
        })
    }
}

fn field_from_spec(spec: &ParamSpec) -> ResolvedField {
    ResolvedField {
        key: spec.key.clone(),
        value: spec.default_value.clone(),
        status: ValueStatus::DefaultValue,
        layer: ConfigLayer::SchemaDefault,
        origin: format!("schema:{}", spec.source_line),
        line: spec.source_line,
        via: "schema default".to_string(),
        history: vec![format!("schemaDefault schema {}", spec.default_value)],
        doc: spec.doc.clone(),
        unit: spec.unit.clone(),
        mutability: spec.mutability,
        deprecated: spec.deprecated,
        deprecation_note: spec.deprecation_reason.clone(),
        type_name: spec.type_name.clone(),
    }
}

fn apply_user(
    fields: &mut BTreeMap<String, ResolvedField>,
    layer: ConfigLayer,
    hit: &Assignment,
) -> Result<(), Error> {
    if !fields.contains_key(&hit.key) {
        if !hit.key.starts_with("ext.") {
            return Err(Error::message(format!("unknown parameter `{}`", hit.key)));
        }
        fields.insert(hit.key.clone(), extension_field(hit));
    }
    let field = fields
        .get_mut(&hit.key)
        .ok_or_else(|| Error::message(format!("missing field {}", hit.key)))?;
    if field.layer == layer && is_user(field.status) && layer != ConfigLayer::LiveRuntimeEdit {
        if !field.value.agrees(&hit.value) {
            return Err(Error::message(format!(
                "contradictory values for `{}`: `{}` from {} and `{}` from {}",
                hit.key, field.value, field.via, hit.value, hit.via
            )));
        }
        field
            .history
            .push(format!("{} {} {}", layer.name(), hit.via, hit.value));
        field.via = format!("{}, {}", field.via, hit.via);
        if hit.deprecated {
            field.status = ValueStatus::DeprecatedAlias;
            field.deprecated = true;
            field.deprecation_note = hit.deprecation_note.clone();
        }
        return Ok(());
    }
    let had_user = is_user(field.status);
    field.value = hit.value.clone();
    field.layer = layer;
    field.origin = hit.origin.clone();
    field.line = hit.line;
    field.via = hit.via.clone();
    field.status = if hit.deprecated {
        ValueStatus::DeprecatedAlias
    } else if had_user {
        ValueStatus::LayeredOverride
    } else {
        ValueStatus::ExplicitlySet
    };
    if hit.deprecated {
        field.deprecated = true;
        if !hit.deprecation_note.is_empty() {
            field.deprecation_note = hit.deprecation_note.clone();
        }
    }
    field
        .history
        .push(format!("{} {} {}", layer.name(), hit.via, hit.value));
    Ok(())
}

fn extension_field(hit: &Assignment) -> ResolvedField {
    ResolvedField {
        key: hit.key.clone(),
        value: hit.value.clone(),
        status: ValueStatus::DefaultValue,
        layer: ConfigLayer::SchemaDefault,
        origin: hit.origin.clone(),
        line: hit.line,
        via: hit.via.clone(),
        history: Vec::new(),
        doc: "third party extension".to_string(),
        unit: String::new(),
        mutability: Mutability::MutableLive,
        deprecated: false,
        deprecation_note: String::new(),
        type_name: "extension".to_string(),
    }
}

fn apply_refine(
    fields: &mut BTreeMap<String, ResolvedField>,
    layers: &BTreeMap<ConfigLayer, Vec<Assignment>>,
    profile: Profile,
) -> Result<(), Error> {
    let job = effective_job(fields, profile);
    let prefix = handoff_prefix(&job);
    for (layer, hits) in layers {
        for hit in hits {
            let Some(suffix) = &hit.refine_suffix else {
                continue;
            };
            if suffix == "secondaryMethod" && hit.value.to_string() == "none" {
                let mut disabled = hit.clone();
                disabled.key = format!("{prefix}.enabled");
                disabled.value = ParamValue::Bool(false);
                disabled.refine_suffix = None;
                apply_user(fields, *layer, &disabled)?;
                continue;
            }
            let mut routed = hit.clone();
            routed.key = format!("{prefix}.{suffix}");
            routed.refine_suffix = None;
            apply_user(fields, *layer, &routed)?;
            if suffix == "secondaryMethod" {
                let enabled_key = format!("{prefix}.enabled");
                if fields
                    .get(&enabled_key)
                    .is_some_and(|field| !is_user(field.status))
                {
                    let mut enabled = hit.clone();
                    enabled.key = enabled_key;
                    enabled.value = ParamValue::Bool(true);
                    enabled.refine_suffix = None;
                    enabled.via = format!("{} (enables handoff)", hit.via);
                    apply_user(fields, *layer, &enabled)?;
                }
            }
        }
    }
    Ok(())
}

fn apply_formulas(
    fields: &mut BTreeMap<String, ResolvedField>,
    profile: Profile,
) -> Result<(), Error> {
    let cat = catalog()?;
    let keys: Vec<String> = fields.keys().cloned().collect();
    for _ in 0..ZOOM_FORMULA_PASSES {
        let mut changed = false;
        for key in &keys {
            let user = fields.get(key).is_some_and(|field| is_user(field.status));
            if user {
                continue;
            }
            let function = cat
                .get(key)
                .map(|spec| spec.resolver_function.clone())
                .unwrap_or_else(String::new);
            if let Some((value, status, origin)) = formula(key, &function, fields, profile) {
                let field = fields
                    .get_mut(key)
                    .ok_or_else(|| Error::message(format!("missing {key}")))?;
                if field.value.agrees(&value) && field.status == status && field.origin == origin {
                    continue;
                }
                field.value = value;
                field.status = status;
                field.layer = if status == ValueStatus::ProfileDefault {
                    ConfigLayer::CompatibilityProfile
                } else {
                    ConfigLayer::SchemaDefault
                };
                field.origin = origin.clone();
                field.via = origin.clone();
                field
                    .history
                    .push(format!("{} {} {}", field.layer.name(), origin, field.value));
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    Ok(())
}

fn formula(
    key: &str,
    function: &str,
    fields: &BTreeMap<String, ResolvedField>,
    profile: Profile,
) -> Option<(ParamValue, ValueStatus, String)> {
    match function {
        "resolve_force_norm_from_profile" => Some((
            ParamValue::Text(profile_norm(profile, key).to_string()),
            ValueStatus::ProfileDefault,
            format!("profile:{}", profile.name()),
        )),
        "resolve_force_tolerance_from_profile" => profile_force(profile, key, fields),
        "resolve_band_image_count_from_profile" if key == "band.imageCount" => {
            let count = match profile {
                Profile::GpridgePaper | Profile::GpridgeLegacyToml => 10,
                _ => 5,
            };
            Some((
                ParamValue::Uint(count),
                ValueStatus::ProfileDefault,
                format!("profile:{}", profile.name()),
            ))
        }
        "resolve_job_kind_from_profile" => {
            let job = if profile == Profile::EonServerYaml {
                "akmcProcessSearch"
            } else {
                "minimization"
            };
            Some((
                ParamValue::Text(job.to_string()),
                ValueStatus::ProfileDefault,
                format!("profile:{}", profile.name()),
            ))
        }
        "resolve_gp_model_profile" => {
            let model = if profile == Profile::GpridgeLegacyToml {
                "extended"
            } else {
                "gprdZbl"
            };
            Some((
                ParamValue::Text(model.to_string()),
                ValueStatus::ProfileDefault,
                format!("profile:{}", profile.name()),
            ))
        }
        "resolve_zoom_trigger_from_band_tolerance" => {
            let force = number_of(fields, "band.convergence.forceToleranceEvPerAngstrom")?;
            let multiplier = number_of(fields, "band.zoom.triggerForceMultiplier")?;
            Some((
                ParamValue::Float(multiplier * force),
                ValueStatus::DerivedFromUnset,
                "derived:resolve_zoom_trigger_from_band_tolerance".to_string(),
            ))
        }
        "resolve_ew_k_min_or_tenth_ceiling" => {
            let sibling = sibling_key(key, "ewKMinEvPerAngstrom2", "constantEvPerAngstrom2")?;
            let constant = number_of(fields, &sibling)?;
            Some((
                ParamValue::Float(EW_K_MIN_FRACTION_OF_SPRING * constant),
                ValueStatus::DerivedFromUnset,
                "derived:resolve_ew_k_min_or_tenth_ceiling".to_string(),
            ))
        }
        // eOn lineb_options_t opt_tol / perp_tol defaults (ParametersOptions.h).
        "resolve_lineb_opt_tol_eon_default" => Some((
            ParamValue::Float(LINEB_EON_FORCE_TOL_SQRT_EV),
            ValueStatus::DerivedFromUnset,
            "derived:eon.lineb_options_t.opt_tol".to_string(),
        )),
        "resolve_lineb_perp_tol_eon_default" => Some((
            ParamValue::Float(LINEB_EON_FORCE_TOL_SQRT_EV),
            ValueStatus::DerivedFromUnset,
            "derived:eon.lineb_options_t.perp_tol".to_string(),
        )),
        _ if key == "potential.arm" => {
            let arm = if profile == Profile::EonServerYaml {
                "external"
            } else {
                "lj"
            };
            Some((
                ParamValue::Text(arm.to_string()),
                ValueStatus::ProfileDefault,
                format!("profile:{}", profile.name()),
            ))
        }
        _ => None,
    }
}

fn profile_force(
    profile: Profile,
    key: &str,
    fields: &BTreeMap<String, ResolvedField>,
) -> Option<(ParamValue, ValueStatus, String)> {
    if !key.ends_with("forceToleranceEvPerAngstrom")
        && !key.ends_with("forceThresholdEvPerAngstrom")
    {
        return None;
    }
    if key.starts_with("band.") && profile == Profile::EonLegacyIni {
        let inherited = fields
            .get("optimizer.convergence.forceToleranceEvPerAngstrom")?
            .value
            .clone();
        return Some((
            inherited,
            ValueStatus::DerivedFromUnset,
            "derived:optimizer.convergence.forceToleranceEvPerAngstrom".to_string(),
        ));
    }
    let value = match profile {
        Profile::GpridgePaper if key.starts_with("band.") => gpridge_paper_force_ev_per_angstrom(),
        Profile::GpridgeLegacyToml if key.starts_with("band.") => {
            GPRIDGE_LEGACY_FORCE_EV_PER_ANGSTROM
        }
        Profile::AseDropin if key.starts_with("band.") => ASE_DROPIN_FORCE_EV_PER_ANGSTROM,
        _ => EON_DEFAULT_FORCE_EV_PER_ANGSTROM,
    };
    Some((
        ParamValue::Float(value),
        ValueStatus::ProfileDefault,
        format!("profile:{}", profile.name()),
    ))
}

fn profile_norm(profile: Profile, key: &str) -> &'static str {
    let optimizer = key.starts_with("optimizer.");
    match profile {
        Profile::EonLegacyIni | Profile::EonServerYaml if optimizer => "l2Euclidean",
        _ => "maxAtomL2",
    }
}

fn effective_job(fields: &BTreeMap<String, ResolvedField>, profile: Profile) -> String {
    if let Some(field) = fields.get("runtime.job") {
        if is_user(field.status) {
            return field.value.to_string();
        }
    }
    if profile == Profile::EonServerYaml {
        "akmcProcessSearch".to_string()
    } else {
        "minimization".to_string()
    }
}

pub fn handoff_prefix(job: &str) -> &'static str {
    match job {
        "nudgedElasticBand" => "band.refine.handoff",
        "saddleSearch" => "saddleSearch.refine.handoff",
        _ => "optimizer.handoff",
    }
}

fn number_of(fields: &BTreeMap<String, ResolvedField>, key: &str) -> Option<f64> {
    match fields.get(key)?.value {
        ParamValue::Float(number) => Some(number),
        ParamValue::Int(number) => Some(number as f64),
        ParamValue::Uint(number) => Some(number as f64),
        _ => None,
    }
}

fn sibling_key(key: &str, from: &str, to: &str) -> Option<String> {
    let (head, tail) = key.rsplit_once('.')?;
    if tail != from {
        return None;
    }
    Some(format!("{head}.{to}"))
}

pub fn is_user(status: ValueStatus) -> bool {
    matches!(
        status,
        ValueStatus::ExplicitlySet
            | ValueStatus::LayeredOverride
            | ValueStatus::DeprecatedAlias
            | ValueStatus::MutatedMidRun
    )
}
