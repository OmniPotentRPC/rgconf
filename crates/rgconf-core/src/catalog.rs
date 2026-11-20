// Copyright 2023--present rgconf developers

use crate::error::Error;
use crate::value::{parse_loose_number, ParamValue, ScalarKind};
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

const SCHEMA_FILES: &[(&str, &str)] = &[
    ("common.capnp", include_str!("../schema/common.capnp")),
    (
        "convergence.capnp",
        include_str!("../schema/convergence.capnp"),
    ),
    (
        "optimizer.capnp",
        include_str!("../schema/optimizer.capnp"),
    ),
    ("refine.capnp", include_str!("../schema/refine.capnp")),
    ("band.capnp", include_str!("../schema/band.capnp")),
    (
        "minmode.capnp",
        include_str!("../schema/minmode.capnp"),
    ),
    (
        "surrogate.capnp",
        include_str!("../schema/surrogate.capnp"),
    ),
    (
        "potential.capnp",
        include_str!("../schema/potential.capnp"),
    ),
    (
        "dynamics.capnp",
        include_str!("../schema/dynamics.capnp"),
    ),
    (
        "provenance.capnp",
        include_str!("../schema/provenance.capnp"),
    ),
    ("rgconf.capnp", include_str!("../schema/rgconf.capnp")),
    (
        "gpridge.capnp",
        include_str!("../schema/gpridge.capnp"),
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    MutableLive,
    BetweenStepsWithHook,
    RestartOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultKind {
    Literal,
    Profile,
    Derived,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bounds {
    pub min_inclusive: Option<f64>,
    pub min_exclusive: Option<f64>,
    pub max_inclusive: Option<f64>,
    pub max_exclusive: Option<f64>,
    pub require_finite: bool,
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            min_inclusive: None,
            min_exclusive: None,
            max_inclusive: None,
            max_exclusive: None,
            require_finite: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alias {
    pub format: String,
    pub section: String,
    pub key: String,
    pub transform: String,
    pub deprecated: bool,
    pub deprecation_note: String,
}

#[derive(Debug, Clone)]
pub struct ParamSpec {
    pub key: String,
    pub kind: ScalarKind,
    pub default_value: ParamValue,
    pub unit: String,
    pub dimension: String,
    pub mutability: Mutability,
    pub bounds: Bounds,
    pub default_kind: DefaultKind,
    pub resolver_function: String,
    pub deprecated: bool,
    pub deprecation_reason: String,
    pub doc: String,
    pub cli_flags: Vec<String>,
    pub aliases: Vec<Alias>,
    pub source_line: u32,
    pub type_name: String,
}

#[derive(Debug, Clone)]
struct RawField {
    name: String,
    type_name: String,
    default_text: Option<String>,
    mutability: Option<String>,
    meta: BTreeMap<String, MetaValue>,
    doc: String,
    union_arm: bool,
    source_line: u32,
}

#[derive(Debug, Clone)]
enum MetaValue {
    Word(String),
    List(Vec<MetaValue>),
    Group(BTreeMap<String, MetaValue>),
}

#[derive(Debug, Clone)]
struct StructDef {
    fields: Vec<RawField>,
}

#[derive(Debug, Clone)]
struct SchemaModel {
    structs: HashMap<String, StructDef>,
    enums: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Catalog {
    pub params: Vec<ParamSpec>,
    pub by_key: HashMap<String, usize>,
    /// `(format, section, key)` -> canonical keys.
    pub alias_index: HashMap<(String, String, String), Vec<String>>,
    /// Refine aliases, routed to the active job's handoff owner.
    pub refine_aliases: Vec<(Alias, String)>,
    pub cli_index: HashMap<String, Vec<String>>,
}

impl Catalog {
    pub fn get(&self, key: &str) -> Option<&ParamSpec> {
        self.by_key.get(key).map(|index| &self.params[*index])
    }

    pub fn env_name(canonical: &str) -> String {
        let chars: Vec<char> = canonical.chars().collect();
        let mut out = String::from("RGCONF");
        for (index, symbol) in chars.iter().enumerate() {
            if *symbol == '.' {
                out.push('_');
                continue;
            }
            if symbol.is_uppercase() {
                let previous = index.checked_sub(1).and_then(|at| chars.get(at)).copied();
                if previous.is_some_and(|item| item.is_ascii_lowercase() || item.is_ascii_digit()) {
                    out.push('_');
                }
            }
            out.push(symbol.to_ascii_uppercase());
        }
        out
    }
}

pub fn catalog() -> Result<&'static Catalog, Error> {
    static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();
    match CATALOG.get_or_init(|| build_catalog().map_err(|err| err.to_string())) {
        Ok(catalog) => Ok(catalog),
        Err(err) => Err(Error::message(err.clone())),
    }
}

fn build_catalog() -> Result<Catalog, Error> {
    let mut model = SchemaModel {
        structs: HashMap::new(),
        enums: HashMap::new(),
    };
    for (name, text) in SCHEMA_FILES {
        parse_file(text, &mut model)
            .map_err(|err| Error::message(format!("schema {name} did not parse: {err}")))?;
    }
    let mut params = Vec::new();
    let mut stack = Vec::new();
    if !model.structs.contains_key("RgConf") {
        return Err(Error::message("schema is missing RgConf"));
    }
    walk(&model, "RgConf", "", &mut params, &mut stack)?;
    stack.clear();
    if model.structs.contains_key("GpridgeConfig") {
        walk(&model, "GpridgeConfig", "gpridge", &mut params, &mut stack)?;
    }
    params.retain(|spec| {
        spec.key != "gpridge.canonical" && !spec.key.starts_with("gpridge.canonical.")
    });
    let mut by_key = HashMap::new();
    for (index, spec) in params.iter().enumerate() {
        by_key.insert(spec.key.clone(), index);
    }
    let (alias_index, refine_aliases) = index_aliases(&params);
    let mut cli_index: HashMap<String, Vec<String>> = HashMap::new();
    for spec in &params {
        for flag in &spec.cli_flags {
            let bucket = cli_index.entry(flag.clone()).or_default();
            if !bucket.iter().any(|key| key == &spec.key) {
                bucket.push(spec.key.clone());
            }
        }
    }
    Ok(Catalog {
        params,
        by_key,
        alias_index,
        refine_aliases,
        cli_index,
    })
}

type AliasIndex = HashMap<(String, String, String), Vec<String>>;
type RefineIndex = Vec<(Alias, String)>;

fn index_aliases(params: &[ParamSpec]) -> (AliasIndex, RefineIndex) {
    let mut grouped: HashMap<(String, String, String), Vec<(String, Alias)>> = HashMap::new();
    for spec in params {
        for alias in &spec.aliases {
            grouped
                .entry((
                    alias.format.clone(),
                    alias.section.clone(),
                    alias.key.clone(),
                ))
                .or_default()
                .push((spec.key.clone(), alias.clone()));
        }
    }
    let mut index = HashMap::new();
    let mut refine = Vec::new();
    for ((format, section, key), hits) in grouped {
        if section_owner(&section) == Some("refine") {
            for (path, alias) in hits {
                let suffix = if let Some((_, tail)) = path.rsplit_once(".handoff.") {
                    tail.to_string()
                } else if let Some(tail) = path.rsplit('.').next() {
                    tail.to_string()
                } else {
                    path.clone()
                };
                if refine.iter().any(|(have, have_suffix): &(Alias, String)| {
                    have.key == alias.key && have_suffix == &suffix
                }) {
                    continue;
                }
                refine.push((alias, suffix));
            }
            continue;
        }
        let chosen = select_alias_targets(&section, &hits);
        if !chosen.is_empty() {
            index.insert((format, section, key), chosen);
        }
    }
    (index, refine)
}

fn select_alias_targets(section: &str, hits: &[(String, Alias)]) -> Vec<String> {
    if hits.len() == 1 {
        return vec![hits[0].0.clone()];
    }
    if let Some(owner) = section_owner(section) {
        let matched: Vec<String> = hits
            .iter()
            .filter(|(path, _)| path_matches_owner(path, owner))
            .map(|(path, _)| path.clone())
            .collect();
        if !matched.is_empty() {
            return matched;
        }
    }
    hits.iter()
        .filter(|(path, _)| path.starts_with("band."))
        .map(|(path, _)| path.clone())
        .collect()
}

fn section_owner(section: &str) -> Option<&'static str> {
    let head = match section.split('.').next() {
        Some(item) => item,
        None => section,
    };
    Some(match head {
        "Main" | "main" | "runtime" => "runtime",
        "Structure Comparison" => "structureComparison",
        "Potential" | "LennardJones" | "Morse" | "ZBLPot" | "D3Pot" | "D4Pot" | "XTBPot"
        | "MOPACPot" | "Metatomic" | "cpmd" | "potential" => "potential",
        "Optimizer" | "LBFGS" | "FIRE" | "CG" | "QuickMin" | "SD" | "Xtsci" | "optimizer" => {
            "optimizer"
        }
        "Nudged Elastic Band" | "NEB" | "band" => "band",
        "Saddle Search" | "Dimer" | "Lanczos" | "Davidson" | "ARTn" | "GPR Dimer"
        | "saddleSearch" => "saddleSearch",
        "Dynamics" | "Thermostat" | "dynamics" => "dynamics",
        "Hessian" => "hessian",
        "Prefactor" => "prefactor",
        "Instanton" => "instanton",
        "Lineb" | "lineb" => "lineb",
        "Debug" | "debug" => "debug",
        "Serve" => "serve",
        "run" => "run",
        "Refine" => "refine",
        "con" | "conIo" => "conIo",
        _ => return None,
    })
}

fn path_matches_owner(path: &str, owner: &str) -> bool {
    match owner {
        "run" => {
            path.starts_with("band.")
                || path.starts_with("runtime.")
                || path.starts_with("conIo.")
                || path.starts_with("gpridge.run.")
        }
        other => path == other || path.starts_with(&format!("{other}.")),
    }
}

fn walk(
    model: &SchemaModel,
    struct_name: &str,
    prefix: &str,
    out: &mut Vec<ParamSpec>,
    stack: &mut Vec<String>,
) -> Result<(), Error> {
    if stack.iter().any(|seen| seen == struct_name) {
        return Ok(());
    }
    let def = match model.structs.get(struct_name) {
        Some(def) => def.clone(),
        None => return Ok(()),
    };
    stack.push(struct_name.to_string());
    let arms: Vec<&RawField> = def.fields.iter().filter(|field| field.union_arm).collect();
    if !arms.is_empty() {
        let key = join_key(prefix, "arm");
        let names: Vec<String> = arms.iter().map(|field| field.name.clone()).collect();
        let default = names.first().cloned().unwrap_or_default();
        push_spec(
            out,
            key,
            ScalarKind::Enum(names),
            Some(default),
            &RawField {
                name: "arm".into(),
                type_name: "Text".into(),
                default_text: None,
                mutability: Some("restartOnly".into()),
                meta: BTreeMap::new(),
                doc: "Active union arm.".into(),
                union_arm: false,
                source_line: 0,
            },
        );
    }
    for field in &def.fields {
        if field.name == "canonical" && prefix == "gpridge" {
            continue;
        }
        if field.name == "hydratedArchive" || field.type_name.ends_with("HydratedRunArchive") {
            continue;
        }
        let key = join_key(prefix, &field.name);
        let type_name = unwrap_optional(&field.type_name);
        let local = local_name(type_name);
        if local.ends_with("Patch") || local == "ThirdPartyExtension" {
            continue;
        }
        if local == "CountLimit" || local == "FeatureCount" {
            let default = if local == "CountLimit" {
                "conditionDriven"
            } else {
                "disabled"
            };
            push_spec(out, key, ScalarKind::Text, Some(default.to_string()), field);
            continue;
        }
        if let Some(hit) = resolve_type(model, struct_name, type_name) {
            match hit {
                TypeHit::Enum(names) => {
                    let default = field
                        .default_text
                        .clone()
                        .or_else(|| names.first().cloned());
                    push_spec(out, key, ScalarKind::Enum(names), default, field);
                    continue;
                }
                TypeHit::Struct(qualified) => {
                    walk(model, &qualified, &key, out, stack)?;
                    continue;
                }
            }
        }
        if let Some(kind) = scalar_kind(local) {
            push_spec(out, key, kind, field.default_text.clone(), field);
            continue;
        }
        if local.starts_with("List(") || local == "Data" || local == "Void" {
            continue;
        }
    }
    stack.pop();
    Ok(())
}

fn push_spec(
    out: &mut Vec<ParamSpec>,
    key: String,
    kind: ScalarKind,
    default_text: Option<String>,
    field: &RawField,
) {
    if out.iter().any(|spec| spec.key == key) {
        return;
    }
    let default_value = parse_default(&kind, default_text.as_deref());
    let mutability = match field.mutability.as_deref() {
        Some("betweenStepsWithHook") => Mutability::BetweenStepsWithHook,
        Some("restartOnly") => Mutability::RestartOnly,
        _ => Mutability::MutableLive,
    };
    let default_kind = match word_meta(&field.meta, "defaultKind").as_deref() {
        Some("profileDependent") => DefaultKind::Profile,
        Some("resolverDerived") => DefaultKind::Derived,
        _ => DefaultKind::Literal,
    };
    let doc = {
        let annotated = word_meta(&field.meta, "doc").unwrap_or_default();
        if annotated.is_empty() {
            field.doc.clone()
        } else {
            annotated
        }
    };
    out.push(ParamSpec {
        key,
        kind,
        default_value,
        unit: word_meta(&field.meta, "unit").unwrap_or_else(|| "none".into()),
        dimension: word_meta(&field.meta, "dimension").unwrap_or_else(|| "dimensionless".into()),
        mutability,
        bounds: bounds_from(&field.meta),
        default_kind,
        resolver_function: word_meta(&field.meta, "resolverFunction").unwrap_or_default(),
        deprecated: bool_meta(&field.meta, "deprecated"),
        deprecation_reason: word_meta(&field.meta, "deprecationReason").unwrap_or_default(),
        doc,
        cli_flags: string_list_meta(&field.meta, "cliFlags"),
        aliases: aliases_from(&field.meta),
        source_line: field.source_line,
        type_name: field.type_name.clone(),
    });
}

fn parse_default(kind: &ScalarKind, text: Option<&str>) -> ParamValue {
    let Some(text) = text.map(str::trim).filter(|text| !text.is_empty()) else {
        return match kind {
            ScalarKind::Bool => ParamValue::Bool(false),
            ScalarKind::Int => ParamValue::Int(0),
            ScalarKind::Uint => ParamValue::Uint(0),
            ScalarKind::Float => ParamValue::Float(0.0),
            ScalarKind::Text | ScalarKind::Enum(_) => ParamValue::Text(String::new()),
            ScalarKind::IntList => ParamValue::IntList(Vec::new()),
            ScalarKind::FloatList => ParamValue::FloatList(Vec::new()),
            ScalarKind::TextList => ParamValue::TextList(Vec::new()),
        };
    };
    let unquoted = unquote(text);
    match kind {
        ScalarKind::Bool => ParamValue::Bool(matches!(unquoted.as_str(), "true" | "yes")),
        ScalarKind::Enum(_) | ScalarKind::Text => ParamValue::Text(unquoted),
        ScalarKind::Int | ScalarKind::Uint | ScalarKind::Float => {
            match parse_loose_number(&unquoted).and_then(|parsed| kind.coerce(&parsed)) {
                Ok(value) => value,
                Err(_) => ParamValue::Text(unquoted),
            }
        }
        ScalarKind::IntList => ParamValue::IntList(Vec::new()),
        ScalarKind::FloatList => ParamValue::FloatList(Vec::new()),
        ScalarKind::TextList => ParamValue::TextList(Vec::new()),
    }
}

fn word_meta(meta: &BTreeMap<String, MetaValue>, key: &str) -> Option<String> {
    match meta.get(key) {
        Some(MetaValue::Word(word)) => Some(unquote(word)),
        _ => None,
    }
}

fn bool_meta(meta: &BTreeMap<String, MetaValue>, key: &str) -> bool {
    matches!(word_meta(meta, key).as_deref(), Some("true"))
}

fn string_list_meta(meta: &BTreeMap<String, MetaValue>, key: &str) -> Vec<String> {
    match meta.get(key) {
        Some(MetaValue::List(items)) => items
            .iter()
            .filter_map(|item| match item {
                MetaValue::Word(word) => Some(unquote(word)),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn aliases_from(meta: &BTreeMap<String, MetaValue>) -> Vec<Alias> {
    let Some(MetaValue::List(items)) = meta.get("aliases") else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let MetaValue::Group(group) = item else {
                return None;
            };
            Some(Alias {
                format: group_word(group, "format"),
                section: group_word(group, "section"),
                key: group_word(group, "key"),
                transform: group_word(group, "transform"),
                deprecated: matches!(group_word(group, "deprecated").as_str(), "true"),
                deprecation_note: group_word(group, "deprecationNote"),
            })
        })
        .filter(|alias| !alias.format.is_empty() && !alias.key.is_empty())
        .collect()
}

fn group_word(group: &BTreeMap<String, MetaValue>, key: &str) -> String {
    match group.get(key) {
        Some(MetaValue::Word(word)) => unquote(word),
        _ => String::new(),
    }
}

fn bounds_from(meta: &BTreeMap<String, MetaValue>) -> Bounds {
    let Some(MetaValue::Group(group)) = meta.get("bounds") else {
        return Bounds::default();
    };
    let mut bounds = Bounds::default();
    if matches!(group_word(group, "hasMinInclusive").as_str(), "true") {
        bounds.min_inclusive = group_word(group, "minInclusive").parse().ok();
    }
    if matches!(group_word(group, "hasMinExclusive").as_str(), "true") {
        bounds.min_exclusive = group_word(group, "minExclusive").parse().ok();
    }
    if matches!(group_word(group, "hasMaxInclusive").as_str(), "true") {
        bounds.max_inclusive = group_word(group, "maxInclusive").parse().ok();
    }
    if matches!(group_word(group, "hasMaxExclusive").as_str(), "true") {
        bounds.max_exclusive = group_word(group, "maxExclusive").parse().ok();
    }
    if matches!(group_word(group, "requireFinite").as_str(), "false") {
        bounds.require_finite = false;
    }
    bounds
}

fn unquote(text: &str) -> String {
    let trimmed = text.trim().trim_end_matches(',');
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

fn join_key(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix}.{name}")
    }
}

fn local_name(type_name: &str) -> &str {
    match type_name.rsplit('.').next() {
        Some(item) => item,
        None => type_name,
    }
}

fn unwrap_optional(type_name: &str) -> &str {
    match local_name(type_name) {
        "OptFloat64" => "Float64",
        "OptInt64" => "Int64",
        "OptUInt64" => "UInt64",
        "OptInt32" => "Int32",
        "OptUInt32" => "UInt32",
        "OptUInt16" => "UInt16",
        "OptUInt8" => "UInt8",
        "OptBool" => "Bool",
        "OptText" => "Text",
        _ => type_name,
    }
}

enum TypeHit {
    Enum(Vec<String>),
    Struct(String),
}

enum Lookup {
    Miss,
    Hit(TypeHit),
    Both,
}

/// Resolve `type_name` from the struct currently being walked.
///
/// Nested declarations live under their enclosing path, so a local name
/// that a struct and an enum share is the one in the nearest scope. A
/// dotted reference selects that path. A leading import alias is only
/// dropped after the complete path misses in every enclosing scope.
/// When one path names both a struct and an enum, the reference stays
/// unresolved.
fn resolve_type(model: &SchemaModel, scope: &str, type_name: &str) -> Option<TypeHit> {
    for base in scope_chain(scope) {
        match lookup_exact(model, &join_scope(base, type_name)) {
            Lookup::Hit(hit) => return Some(hit),
            Lookup::Both => return None,
            Lookup::Miss => {}
        }
    }
    let (_, tail) = type_name.split_once('.')?;
    match lookup_exact(model, tail) {
        Lookup::Hit(hit) => Some(hit),
        Lookup::Both | Lookup::Miss => None,
    }
}

fn scope_chain(scope: &str) -> Vec<&str> {
    let mut chain = Vec::new();
    let mut rest = scope;
    loop {
        chain.push(rest);
        if rest.is_empty() {
            break;
        }
        rest = match rest.rsplit_once('.') {
            Some((parent, _)) => parent,
            None => "",
        };
    }
    chain
}

fn join_scope(base: &str, tail: &str) -> String {
    if base.is_empty() {
        tail.to_string()
    } else if tail.is_empty() {
        base.to_string()
    } else {
        format!("{base}.{tail}")
    }
}

fn qualify(scope: &str, name: &str) -> String {
    join_scope(scope, name)
}

fn lookup_exact(model: &SchemaModel, qualified: &str) -> Lookup {
    let as_struct = model.structs.contains_key(qualified);
    match (as_struct, model.enums.get(qualified)) {
        (true, Some(_)) => Lookup::Both,
        (true, None) => Lookup::Hit(TypeHit::Struct(qualified.to_string())),
        (false, Some(names)) => Lookup::Hit(TypeHit::Enum(names.clone())),
        (false, None) => Lookup::Miss,
    }
}

fn scalar_kind(local: &str) -> Option<ScalarKind> {
    Some(match local {
        "Bool" => ScalarKind::Bool,
        "Int8" | "Int16" | "Int32" | "Int64" => ScalarKind::Int,
        "UInt8" | "UInt16" | "UInt32" | "UInt64" => ScalarKind::Uint,
        "Float32" | "Float64" => ScalarKind::Float,
        "Text" => ScalarKind::Text,
        "List(Int32)" | "List(Int64)" | "List(UInt32)" => ScalarKind::IntList,
        "List(Float32)" | "List(Float64)" => ScalarKind::FloatList,
        "List(Text)" => ScalarKind::TextList,
        _ => return None,
    })
}

fn parse_file(text: &str, model: &mut SchemaModel) -> Result<(), Error> {
    let bytes = text.as_bytes();
    let mut scan = Scan {
        s: bytes,
        i: 0,
        line: 1,
        pending_doc: String::new(),
    };
    while scan.i < scan.s.len() {
        scan.skip();
        if scan.i >= scan.s.len() {
            break;
        }
        if scan.s[scan.i] == b'@' {
            scan.skip_statement();
            continue;
        }
        let Some(word) = scan.ident() else {
            scan.bump();
            continue;
        };
        if matches!(word.as_str(), "struct" | "enum" | "interface") {
            scan.take_nested_decl(model, "", &word);
        } else {
            scan.skip_statement();
        }
    }
    Ok(())
}

struct Scan<'a> {
    s: &'a [u8],
    i: usize,
    line: u32,
    pending_doc: String,
}

impl Scan<'_> {
    fn bump(&mut self) {
        if self.i < self.s.len() {
            if self.s[self.i] == b'\n' {
                self.line += 1;
            }
            self.i += 1;
        }
    }

    fn skip(&mut self) {
        loop {
            let mut newlines = 0u32;
            while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
                if self.s[self.i] == b'\n' {
                    newlines += 1;
                }
                self.bump();
            }
            if newlines >= 2 {
                self.pending_doc.clear();
            }
            if self.i < self.s.len() && self.s[self.i] == b'#' {
                self.bump();
                let start = self.i;
                while self.i < self.s.len() && self.s[self.i] != b'\n' {
                    self.bump();
                }
                let text = String::from_utf8_lossy(&self.s[start..self.i])
                    .trim()
                    .to_string();
                if !text.is_empty() {
                    if !self.pending_doc.is_empty() {
                        self.pending_doc.push(' ');
                    }
                    self.pending_doc.push_str(&text);
                }
                continue;
            }
            break;
        }
    }

    fn ident(&mut self) -> Option<String> {
        self.skip();
        if self.i >= self.s.len() || !is_ident_start(self.s[self.i]) {
            return None;
        }
        let start = self.i;
        self.bump();
        while self.i < self.s.len() && is_ident_cont(self.s[self.i]) {
            self.bump();
        }
        Some(String::from_utf8_lossy(&self.s[start..self.i]).into_owned())
    }

    fn eat(&mut self, expected: u8) -> bool {
        self.skip();
        if self.i < self.s.len() && self.s[self.i] == expected {
            self.bump();
            true
        } else {
            false
        }
    }

    fn skip_statement(&mut self) {
        let mut depth = 0i32;
        let mut in_string = false;
        while self.i < self.s.len() {
            let byte = self.s[self.i];
            if in_string {
                if byte == b'\\' {
                    self.bump();
                    self.bump();
                    continue;
                }
                if byte == b'"' {
                    in_string = false;
                }
                self.bump();
                continue;
            }
            if byte == b'"' {
                in_string = true;
                self.bump();
                continue;
            }
            if byte == b'#' {
                while self.i < self.s.len() && self.s[self.i] != b'\n' {
                    self.bump();
                }
                continue;
            }
            if byte == b'{' || byte == b'(' || byte == b'[' {
                depth += 1;
            } else if byte == b'}' || byte == b')' || byte == b']' {
                depth -= 1;
            } else if byte == b';' && depth == 0 {
                self.bump();
                return;
            }
            self.bump();
        }
    }

    fn skip_balanced_body(&mut self) {
        let mut depth = 1i32;
        let mut in_string = false;
        while self.i < self.s.len() && depth > 0 {
            let byte = self.s[self.i];
            if in_string {
                if byte == b'\\' {
                    self.bump();
                    self.bump();
                    continue;
                }
                if byte == b'"' {
                    in_string = false;
                }
                self.bump();
                continue;
            }
            if byte == b'"' {
                in_string = true;
                self.bump();
                continue;
            }
            if byte == b'#' {
                while self.i < self.s.len() && self.s[self.i] != b'\n' {
                    self.bump();
                }
                continue;
            }
            if byte == b'{' {
                depth += 1;
            } else if byte == b'}' {
                depth -= 1;
            }
            self.bump();
        }
    }

    fn parse_enum_names(&mut self) -> Vec<String> {
        let mut names = Vec::new();
        loop {
            self.skip();
            if self.i >= self.s.len() || self.s[self.i] == b'}' {
                self.eat(b'}');
                break;
            }
            if let Some(name) = self.ident() {
                names.push(name);
                let _ = self.eat(b'@');
                while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                    self.bump();
                }
                let _ = self.eat(b';');
            } else {
                self.bump();
            }
        }
        names
    }

    fn take_nested_decl(&mut self, model: &mut SchemaModel, scope: &str, keyword: &str) {
        let Some(name) = self.ident() else {
            return;
        };
        if !self.eat(b'{') {
            return;
        }
        let qualified = qualify(scope, &name);
        match keyword {
            "struct" => {
                let fields = self.parse_struct_fields(model, &qualified);
                model.structs.insert(qualified, StructDef { fields });
            }
            "enum" => {
                let names = self.parse_enum_names();
                model.enums.insert(qualified, names);
            }
            "interface" => self.skip_balanced_body(),
            _ => {}
        }
        self.pending_doc.clear();
    }

    fn parse_struct_fields(&mut self, model: &mut SchemaModel, scope: &str) -> Vec<RawField> {
        let mut fields = Vec::new();
        loop {
            self.skip();
            if self.i >= self.s.len() {
                break;
            }
            if self.s[self.i] == b'}' {
                self.bump();
                break;
            }
            let Some(word) = self.ident() else {
                self.bump();
                continue;
            };
            if matches!(word.as_str(), "struct" | "enum" | "interface") && !self.next_is(b'@') {
                self.take_nested_decl(model, scope, &word);
                continue;
            }
            if word == "union" {
                if self.eat(b'{') {
                    fields.extend(self.parse_union_fields(model, scope));
                }
                continue;
            }
            if let Some(field) = self.parse_field(word, false) {
                fields.push(field);
            }
        }
        fields
    }

    fn parse_union_fields(&mut self, model: &mut SchemaModel, scope: &str) -> Vec<RawField> {
        let mut fields = Vec::new();
        loop {
            self.skip();
            if self.i >= self.s.len() || self.s[self.i] == b'}' {
                self.eat(b'}');
                break;
            }
            let Some(name) = self.ident() else {
                self.bump();
                continue;
            };
            if matches!(name.as_str(), "struct" | "enum" | "interface") && !self.next_is(b'@') {
                self.take_nested_decl(model, scope, &name);
                continue;
            }
            if let Some(field) = self.parse_field(name, true) {
                fields.push(field);
            }
        }
        fields
    }

    fn parse_field(&mut self, name: String, union_arm: bool) -> Option<RawField> {
        let doc = self.pending_doc.clone();
        self.pending_doc.clear();
        if !self.eat(b'@') {
            self.skip_statement();
            return None;
        }
        while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
            self.bump();
        }
        if !self.eat(b':') {
            self.skip_statement();
            return None;
        }
        let type_name = self.parse_type()?;
        let tail = self.read_tail();
        let (default_text, annotation) = split_tail(&tail);
        let (mutability, meta) = parse_annotations(&annotation);
        Some(RawField {
            name,
            type_name,
            default_text,
            mutability,
            meta,
            doc,
            union_arm,
            source_line: self.line,
        })
    }

    fn next_is(&mut self, expected: u8) -> bool {
        self.skip();
        self.i < self.s.len() && self.s[self.i] == expected
    }

    fn parse_type(&mut self) -> Option<String> {
        self.skip();
        let start = self.i;
        let _ = self.ident()?;
        while self.i < self.s.len() && self.s[self.i] == b'.' {
            self.bump();
            if self.ident().is_none() {
                break;
            }
        }
        self.skip();
        if self.i < self.s.len() && self.s[self.i] == b'(' {
            let mut depth = 0i32;
            while self.i < self.s.len() {
                if self.s[self.i] == b'(' {
                    depth += 1;
                } else if self.s[self.i] == b')' {
                    depth -= 1;
                }
                self.bump();
                if depth == 0 {
                    break;
                }
            }
        }
        Some(
            String::from_utf8_lossy(&self.s[start..self.i])
                .into_owned()
                .replace(char::is_whitespace, ""),
        )
    }

    fn read_tail(&mut self) -> String {
        let start = self.i;
        let mut depth = 0i32;
        let mut in_string = false;
        while self.i < self.s.len() {
            let byte = self.s[self.i];
            if in_string {
                if byte == b'\\' {
                    self.bump();
                    self.bump();
                    continue;
                }
                if byte == b'"' {
                    in_string = false;
                }
                self.bump();
                continue;
            }
            if byte == b'"' {
                in_string = true;
                self.bump();
                continue;
            }
            if byte == b'#' {
                while self.i < self.s.len() && self.s[self.i] != b'\n' {
                    self.bump();
                }
                continue;
            }
            if byte == b'{' || byte == b'(' || byte == b'[' {
                depth += 1;
            } else if byte == b'}' || byte == b')' || byte == b']' {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            } else if byte == b';' && depth == 0 {
                let text = String::from_utf8_lossy(&self.s[start..self.i]).into_owned();
                self.bump();
                return text;
            }
            self.bump();
        }
        String::from_utf8_lossy(&self.s[start..self.i]).into_owned()
    }
}

fn is_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_ident_cont(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn split_tail(tail: &str) -> (Option<String>, String) {
    let trimmed = tail.trim();
    if let Some(rest) = trimmed.strip_prefix('=') {
        let (default, annotation) = take_balanced_value(rest.trim_start());
        (
            Some(default.trim().trim_end_matches(',').to_string()),
            annotation,
        )
    } else {
        (None, trimmed.to_string())
    }
}

fn take_balanced_value(text: &str) -> (String, String) {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            in_string = true;
            index += 1;
            continue;
        }
        if byte == b'(' || byte == b'[' || byte == b'{' {
            depth += 1;
        } else if byte == b')' || byte == b']' || byte == b'}' {
            depth -= 1;
        } else if byte == b'$' && depth == 0 {
            return (text[..index].to_string(), text[index..].to_string());
        }
        index += 1;
    }
    (text.to_string(), String::new())
}

fn parse_annotations(text: &str) -> (Option<String>, BTreeMap<String, MetaValue>) {
    let mut mutability = None;
    let mut meta = BTreeMap::new();
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() || bytes[index] != b'$' {
            break;
        }
        index += 1;
        let start = index;
        while index < bytes.len()
            && (bytes[index].is_ascii_alphanumeric()
                || bytes[index] == b'_'
                || bytes[index] == b'.')
        {
            index += 1;
        }
        let name = String::from_utf8_lossy(&bytes[start..index]).into_owned();
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() || bytes[index] != b'(' {
            continue;
        }
        let (body, next) = take_group(&text[index..]);
        index = text.len() - text[index..].len() + next;
        let local = match name.rsplit('.').next() {
            Some(item) => item,
            None => name.as_str(),
        };
        if local == "mutability" {
            mutability = Some(body.trim().to_string());
        } else if local == "meta" {
            meta = parse_group(&body);
        }
    }
    (mutability, meta)
}

fn take_group(text: &str) -> (String, usize) {
    let bytes = text.as_bytes();
    if bytes.first() != Some(&b'(') {
        return (String::new(), 0);
    }
    let mut depth = 0i32;
    let mut in_string = false;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            in_string = true;
        } else if byte == b'(' {
            depth += 1;
        } else if byte == b')' {
            depth -= 1;
            if depth == 0 {
                return (text[1..index].to_string(), index + 1);
            }
        }
        index += 1;
    }
    (text.to_string(), text.len())
}

fn parse_group(text: &str) -> BTreeMap<String, MetaValue> {
    let mut map = BTreeMap::new();
    for piece in split_top(text, b',') {
        let Some((key, value)) = piece.split_once('=') else {
            continue;
        };
        map.insert(key.trim().to_string(), parse_meta_value(value.trim()));
    }
    map
}

fn parse_meta_value(text: &str) -> MetaValue {
    if let Some(inner) = text
        .strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return MetaValue::Group(parse_group(inner));
    }
    if let Some(inner) = text
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    {
        let items = split_top(inner, b',')
            .into_iter()
            .filter(|piece| !piece.trim().is_empty())
            .map(|piece| parse_meta_value(piece.trim()))
            .collect();
        return MetaValue::List(items);
    }
    MetaValue::Word(text.trim().trim_end_matches(',').to_string())
}

fn split_top(text: &str, separator: u8) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut start = 0;
    let mut parts = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            in_string = true;
        } else if byte == b'(' || byte == b'[' || byte == b'{' {
            depth += 1;
        } else if byte == b')' || byte == b']' || byte == b'}' {
            depth -= 1;
        } else if byte == separator && depth == 0 {
            parts.push(text[start..index].to_string());
            start = index + 1;
        }
        index += 1;
    }
    if start < text.len() {
        parts.push(text[start..].to_string());
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::{parse_file, walk, SchemaModel};
    use crate::value::{ParamValue, ScalarKind};
    use std::collections::HashMap;

    fn specs(schema: &str, root: &str) -> Vec<super::ParamSpec> {
        let mut model = SchemaModel {
            structs: HashMap::new(),
            enums: HashMap::new(),
        };
        parse_file(schema, &mut model).expect("schema parses");
        let mut out = Vec::new();
        let mut stack = Vec::new();
        walk(&model, root, "", &mut out, &mut stack).expect("walk");
        out
    }

    fn spec<'a>(rows: &'a [super::ParamSpec], key: &str) -> &'a super::ParamSpec {
        rows.iter()
            .find(|row| row.key == key)
            .unwrap_or_else(|| panic!("missing {key}"))
    }

    #[test]
    fn shared_struct_and_enum_names_follow_scope_and_path() {
        let rows = specs(
            r#"
            struct Tone {
              hz @0 :Float64 = 440.0;
            }
            enum Mode {
              fast @0;
              safe @1;
            }
            struct Holder {
              own @0 :Mode;
              struct Mode {
                enabled @0 :Bool = true;
              }
            }
            struct Mixer {
              tone @0 :Tone;
              enum Tone {
                low @0;
                high @1;
              }
            }
            struct Root {
              holder @0 :Holder;
              mixer @1 :Mixer;
              picked @2 :Mode;
              plain @3 :Tone;
              viaPath @4 :Holder.Mode;
            }
            "#,
            "Root",
        );
        let own = spec(&rows, "holder.own.enabled");
        assert_eq!(own.kind, ScalarKind::Bool);
        assert_eq!(own.default_value, ParamValue::Bool(true));
        assert!(rows.iter().all(|row| row.key != "holder.own"));

        let tone = spec(&rows, "mixer.tone");
        assert_eq!(
            tone.kind,
            ScalarKind::Enum(vec!["low".into(), "high".into()])
        );
        assert_eq!(tone.default_value, ParamValue::Text("low".into()));
        assert!(rows.iter().all(|row| row.key != "mixer.tone.hz"));

        let picked = spec(&rows, "picked");
        assert_eq!(
            picked.kind,
            ScalarKind::Enum(vec!["fast".into(), "safe".into()])
        );
        assert_eq!(picked.default_value, ParamValue::Text("fast".into()));

        let plain = spec(&rows, "plain.hz");
        assert_eq!(plain.kind, ScalarKind::Float);
        assert_eq!(plain.default_value, ParamValue::Float(440.0));

        let via_path = spec(&rows, "viaPath.enabled");
        assert_eq!(via_path.kind, ScalarKind::Bool);
        assert_eq!(via_path.default_value, ParamValue::Bool(true));
    }

    #[test]
    fn dotted_references_resolve_before_local_tail_names() {
        let rows = specs(
            r#"
            enum Mode {
              imported @0;
              fallback @1;
            }
            struct Outer {
              struct Inner {
                struct Leaf {
                  selected @0 :Bool = true;
                }
              }
              struct Holder {
                struct Leaf {
                  wrong @0 :Text;
                }
                enum Mode {
                  local @0;
                }
                sibling @0 :Inner.Leaf;
                imported @1 :Common.Mode;
              }
              holder @0 :Holder;
            }
            "#,
            "Outer",
        );

        let sibling = spec(&rows, "holder.sibling.selected");
        assert_eq!(sibling.kind, ScalarKind::Bool);
        assert_eq!(sibling.default_value, ParamValue::Bool(true));
        assert!(rows.iter().all(|row| row.key != "holder.sibling.wrong"));

        let imported = spec(&rows, "holder.imported");
        assert_eq!(
            imported.kind,
            ScalarKind::Enum(vec!["imported".into(), "fallback".into()])
        );
        assert_eq!(imported.default_value, ParamValue::Text("imported".into()));
    }

    #[test]
    fn keyword_named_fields_are_parsed_by_context() {
        let rows = specs(
            r#"
            struct Root {
              struct @0 :Bool = true;
              enum @1 :Int32 = 7;
              interface @2 :Text = "field";
            }
            "#,
            "Root",
        );

        assert_eq!(spec(&rows, "struct").default_value, ParamValue::Bool(true));
        assert_eq!(spec(&rows, "enum").default_value, ParamValue::Int(7));
        assert_eq!(
            spec(&rows, "interface").default_value,
            ParamValue::Text("field".into())
        );

        let cpmd = specs(
            include_str!("../schema/potential.capnp"),
            "CPMDCpmdSection",
        );
        assert_eq!(
            spec(&cpmd, "interface").default_value,
            ParamValue::Bool(false)
        );
    }

    #[test]
    fn recursive_struct_cycles_are_bounded() {
        let rows = specs(
            r#"
            struct A {
              value @0 :Bool = true;
              next @1 :B;
            }
            struct B {
              count @0 :Int32 = 2;
              next @1 :A;
            }
            struct Root {
              cycle @0 :A;
            }
            "#,
            "Root",
        );

        assert_eq!(
            spec(&rows, "cycle.value").default_value,
            ParamValue::Bool(true)
        );
        assert_eq!(
            spec(&rows, "cycle.next.count").default_value,
            ParamValue::Int(2)
        );
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn nested_schema_leaves_have_specs() {
        let cat = super::catalog().expect("catalog");
        let kind = cat
            .get("potential.common.smearing.kind")
            .expect("smearing.kind");
        assert_eq!(
            kind.kind,
            ScalarKind::Enum(vec![
                "none".into(),
                "fermi".into(),
                "gaussian".into(),
                "methfesselPaxton".into()
            ])
        );
        assert_eq!(kind.default_value, ParamValue::Text("none".into()));
        let width = cat
            .get("potential.common.smearing.widthEv")
            .expect("smearing.widthEv");
        assert_eq!(width.kind, ScalarKind::Float);
        assert_eq!(width.default_value, ParamValue::Float(0.0));
        let torch = cat
            .get("potential.metatomic.torchDeterminism")
            .expect("torchDeterminism");
        assert_eq!(
            torch.kind,
            ScalarKind::Enum(vec!["fast".into(), "deterministic".into()])
        );
        assert_eq!(torch.default_value, ParamValue::Text("fast".into()));
        let oracle = cat
            .get("potential.gpr.oracle.metatomic.torchDeterminism")
            .expect("oracle torchDeterminism");
        assert_eq!(oracle.kind, torch.kind);
        assert_eq!(oracle.default_value, torch.default_value);
    }
}
