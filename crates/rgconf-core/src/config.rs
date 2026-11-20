// Copyright 2023--present rgconf developers

//! Public views of a resolved configuration.
//!
//! Every field is a parameter the schema already defines. `Default` matches
//! the schema literal. `with_*` returns a new value.

use crate::error::Error;
use crate::resolve::Resolution;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct UnitSystem {
    pub length: String,
    pub energy: String,
    pub time: String,
    pub force: String,
    pub angle: String,
    pub mass: String,
    pub temperature: String,
    pub charge: String,
    pub pressure_stress: String,
    pub stress_sign: String,
    pub frequency: String,
    pub dipole: String,
    pub hessian: String,
}

impl Default for UnitSystem {
    fn default() -> Self {
        Self {
            length: "angstrom".to_string(),
            energy: "ev".to_string(),
            time: "femtosecond".to_string(),
            force: "evPerAngstrom".to_string(),
            angle: "radian".to_string(),
            mass: "amu".to_string(),
            temperature: "kelvin".to_string(),
            charge: "elementaryCharge".to_string(),
            pressure_stress: "evPerAngstrom3".to_string(),
            stress_sign: "cauchyPositiveTensile".to_string(),
            frequency: "invCm".to_string(),
            dipole: "debye".to_string(),
            hessian: "evPerAngstrom2".to_string(),
        }
    }
}

impl UnitSystem {
    pub fn with_length(mut self, length: impl Into<String>) -> Self {
        self.length = length.into();
        self
    }

    pub fn with_energy(mut self, energy: impl Into<String>) -> Self {
        self.energy = energy.into();
        self
    }

    pub fn with_time(mut self, time: impl Into<String>) -> Self {
        self.time = time.into();
        self
    }

    pub fn with_force(mut self, force: impl Into<String>) -> Self {
        self.force = force.into();
        self
    }

    pub fn with_angle(mut self, angle: impl Into<String>) -> Self {
        self.angle = angle.into();
        self
    }

    pub fn with_mass(mut self, mass: impl Into<String>) -> Self {
        self.mass = mass.into();
        self
    }

    pub fn with_temperature(mut self, temperature: impl Into<String>) -> Self {
        self.temperature = temperature.into();
        self
    }

    pub fn with_charge(mut self, charge: impl Into<String>) -> Self {
        self.charge = charge.into();
        self
    }

    pub fn with_pressure_stress(mut self, pressure_stress: impl Into<String>) -> Self {
        self.pressure_stress = pressure_stress.into();
        self
    }

    pub fn with_stress_sign(mut self, stress_sign: impl Into<String>) -> Self {
        self.stress_sign = stress_sign.into();
        self
    }

    pub fn with_frequency(mut self, frequency: impl Into<String>) -> Self {
        self.frequency = frequency.into();
        self
    }

    pub fn with_dipole(mut self, dipole: impl Into<String>) -> Self {
        self.dipole = dipole.into();
        self
    }

    pub fn with_hessian(mut self, hessian: impl Into<String>) -> Self {
        self.hessian = hessian.into();
        self
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct ConvergenceConfig {
    pub force_tolerance_ev_per_angstrom: f64,
    pub force_norm: String,
}

impl Default for ConvergenceConfig {
    fn default() -> Self {
        Self {
            force_tolerance_ev_per_angstrom: 0.01,
            force_norm: "maxAtomL2".to_string(),
        }
    }
}

impl ConvergenceConfig {
    pub fn with_force_tolerance_ev_per_angstrom(mut self, value: f64) -> Self {
        self.force_tolerance_ev_per_angstrom = value;
        self
    }

    pub fn with_force_norm(mut self, value: impl Into<String>) -> Self {
        self.force_norm = value.into();
        self
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct LbfgsConfig {
    pub memory: u32,
    pub max_move_angstrom: f64,
}

impl Default for LbfgsConfig {
    fn default() -> Self {
        Self {
            memory: 20,
            max_move_angstrom: 0.2,
        }
    }
}

impl LbfgsConfig {
    pub fn with_memory(mut self, memory: u32) -> Self {
        self.memory = memory;
        self
    }

    pub fn with_max_move_angstrom(mut self, max_move_angstrom: f64) -> Self {
        self.max_move_angstrom = max_move_angstrom;
        self
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct FireConfig {
    pub timestep_femtoseconds: f64,
}

impl Default for FireConfig {
    fn default() -> Self {
        Self {
            timestep_femtoseconds: 1.0,
        }
    }
}

impl FireConfig {
    pub fn with_timestep_femtoseconds(mut self, timestep_femtoseconds: f64) -> Self {
        self.timestep_femtoseconds = timestep_femtoseconds;
        self
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct HandoffConfig {
    pub enabled: bool,
    pub secondary_method: String,
    pub force_threshold_ev_per_angstrom: f64,
}

impl Default for HandoffConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            secondary_method: "fire".to_string(),
            force_threshold_ev_per_angstrom: 0.5,
        }
    }
}

impl HandoffConfig {
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_secondary_method(mut self, method: impl Into<String>) -> Self {
        self.secondary_method = method.into();
        self
    }

    pub fn with_force_threshold_ev_per_angstrom(mut self, value: f64) -> Self {
        self.force_threshold_ev_per_angstrom = value;
        self
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct SpringConfig {
    pub constant_ev_per_angstrom2: f64,
    pub ew_k_min_ev_per_angstrom2: f64,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            constant_ev_per_angstrom2: 5.0,
            ew_k_min_ev_per_angstrom2: 0.972,
        }
    }
}

impl SpringConfig {
    pub fn with_constant_ev_per_angstrom2(mut self, value: f64) -> Self {
        self.constant_ev_per_angstrom2 = value;
        self
    }

    pub fn with_ew_k_min_ev_per_angstrom2(mut self, value: f64) -> Self {
        self.ew_k_min_ev_per_angstrom2 = value;
        self
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct ClimbingImageConfig {
    pub trigger_force_ev_per_angstrom: f64,
}

impl Default for ClimbingImageConfig {
    fn default() -> Self {
        Self {
            trigger_force_ev_per_angstrom: 0.5,
        }
    }
}

impl ClimbingImageConfig {
    pub fn with_trigger_force_ev_per_angstrom(mut self, value: f64) -> Self {
        self.trigger_force_ev_per_angstrom = value;
        self
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizerConfig {
    pub method: String,
    pub max_move_angstrom: f64,
    pub convergence: ConvergenceConfig,
    pub lbfgs: LbfgsConfig,
    pub fire: FireConfig,
    pub handoff: HandoffConfig,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            method: "lbfgs".to_string(),
            max_move_angstrom: 0.2,
            convergence: ConvergenceConfig::default(),
            lbfgs: LbfgsConfig::default(),
            fire: FireConfig::default(),
            handoff: HandoffConfig::default(),
        }
    }
}

impl OptimizerConfig {
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }

    pub fn with_max_move_angstrom(mut self, value: f64) -> Self {
        self.max_move_angstrom = value;
        self
    }

    pub fn with_convergence(mut self, convergence: ConvergenceConfig) -> Self {
        self.convergence = convergence;
        self
    }

    pub fn with_lbfgs(mut self, lbfgs: LbfgsConfig) -> Self {
        self.lbfgs = lbfgs;
        self
    }

    pub fn with_fire(mut self, fire: FireConfig) -> Self {
        self.fire = fire;
        self
    }

    pub fn with_handoff(mut self, handoff: HandoffConfig) -> Self {
        self.handoff = handoff;
        self
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct BandConfig {
    pub image_count: u32,
    pub convergence: ConvergenceConfig,
    pub spring: SpringConfig,
    pub climbing: ClimbingImageConfig,
}

impl Default for BandConfig {
    fn default() -> Self {
        Self {
            image_count: 5,
            convergence: ConvergenceConfig::default(),
            spring: SpringConfig::default(),
            climbing: ClimbingImageConfig::default(),
        }
    }
}

impl BandConfig {
    pub fn with_image_count(mut self, image_count: u32) -> Self {
        self.image_count = image_count;
        self
    }

    pub fn with_convergence(mut self, convergence: ConvergenceConfig) -> Self {
        self.convergence = convergence;
        self
    }

    pub fn with_spring(mut self, spring: SpringConfig) -> Self {
        self.spring = spring;
        self
    }

    pub fn with_climbing(mut self, climbing: ClimbingImageConfig) -> Self {
        self.climbing = climbing;
        self
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeConfig {
    pub job: String,
    pub temperature_k: f64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            job: "minimization".to_string(),
            temperature_k: 300.0,
        }
    }
}

impl RuntimeConfig {
    pub fn with_job(mut self, job: impl Into<String>) -> Self {
        self.job = job.into();
        self
    }

    pub fn with_temperature_k(mut self, temperature_k: f64) -> Self {
        self.temperature_k = temperature_k;
        self
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RgConf {
    pub runtime: RuntimeConfig,
    pub optimizer: OptimizerConfig,
    pub band: BandConfig,
    pub units: UnitSystem,
}

impl RgConf {
    pub fn with_runtime(mut self, runtime: RuntimeConfig) -> Self {
        self.runtime = runtime;
        self
    }

    pub fn with_optimizer(mut self, optimizer: OptimizerConfig) -> Self {
        self.optimizer = optimizer;
        self
    }

    pub fn with_band(mut self, band: BandConfig) -> Self {
        self.band = band;
        self
    }

    pub fn with_units(mut self, units: UnitSystem) -> Self {
        self.units = units;
        self
    }

    pub fn from_resolution(resolution: &Resolution) -> Result<Self, Error> {
        Ok(Self {
            runtime: RuntimeConfig {
                job: text(resolution, "runtime.job")?,
                temperature_k: number(resolution, "runtime.temperatureK")?,
            },
            optimizer: OptimizerConfig {
                method: text(resolution, "optimizer.method")?,
                max_move_angstrom: number(resolution, "optimizer.maxAtomMoveAngstrom")?,
                convergence: ConvergenceConfig {
                    force_tolerance_ev_per_angstrom: number(
                        resolution,
                        "optimizer.convergence.forceToleranceEvPerAngstrom",
                    )?,
                    force_norm: text(resolution, "optimizer.convergence.norm")?,
                },
                lbfgs: LbfgsConfig {
                    memory: unsigned(resolution, "optimizer.lbfgs.memory")? as u32,
                    max_move_angstrom: number(resolution, "optimizer.maxAtomMoveAngstrom")?,
                },
                fire: FireConfig {
                    timestep_femtoseconds: number(resolution, "optimizer.fire.timeStepFs")?,
                },
                handoff: HandoffConfig {
                    enabled: flag(resolution, "optimizer.handoff.enabled")?,
                    secondary_method: text(resolution, "optimizer.handoff.secondaryMethod")?,
                    force_threshold_ev_per_angstrom: number(
                        resolution,
                        "optimizer.handoff.forceThresholdEvPerAngstrom",
                    )?,
                },
            },
            band: BandConfig {
                image_count: unsigned(resolution, "band.imageCount")? as u32,
                convergence: ConvergenceConfig {
                    force_tolerance_ev_per_angstrom: number(
                        resolution,
                        "band.convergence.forceToleranceEvPerAngstrom",
                    )?,
                    force_norm: text(resolution, "band.convergence.norm")?,
                },
                spring: SpringConfig {
                    constant_ev_per_angstrom2: number(
                        resolution,
                        "band.spring.constantEvPerAngstrom2",
                    )?,
                    ew_k_min_ev_per_angstrom2: number(
                        resolution,
                        "band.spring.ewKMinEvPerAngstrom2",
                    )?,
                },
                climbing: ClimbingImageConfig {
                    trigger_force_ev_per_angstrom: number(
                        resolution,
                        "band.climbingImage.triggerForceEvPerAngstrom",
                    )?,
                },
            },
            units: UnitSystem {
                length: text(resolution, "units.length")?,
                energy: text(resolution, "units.energy")?,
                time: text(resolution, "units.time")?,
                force: "evPerAngstrom".to_string(),
                angle: text(resolution, "units.angle")?,
                mass: text(resolution, "units.mass")?,
                temperature: text(resolution, "units.temperature")?,
                charge: text(resolution, "units.charge")?,
                pressure_stress: text(resolution, "units.pressureStress")?,
                stress_sign: text(resolution, "units.stressSign")?,
                frequency: text(resolution, "units.frequency")?,
                dipole: text(resolution, "units.dipole")?,
                hessian: text(resolution, "units.hessian")?,
            },
        })
    }
}

fn text(resolution: &Resolution, key: &str) -> Result<String, Error> {
    Ok(resolution.require(key)?.value.to_string())
}

fn number(resolution: &Resolution, key: &str) -> Result<f64, Error> {
    let field = resolution.require(key)?;
    match &field.value {
        crate::value::ParamValue::Float(value) => Ok(*value),
        crate::value::ParamValue::Int(value) => Ok(*value as f64),
        crate::value::ParamValue::Uint(value) => Ok(*value as f64),
        other => Err(Error::message(format!("`{key}` is {other}, not a number"))),
    }
}

fn unsigned(resolution: &Resolution, key: &str) -> Result<u64, Error> {
    match &resolution.require(key)?.value {
        crate::value::ParamValue::Uint(value) => Ok(*value),
        crate::value::ParamValue::Int(value) if *value >= 0 => Ok(*value as u64),
        other => Err(Error::message(format!(
            "`{key}` is {other}, not an unsigned count"
        ))),
    }
}

fn flag(resolution: &Resolution, key: &str) -> Result<bool, Error> {
    match &resolution.require(key)?.value {
        crate::value::ParamValue::Bool(value) => Ok(*value),
        other => Err(Error::message(format!("`{key}` is {other}, not a boolean"))),
    }
}
