// Copyright 2023--present rgconf developers

use crate::catalog::Mutability;
use crate::error::Error;
use crate::input::{Assignment, ConfigLayer};
use crate::resolve::{is_user, Resolution, ValueStatus};
use crate::value::ParamValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditOrigin {
    LiveCall,
    CheckpointRestart,
}

impl EditOrigin {
    pub fn name(self) -> &'static str {
        match self {
            Self::LiveCall => "liveCall",
            Self::CheckpointRestart => "checkpointRestart",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditAction {
    None,
    ResetLbfgsHistory,
    RebuildPreconditioner,
    RecomputeClimbingImage,
    RedistributeBandImages,
    ReinitializeThermostatRng,
    ReopenPotential,
    ClearForceCache,
    ResetLbfgsAfterPotential,
    ReseedDimerRotation,
    ClearEndpointCache,
    ResetLbfgsAfterCell,
}

impl EditAction {
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ResetLbfgsHistory => "resetLbfgsHistory",
            Self::RebuildPreconditioner => "rebuildPreconditioner",
            Self::RecomputeClimbingImage => "recomputeClimbingImage",
            Self::RedistributeBandImages => "redistributeBandImages",
            Self::ReinitializeThermostatRng => "reinitializeThermostatRng",
            Self::ReopenPotential => "reopenPotential",
            Self::ClearForceCache => "clearForceCache",
            Self::ResetLbfgsAfterPotential => "resetLbfgsAfterPotential",
            Self::ReseedDimerRotation => "reseedDimerRotation",
            Self::ClearEndpointCache => "clearEndpointCache",
            Self::ResetLbfgsAfterCell => "resetLbfgsAfterCell",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamEdit {
    pub key: String,
    pub previous: String,
    pub next: String,
    pub origin: EditOrigin,
    pub actions: Vec<EditAction>,
}

pub fn apply_edit(
    resolution: &mut Resolution,
    key: &str,
    value: ParamValue,
    origin: EditOrigin,
) -> Result<ParamEdit, Error> {
    let field = resolution
        .fields
        .get(key)
        .ok_or_else(|| Error::message(format!("unknown parameter `{key}`")))?;
    if field.mutability == Mutability::RestartOnly && origin != EditOrigin::CheckpointRestart {
        return Err(Error::message(format!(
            "`{key}` is restartOnly and cannot change during a live run"
        )));
    }
    let previous = field.value.to_string();
    let actions = actions_for(key, field.mutability);
    let assignment = Assignment {
        key: key.to_string(),
        value,
        origin: format!("edit:{}", origin.name()),
        line: 0,
        column: 1,
        via: format!("ParamEdit {}", origin.name()),
        deprecated: false,
        deprecation_note: String::new(),
        refine_suffix: None,
    };
    crate::resolve::Resolver::apply_assignment(resolution, &assignment)?;
    if let Some(field) = resolution.fields.get_mut(key) {
        if is_user(field.status) || field.status == ValueStatus::MutatedMidRun {
            field.status = ValueStatus::MutatedMidRun;
            field.layer = ConfigLayer::LiveRuntimeEdit;
        }
    }
    let next = resolution.require(key)?.value.to_string();
    let edit = ParamEdit {
        key: key.to_string(),
        previous,
        next,
        origin,
        actions,
    };
    resolution.edits.push(edit.clone());
    Ok(edit)
}

fn actions_for(key: &str, mutability: Mutability) -> Vec<EditAction> {
    if mutability == Mutability::MutableLive {
        return vec![EditAction::None];
    }
    let mut actions = Vec::new();
    if key.contains("optimizer.method") || key.contains("lbfgs") {
        actions.push(EditAction::ResetLbfgsHistory);
    }
    if key.contains("preconditioner") || key.contains("neighbor") {
        actions.push(EditAction::RebuildPreconditioner);
    }
    if key.contains("spring")
        || key.contains("tangent")
        || key.contains("projection")
        || key.contains("climbing")
        || key.ends_with("forceNorm")
    {
        actions.push(EditAction::RecomputeClimbingImage);
    }
    if key.ends_with("imageCount") || key.contains("zoom") {
        actions.push(EditAction::RedistributeBandImages);
    }
    if key.contains("thermostat") {
        actions.push(EditAction::ReinitializeThermostatRng);
    }
    if key.starts_with("potential.") {
        actions.push(EditAction::ReopenPotential);
        actions.push(EditAction::ClearForceCache);
        actions.push(EditAction::ResetLbfgsAfterPotential);
    }
    if key.contains("dimer") || key.contains("estimator") {
        actions.push(EditAction::ReseedDimerRotation);
    }
    if key.contains("cell") || key.contains("solidState") {
        actions.push(EditAction::ClearEndpointCache);
        actions.push(EditAction::ResetLbfgsAfterCell);
    }
    if actions.is_empty() {
        actions.push(EditAction::None);
    }
    actions
}
