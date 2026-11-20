// Copyright 2023--present rgconf developers

use crate::error::Error;
use std::f64::consts::PI;

/// CODATA 2018 Bohr radius, in angstroms.
pub const BOHR_IN_ANGSTROM: f64 = 0.529177210903;
/// CODATA 2018 hartree, in electronvolts.
pub const HARTREE_IN_EV: f64 = 27.211386245988;
/// Thermochemical calorie, exact, in kilojoules.
pub const KCAL_IN_KJ: f64 = 4.184;
/// CODATA 2018 electronvolt, in kJ/mol.
pub const EV_IN_KJ_PER_MOL: f64 = 96.48533212331002;
/// One electronvolt per cubic angstrom, in gigapascals.
pub const EV_PER_ANGSTROM3_IN_GPA: f64 = 160.2176634;
/// One eOn time unit, in femtoseconds.
pub const EON_TIME_UNIT_IN_FEMTOSECONDS: f64 = 10.1805055;
/// CODATA 2018 atomic unit of time, in femtoseconds.
pub const ATOMIC_TIME_UNIT_IN_FEMTOSECONDS: f64 = 0.024188843265857;
/// CODATA 2018 electron mass, in atomic mass units.
pub const ELECTRON_MASS_IN_AMU: f64 = 0.000548579909065;
/// Nanometres in one angstrom's reciprocal: 1 nm = 10 angstrom.
pub const ANGSTROM_PER_NANOMETER: f64 = 10.0;
/// Femtoseconds in one picosecond.
pub const FEMTOSECONDS_PER_PICOSECOND: f64 = 1000.0;
/// Femtoseconds in one second.
pub const FEMTOSECONDS_PER_SECOND: f64 = 1_000_000_000_000_000.0;
/// How far a length, energy, or force may drift across a unit round trip.
pub const UNIT_ROUNDTRIP_ABS_TOLERANCE: f64 = 0.000_000_000_001;
/// Paper gpridge force: one thousandth of a hartree per bohr, expressed in eV/angstrom.
pub const GPRIDGE_PAPER_FORCE_DENOMINATOR: f64 = 1000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dimension {
    Dimensionless,
    Length,
    Energy,
    Time,
    Force,
    ForceConstant,
    Angle,
    Mass,
    Temperature,
    Charge,
    PressureStress,
    Frequency,
    Dipole,
    Memory,
    /// Norm of a force in mass-weighted coordinates (√eV).
    MassWeightedForce,
    InverseRootEnergy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StressSign {
    CauchyPositiveTensile,
    VirialPositiveCompressive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    None,
    Angstrom,
    Bohr,
    Nanometer,
    Ev,
    Hartree,
    Rydberg,
    KjPerMol,
    KcalPerMol,
    Femtosecond,
    Picosecond,
    Second,
    EonTimeUnit,
    AtomicTimeUnit,
    EvPerAngstrom,
    HartreePerBohr,
    EvPerAngstrom2,
    HartreePerBohr2,
    EvPerAngstrom3,
    HartreePerBohr3,
    Gpa,
    Radian,
    Degree,
    Amu,
    ElectronMass,
    Kelvin,
    ElementaryCharge,
    InvCm,
    Debye,
    Megabyte,
    Gigabyte,
    /// Mass-weighted force unit √eV (LI-NEB opt/perp tolerances).
    SqrtEv,
    InvSqrtEv,
}

impl Unit {
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name.trim() {
            "none" => Self::None,
            "angstrom" => Self::Angstrom,
            "bohr" => Self::Bohr,
            "nanometer" => Self::Nanometer,
            "ev" => Self::Ev,
            "hartree" => Self::Hartree,
            "rydberg" => Self::Rydberg,
            "kjPerMol" => Self::KjPerMol,
            "kcalPerMol" => Self::KcalPerMol,
            "femtosecond" => Self::Femtosecond,
            "picosecond" => Self::Picosecond,
            "second" => Self::Second,
            "eonTimeUnit" => Self::EonTimeUnit,
            "atomicTimeUnit" => Self::AtomicTimeUnit,
            "evPerAngstrom" => Self::EvPerAngstrom,
            "hartreePerBohr" => Self::HartreePerBohr,
            "evPerAngstrom2" => Self::EvPerAngstrom2,
            "hartreePerBohr2" => Self::HartreePerBohr2,
            "evPerAngstrom3" => Self::EvPerAngstrom3,
            "hartreePerBohr3" => Self::HartreePerBohr3,
            "gpa" => Self::Gpa,
            "radian" => Self::Radian,
            "degree" => Self::Degree,
            "amu" => Self::Amu,
            "electronMass" => Self::ElectronMass,
            "kelvin" => Self::Kelvin,
            "elementaryCharge" => Self::ElementaryCharge,
            "invCm" => Self::InvCm,
            "debye" => Self::Debye,
            "megabyte" => Self::Megabyte,
            "gigabyte" => Self::Gigabyte,
            "sqrtEv" => Self::SqrtEv,
            "invSqrtEv" => Self::InvSqrtEv,
            _ => return None,
        })
    }

    pub fn dimension(self) -> Dimension {
        match self {
            Self::None => Dimension::Dimensionless,
            Self::Angstrom | Self::Bohr | Self::Nanometer => Dimension::Length,
            Self::Ev | Self::Hartree | Self::Rydberg | Self::KjPerMol | Self::KcalPerMol => {
                Dimension::Energy
            }
            Self::Femtosecond
            | Self::Picosecond
            | Self::Second
            | Self::EonTimeUnit
            | Self::AtomicTimeUnit => Dimension::Time,
            Self::EvPerAngstrom | Self::HartreePerBohr => Dimension::Force,
            Self::EvPerAngstrom2 | Self::HartreePerBohr2 => Dimension::ForceConstant,
            Self::EvPerAngstrom3 | Self::HartreePerBohr3 | Self::Gpa => Dimension::PressureStress,
            Self::Radian | Self::Degree => Dimension::Angle,
            Self::Amu | Self::ElectronMass => Dimension::Mass,
            Self::Kelvin => Dimension::Temperature,
            Self::ElementaryCharge => Dimension::Charge,
            Self::InvCm => Dimension::Frequency,
            Self::Debye => Dimension::Dipole,
            Self::Megabyte | Self::Gigabyte => Dimension::Memory,
            Self::SqrtEv => Dimension::MassWeightedForce,
            Self::InvSqrtEv => Dimension::InverseRootEnergy,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Angstrom => "angstrom",
            Self::Bohr => "bohr",
            Self::Nanometer => "nanometer",
            Self::Ev => "ev",
            Self::Hartree => "hartree",
            Self::Rydberg => "rydberg",
            Self::KjPerMol => "kjPerMol",
            Self::KcalPerMol => "kcalPerMol",
            Self::Femtosecond => "femtosecond",
            Self::Picosecond => "picosecond",
            Self::Second => "second",
            Self::EonTimeUnit => "eonTimeUnit",
            Self::AtomicTimeUnit => "atomicTimeUnit",
            Self::EvPerAngstrom => "evPerAngstrom",
            Self::HartreePerBohr => "hartreePerBohr",
            Self::EvPerAngstrom2 => "evPerAngstrom2",
            Self::HartreePerBohr2 => "hartreePerBohr2",
            Self::EvPerAngstrom3 => "evPerAngstrom3",
            Self::HartreePerBohr3 => "hartreePerBohr3",
            Self::Gpa => "gpa",
            Self::Radian => "radian",
            Self::Degree => "degree",
            Self::Amu => "amu",
            Self::ElectronMass => "electronMass",
            Self::Kelvin => "kelvin",
            Self::ElementaryCharge => "elementaryCharge",
            Self::InvCm => "invCm",
            Self::Debye => "debye",
            Self::Megabyte => "megabyte",
            Self::Gigabyte => "gigabyte",
            Self::SqrtEv => "sqrtEv",
            Self::InvSqrtEv => "invSqrtEv",
        }
    }
}

/// Convert `value` from `from` into `to`. Both units must share a dimension.
pub fn convert(value: f64, from: Unit, to: Unit) -> Result<f64, Error> {
    if !value.is_finite() {
        return Err(Error::message(format!(
            "refusing to convert a non finite quantity `{value}`"
        )));
    }
    if from == to {
        return Ok(value);
    }
    if from.dimension() != to.dimension() || from.dimension() == Dimension::Dimensionless {
        return Err(Error::message(format!(
            "cannot convert {} ({:?}) into {} ({:?})",
            from.name(),
            from.dimension(),
            to.name(),
            to.dimension()
        )));
    }
    let canonical = to_canonical(value, from)?;
    from_canonical(canonical, to)
}

pub fn convert_stress(
    value: f64,
    from: Unit,
    from_sign: StressSign,
    to: Unit,
    to_sign: StressSign,
) -> Result<f64, Error> {
    let converted = convert(value, from, to)?;
    if from_sign == to_sign {
        Ok(converted)
    } else {
        Ok(-converted)
    }
}

/// `1/1000` hartree per bohr, in eV per angstrom. The design text rounds this
/// to 0.0514221. The resolver uses this quotient so the factor stays exact.
pub fn gpridge_paper_force_ev_per_angstrom() -> f64 {
    (HARTREE_IN_EV / BOHR_IN_ANGSTROM) / GPRIDGE_PAPER_FORCE_DENOMINATOR
}

fn to_canonical(value: f64, unit: Unit) -> Result<f64, Error> {
    Ok(match unit {
        Unit::Angstrom
        | Unit::Ev
        | Unit::Femtosecond
        | Unit::EvPerAngstrom
        | Unit::EvPerAngstrom2
        | Unit::EvPerAngstrom3
        | Unit::Radian
        | Unit::Amu
        | Unit::Kelvin
        | Unit::ElementaryCharge
        | Unit::Megabyte
        | Unit::SqrtEv
        | Unit::InvSqrtEv => value,
        Unit::Bohr => value * BOHR_IN_ANGSTROM,
        Unit::Nanometer => value * ANGSTROM_PER_NANOMETER,
        Unit::Hartree => value * HARTREE_IN_EV,
        Unit::Rydberg => value * (HARTREE_IN_EV / 2.0),
        Unit::KjPerMol => value / EV_IN_KJ_PER_MOL,
        Unit::KcalPerMol => value * KCAL_IN_KJ / EV_IN_KJ_PER_MOL,
        Unit::Picosecond => value * FEMTOSECONDS_PER_PICOSECOND,
        Unit::Second => value * FEMTOSECONDS_PER_SECOND,
        Unit::EonTimeUnit => value * EON_TIME_UNIT_IN_FEMTOSECONDS,
        Unit::AtomicTimeUnit => value * ATOMIC_TIME_UNIT_IN_FEMTOSECONDS,
        Unit::HartreePerBohr => value * HARTREE_IN_EV / BOHR_IN_ANGSTROM,
        Unit::HartreePerBohr2 => value * HARTREE_IN_EV / (BOHR_IN_ANGSTROM * BOHR_IN_ANGSTROM),
        Unit::HartreePerBohr3 => {
            value * HARTREE_IN_EV / (BOHR_IN_ANGSTROM * BOHR_IN_ANGSTROM * BOHR_IN_ANGSTROM)
        }
        Unit::Gpa => value / EV_PER_ANGSTROM3_IN_GPA,
        Unit::Degree => value * PI / 180.0,
        Unit::ElectronMass => value * ELECTRON_MASS_IN_AMU,
        Unit::Gigabyte => value * 1024.0,
        other => {
            return Err(Error::message(format!(
                "unit {} has no canonical conversion",
                other.name()
            )));
        }
    })
}

fn from_canonical(value: f64, unit: Unit) -> Result<f64, Error> {
    Ok(match unit {
        Unit::Angstrom
        | Unit::Ev
        | Unit::Femtosecond
        | Unit::EvPerAngstrom
        | Unit::EvPerAngstrom2
        | Unit::EvPerAngstrom3
        | Unit::Radian
        | Unit::Amu
        | Unit::Kelvin
        | Unit::ElementaryCharge
        | Unit::Megabyte
        | Unit::SqrtEv
        | Unit::InvSqrtEv => value,
        Unit::Bohr => value / BOHR_IN_ANGSTROM,
        Unit::Nanometer => value / ANGSTROM_PER_NANOMETER,
        Unit::Hartree => value / HARTREE_IN_EV,
        Unit::Rydberg => value / (HARTREE_IN_EV / 2.0),
        Unit::KjPerMol => value * EV_IN_KJ_PER_MOL,
        Unit::KcalPerMol => value * EV_IN_KJ_PER_MOL / KCAL_IN_KJ,
        Unit::Picosecond => value / FEMTOSECONDS_PER_PICOSECOND,
        Unit::Second => value / FEMTOSECONDS_PER_SECOND,
        Unit::EonTimeUnit => value / EON_TIME_UNIT_IN_FEMTOSECONDS,
        Unit::AtomicTimeUnit => value / ATOMIC_TIME_UNIT_IN_FEMTOSECONDS,
        Unit::HartreePerBohr => value * BOHR_IN_ANGSTROM / HARTREE_IN_EV,
        Unit::HartreePerBohr2 => value * (BOHR_IN_ANGSTROM * BOHR_IN_ANGSTROM) / HARTREE_IN_EV,
        Unit::HartreePerBohr3 => {
            value * (BOHR_IN_ANGSTROM * BOHR_IN_ANGSTROM * BOHR_IN_ANGSTROM) / HARTREE_IN_EV
        }
        Unit::Gpa => value * EV_PER_ANGSTROM3_IN_GPA,
        Unit::Degree => value * 180.0 / PI,
        Unit::ElectronMass => value / ELECTRON_MASS_IN_AMU,
        Unit::Gigabyte => value / 1024.0,
        other => {
            return Err(Error::message(format!(
                "unit {} has no canonical conversion",
                other.name()
            )));
        }
    })
}
