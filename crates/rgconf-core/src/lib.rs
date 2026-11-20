// Copyright 2023--present rgconf developers

#![forbid(unsafe_code)]
#![allow(clippy::manual_unwrap_or, clippy::manual_unwrap_or_default)]

/// Packed C ABI version, with the major, minor, and patch bytes in descending order.
pub const ABI_VERSION: u32 = 0x00_01_00;

pub mod catalog;
pub mod config;
pub mod edit;
pub mod error;
pub mod input;
pub mod provenance;
pub mod redact;
pub mod resolve;
pub mod sbom;
pub mod units;
pub mod validate;
pub mod value;

pub use config::{
    BandConfig, ClimbingImageConfig, ConvergenceConfig, FireConfig, HandoffConfig, LbfgsConfig,
    OptimizerConfig, RgConf, RuntimeConfig, SpringConfig, UnitSystem,
};
pub use edit::{apply_edit, EditAction, EditOrigin, ParamEdit};
pub use error::{Error, Located};
pub use input::ConfigLayer;
pub use provenance::{ProvenanceLog, UPSTREAM_PATCHES};
pub use resolve::{Profile, Resolution, Resolver, ValueStatus};
pub use units::{convert, convert_stress, StressSign, Unit};
pub use value::ParamValue;
