# Copyright 2023--present rgconf developers
@0xd782940f9a21b308;

using Common = import "common.capnp";
using Conv = import "convergence.capnp";
using Opt = import "optimizer.capnp";

# Unified dynamics, thermostat, vibrational Hessian/prefactor, instanton,
# rare-event flux (OH-TST, PAFI, FFS, Bennett-Chandler), and long-timescale
# state-to-state kinetics schema.
# DynamicsConfig.thermostat is the single primary authority for MD thermostatting
# (there is no duplicate top-level RgConf.thermostat field); OhTstConfig, PafiConfig,
# and FfsConfig inherit from DynamicsConfig.thermostat via ThermostatPatch when unset.

enum ThermostatKind {
  none @0;
  andersen @1;
  noseHooverChain @2;
  langevin @3;
  bussiCsvr @4;
  gleColoredNoise @5;
  berendsen @6;
}

struct OptThermostatKind {
  union {
    unset @0 :Void;
    value @1 :ThermostatKind;
  }
}

struct ThermostatConfig {
  kind @0 :ThermostatKind = none $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [
      (format = "ini", section = "Thermostat", key = "thermostat"),
      (format = "ini", section = "Dynamics", key = "thermostat")
    ]
  );
  temperatureK @1 :Float64 = 300.0 $Common.mutability(mutableLive) $Common.meta(
    dimension = temperature,
    unit = kelvin,
    bounds = (hasMinInclusive = true, minInclusive = 0.0),
    aliases = [(format = "ini", section = "Main", key = "temperature")]
  );
  andersenCollisionFreqPerFs @2 :Float64 = 1.0 $Common.mutability(mutableLive) $Common.meta(dimension = frictionRate, unit = invFemtosecond);
  andersenAlpha @3 :Float64 = 1.0 $Common.mutability(mutableLive);
  noseMass @4 :Float64 = 1.0 $Common.mutability(betweenStepsWithHook);
  nhcChainLength @5 :UInt16 = 3 $Common.mutability(restartOnly);
  langevinFrictionPerFs @6 :Float64 = 0.01 $Common.mutability(mutableLive) $Common.meta(dimension = frictionRate, unit = invFemtosecond);
  langevinTauFs @7 :Float64 = 100.0 $Common.mutability(mutableLive) $Common.meta(dimension = time, unit = femtosecond);
  bussiTauFs @8 :Float64 = 100.0 $Common.mutability(mutableLive) $Common.meta(dimension = time, unit = femtosecond);
  gleMatrixAPath @9 :Text = "" $Common.mutability(betweenStepsWithHook);
  gleMatrixCPath @10 :Text = "" $Common.mutability(betweenStepsWithHook);
  gleTimeUnit @11 :Common.CanonicalUnit = femtosecond $Common.mutability(restartOnly);
}

struct ThermostatPatch {
  kind @0 :OptThermostatKind;
  temperatureK @1 :Common.OptFloat64;
  andersenCollisionFreqPerFs @2 :Common.OptFloat64;
  andersenAlpha @3 :Common.OptFloat64;
  noseMass @4 :Common.OptFloat64;
  nhcChainLength @5 :Common.OptUInt16;
  langevinFrictionPerFs @6 :Common.OptFloat64;
  langevinTauFs @7 :Common.OptFloat64;
  bussiTauFs @8 :Common.OptFloat64;
  gleMatrixAPath @9 :Common.OptText;
  gleMatrixCPath @10 :Common.OptText;
  hasGleTimeUnit @11 :Bool = false;
  gleTimeUnit @12 :Common.CanonicalUnit = femtosecond;
}

struct DynamicsConfig {
  steps @0 :UInt64 = 1000 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "Dynamics", key = "steps")]
  );
  timeStepFs @1 :Float64 = 1.0 $Common.mutability(mutableLive) $Common.meta(
    dimension = time,
    unit = femtosecond,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [(format = "ini", section = "Dynamics", key = "time_step")]
  );
  temperatureK @2 :Float64 = 300.0 $Common.mutability(mutableLive) $Common.meta(dimension = temperature, unit = kelvin);
  boltzmannConstantEvPerK @3 :Float64 = 8.617333262e-5 $Common.mutability(restartOnly) $Common.meta(dimension = entropyHeatCapacity, unit = evPerKelvin);
  pimdBeads @4 :UInt16 = 1 $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "ini", section = "Dynamics", key = "beads")]
  );
  pimdVirialEstimator @5 :Bool = true $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "Dynamics", key = "virial")]
  );
  # Single authority for molecular dynamics thermostatting.
  thermostat @6 :ThermostatConfig;
}

struct DynamicsPatch {
  steps @0 :Common.OptUInt64;
  timeStepFs @1 :Common.OptFloat64;
  temperatureK @2 :Common.OptFloat64;
  boltzmannConstantEvPerK @3 :Common.OptFloat64;
  pimdBeads @4 :Common.OptUInt16;
  pimdVirialEstimator @5 :Common.OptBool;
  thermostat @6 :ThermostatPatch;
}

struct HessianConfig {
  finiteDifference @0 :Common.FiniteDifferenceConfig;
  atomList @1 :List(Int32) $Common.mutability(restartOnly);
  zeroFreqValue @2 :Float64 = 1e-6 $Common.mutability(mutableLive) $Common.meta(dimension = frequencyWavenumber, unit = invCm);
  removeTranslationRotation @3 :Bool = true $Common.mutability(mutableLive);
}

struct HessianPatch {
  finiteDifference @0 :Common.FiniteDifferencePatch;
  atomList @1 :Common.OptInt32List;
  zeroFreqValue @2 :Common.OptFloat64;
  removeTranslationRotation @3 :Common.OptBool;
}

enum PrefactorRateKind {
  htst @0;
  qqHtst @1;
}

enum PrefactorFilterScheme {
  cutoff @0;
  fraction @1;
}

struct PrefactorConfig {
  defaultValuePerSecond @0 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(dimension = frictionRate, unit = invSecond);
  maxValuePerSecond @1 :Float64 = 1e16 $Common.mutability(mutableLive) $Common.meta(dimension = frictionRate, unit = invSecond);
  minValuePerSecond @2 :Float64 = 1e9 $Common.mutability(mutableLive) $Common.meta(dimension = frictionRate, unit = invSecond);
  withinRadiusAngstrom @3 :Float64 = 3.0 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  minDisplacementAngstrom @4 :Float64 = 0.25 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  rateKind @5 :PrefactorRateKind = htst $Common.mutability(mutableLive);
  configurationKind @6 :Text = "reactant" $Common.mutability(mutableLive);
  allFreeAtoms @7 :Bool = false $Common.mutability(mutableLive);
  smoothCutoff @8 :Bool = false $Common.mutability(mutableLive);
  filterScheme @9 :PrefactorFilterScheme = fraction $Common.mutability(mutableLive);
  filterFraction @10 :Float64 = 0.9 $Common.mutability(mutableLive);
  eigenvalueSolver @11 :Text = "lapack" $Common.mutability(mutableLive);
}

struct PrefactorPatch {
  defaultValuePerSecond @0 :Common.OptFloat64;
  maxValuePerSecond @1 :Common.OptFloat64;
  minValuePerSecond @2 :Common.OptFloat64;
  withinRadiusAngstrom @3 :Common.OptFloat64;
  minDisplacementAngstrom @4 :Common.OptFloat64;
  allFreeAtoms @5 :Common.OptBool;
  smoothCutoff @6 :Common.OptBool;
  filterFraction @7 :Common.OptFloat64;
}

struct InstantonConfig {
  numBeads @0 :UInt32 = 16 $Common.mutability(restartOnly);
  springScale @1 :Float64 = 1.0 $Common.mutability(betweenStepsWithHook);
  temperatureK @2 :Float64 = 200.0 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = temperature, unit = kelvin);
  optimizer @3 :Opt.OptimizerConfig;
}

struct InstantonPatch {
  numBeads @0 :Common.OptUInt32;
  springScale @1 :Common.OptFloat64;
  temperatureK @2 :Common.OptFloat64;
  optimizer @3 :Opt.OptimizerPatch;
}

# Fixed-energy line-integral NEB. Energy and both tolerances retain presence
# across configuration overlays. The physical constants use eV, amu, Å and K.
struct LinebConfig {
  initialPath @0 :Text = "" $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "ini", section = "Lineb", key = "initial_path")],
    cliFlags = ["--lineb-initial-path"],
    doc = "Multi-frame seed band (neb.con). Interior images must start at or above energy plus the turning width."
  );
  energyEv @1 :Common.OptFloat64 $Common.mutability(betweenStepsWithHook) $Common.meta(
    dimension = energy,
    unit = ev,
    aliases = [(format = "ini", section = "Lineb", key = "energy")],
    cliFlags = ["--lineb-energy"],
    doc = "Presence-aware tunneling energy E in eV (eOn default 0.0 when unset)."
  );
  optTolSqrtEv @2 :Common.OptFloat64 $Common.mutability(mutableLive) $Common.meta(
    dimension = massWeightedForce,
    unit = sqrtEv,
    bounds = (hasMinInclusive = true, minInclusive = 0.0),
    aliases = [(format = "ini", section = "Lineb", key = "opt_tol")],
    cliFlags = ["--lineb-opt-tol"],
    defaultKind = resolverDerived,
    resolverFunction = "resolve_lineb_opt_tol_eon_default",
    doc = "Presence-aware mass-weighted nudged-force stop in √eV. Unset resolves to eOn 1e-3."
  );
  perpTolSqrtEv @3 :Common.OptFloat64 $Common.mutability(mutableLive) $Common.meta(
    dimension = massWeightedForce,
    unit = sqrtEv,
    bounds = (hasMinInclusive = true, minInclusive = 0.0),
    aliases = [(format = "ini", section = "Lineb", key = "perp_tol")],
    cliFlags = ["--lineb-perp-tol"],
    defaultKind = resolverDerived,
    resolverFunction = "resolve_lineb_perp_tol_eon_default",
    doc = "Presence-aware mass-weighted perpendicular action-gradient stop in √eV. Unset resolves to eOn 1e-3."
  );
  spring @4 :Float64 = 0.1 $Common.mutability(betweenStepsWithHook) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [(format = "ini", section = "Lineb", key = "spring")],
    cliFlags = ["--lineb-spring"]
  );
  endpointScale @5 :Float64 = 0.2 $Common.mutability(betweenStepsWithHook) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [(format = "ini", section = "Lineb", key = "endpoint_scale")],
    cliFlags = ["--lineb-endpoint-scale"]
  );
  timeStep @6 :Float64 = 0.1 $Common.mutability(mutableLive) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [(format = "ini", section = "Lineb", key = "time_step")],
    cliFlags = ["--lineb-time-step"]
  );
  maxMove @7 :Float64 = 0.2 $Common.mutability(mutableLive) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinInclusive = true, minInclusive = 0.0),
    aliases = [(format = "ini", section = "Lineb", key = "max_move")],
    cliFlags = ["--lineb-max-move"],
    doc = "Step cap; per atom when molecular, else whole-band Euclidean length. 0 uncapped."
  );
  turningWidth @8 :Float64 = 1e-3 $Common.mutability(mutableLive) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinExclusive = true, minExclusive = 0.0, hasMaxExclusive = true, maxExclusive = 1.0),
    aliases = [(format = "ini", section = "Lineb", key = "turning_width")],
    cliFlags = ["--lineb-turning-width"]
  );
  hbar @9 :Float64 = 0.06465415129579072 $Common.mutability(restartOnly) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [(format = "ini", section = "Lineb", key = "hbar")],
    cliFlags = ["--lineb-hbar"],
    doc = "ℏ in eV^0.5 amu^0.5 Angstrom (eOn tunneling::kHbar). Temperature is ℏ/(k_B τ)."
  );
  boltzmann @10 :Float64 = 8.617333262145177e-5 $Common.mutability(restartOnly) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [(format = "ini", section = "Lineb", key = "boltzmann")],
    cliFlags = ["--lineb-boltzmann"],
    doc = "Boltzmann constant in eV/K (eOn tunneling::kBoltzmann)."
  );
  quadraturePairs @11 :UInt32 = 4 $Common.mutability(restartOnly) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinInclusive = true, minInclusive = 1),
    aliases = [(format = "ini", section = "Lineb", key = "quadrature_pairs")],
    cliFlags = ["--lineb-quadrature-pairs"]
  );
  maxSteps @12 :UInt32 = 1000 $Common.mutability(mutableLive) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinInclusive = true, minInclusive = 1),
    aliases = [(format = "ini", section = "Lineb", key = "max_steps")],
    cliFlags = ["--lineb-max-steps"]
  );
  beads @13 :UInt32 = 0 $Common.mutability(restartOnly) $Common.meta(
    dimension = dimensionless,
    unit = none,
    aliases = [(format = "ini", section = "Lineb", key = "beads")],
    cliFlags = ["--lineb-beads"],
    doc = "Use 0 for the path only or at least 2 replicas for harmonic quantum TST."
  );
  molecular @14 :Bool = false $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "ini", section = "Lineb", key = "molecular")],
    cliFlags = ["--lineb-molecular"],
    doc = "false expands atom masses to coordinates; true uses molecule alignment (all atoms free)."
  );
  endpointKappa @15 :Float64 = 0.0 $Common.mutability(betweenStepsWithHook) $Common.meta(
    dimension = inverseRootEnergy,
    unit = invSqrtEv,
    bounds = (hasMinInclusive = true, minInclusive = 0.0),
    aliases = [(format = "ini", section = "Lineb", key = "endpoint_kappa")],
    cliFlags = ["--lineb-endpoint-kappa"],
    doc = "Fixed endpoint force per eV of energy error. Zero selects the gradient-normalized endpoint scale."
  );
  linearSeed @16 :Bool = false $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "ini", section = "Lineb", key = "linear_seed")],
    cliFlags = ["--lineb-linear-seed"],
    doc = "Interpolate along a straight line between the initial MEP energy crossings."
  );
  endpointConvergence @17 :Bool = false $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "Lineb", key = "endpoint_convergence")],
    cliFlags = ["--lineb-endpoint-convergence"],
    doc = "Apply the nudged-force tolerance to endpoints and the perpendicular tolerance to interior images."
  );
  projectedVerlet @18 :Bool = false $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "ini", section = "Lineb", key = "projected_verlet")],
    cliFlags = ["--lineb-projected-verlet"],
    doc = "Optimize the band with velocity-projected velocity Verlet."
  );
  resultCache @19 :Text = "" $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "ini", section = "Lineb", key = "result_cache")],
    cliFlags = ["--lineb-result-cache"],
    doc = "Path to the rgpot geometry result cache. An empty path disables persistent result caching."
  );
}

struct LinebPatch {
  initialPath @0 :Common.OptText;
  energyEv @1 :Common.OptFloat64;
  optTolSqrtEv @2 :Common.OptFloat64;
  perpTolSqrtEv @3 :Common.OptFloat64;
  spring @4 :Common.OptFloat64;
  endpointScale @5 :Common.OptFloat64;
  timeStep @6 :Common.OptFloat64;
  maxMove @7 :Common.OptFloat64;
  turningWidth @8 :Common.OptFloat64;
  hbar @9 :Common.OptFloat64;
  boltzmann @10 :Common.OptFloat64;
  quadraturePairs @11 :Common.OptUInt32;
  maxSteps @12 :Common.OptUInt32;
  beads @13 :Common.OptUInt32;
  molecular @14 :Common.OptBool;
  endpointKappa @15 :Common.OptFloat64;
  linearSeed @16 :Common.OptBool;
  endpointConvergence @17 :Common.OptBool;
  projectedVerlet @18 :Common.OptBool;
  resultCache @19 :Common.OptText;
}

struct OhTstConfig {
  temperatureK @0 :Float64 = 300.0 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = temperature, unit = kelvin);
  numQuadratureNodes @1 :UInt16 = 8 $Common.mutability(restartOnly);
  quadratureRadiusAngstrom @2 :Float64 = 0.5 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = length, unit = angstrom);
  timeStepFs @3 :Float64 = 1.0 $Common.mutability(mutableLive) $Common.meta(dimension = time, unit = femtosecond);
  equilibrationSteps @4 :UInt32 = 100 $Common.mutability(mutableLive);
  productionSteps @5 :UInt32 = 500 $Common.mutability(mutableLive);
  decorrelationStride @6 :UInt32 = 10 $Common.mutability(mutableLive);
  recrossingSteps @7 :UInt32 = 100 $Common.mutability(mutableLive);
  recrossingTrajectories @8 :UInt32 = 20 $Common.mutability(mutableLive);
  harmonicMatchTolerance @9 :Float64 = 0.05 $Common.mutability(mutableLive);
  blockCount @10 :UInt16 = 5 $Common.mutability(mutableLive);
  bootstrapResamples @11 :UInt32 = 200 $Common.mutability(mutableLive);
  # Optional override over DynamicsConfig.thermostat; resolved from DynamicsConfig.thermostat when unset.
  thermostatOverride @12 :ThermostatPatch;
}

struct OhTstPatch {
  temperatureK @0 :Common.OptFloat64;
  numQuadratureNodes @1 :Common.OptUInt16;
  quadratureRadiusAngstrom @2 :Common.OptFloat64;
  timeStepFs @3 :Common.OptFloat64;
  equilibrationSteps @4 :Common.OptUInt32;
  productionSteps @5 :Common.OptUInt32;
  decorrelationStride @6 :Common.OptUInt32;
  recrossingSteps @7 :Common.OptUInt32;
  recrossingTrajectories @8 :Common.OptUInt32;
  harmonicMatchTolerance @9 :Common.OptFloat64;
  blockCount @10 :Common.OptUInt16;
  bootstrapResamples @11 :Common.OptUInt32;
  thermostatOverride @12 :ThermostatPatch;
}

struct PafiConfig {
  temperatureK @0 :Float64 = 300.0 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = temperature, unit = kelvin);
  numHyperplanes @1 :UInt16 = 10 $Common.mutability(restartOnly);
  equilibrationStepsPerPlane @2 :UInt32 = 200 $Common.mutability(mutableLive);
  productionStepsPerPlane @3 :UInt32 = 1000 $Common.mutability(mutableLive);
  timeStepFs @4 :Float64 = 1.0 $Common.mutability(mutableLive) $Common.meta(dimension = time, unit = femtosecond);
  thermostatOverride @5 :ThermostatPatch;
  bootstrapResamples @6 :UInt32 = 200 $Common.mutability(mutableLive);
}

struct PafiPatch {
  temperatureK @0 :Common.OptFloat64;
  numHyperplanes @1 :Common.OptUInt16;
  equilibrationStepsPerPlane @2 :Common.OptUInt32;
  productionStepsPerPlane @3 :Common.OptUInt32;
  timeStepFs @4 :Common.OptFloat64;
  thermostatOverride @5 :ThermostatPatch;
  bootstrapResamples @6 :Common.OptUInt32;
}

struct FfsConfig {
  temperatureK @0 :Float64 = 300.0 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = temperature, unit = kelvin);
  lambdaInterfaces @1 :List(Float64) $Common.mutability(restartOnly);
  trialsPerInterface @2 :UInt32 = 100 $Common.mutability(mutableLive);
  basinEscapeSteps @3 :UInt32 = 5000 $Common.mutability(mutableLive);
  timeStepFs @4 :Float64 = 1.0 $Common.mutability(mutableLive) $Common.meta(dimension = time, unit = femtosecond);
  thermostatOverride @5 :ThermostatPatch;
}

struct FfsPatch {
  temperatureK @0 :Common.OptFloat64;
  lambdaInterfaces @1 :Common.OptFloat64List;
  trialsPerInterface @2 :Common.OptUInt32;
  basinEscapeSteps @3 :Common.OptUInt32;
  timeStepFs @4 :Common.OptFloat64;
  thermostatOverride @5 :ThermostatPatch;
}

struct BennettChandlerConfig {
  temperatureK @0 :Float64 = 300.0 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = temperature, unit = kelvin);
  numTrajectories @1 :UInt32 = 100 $Common.mutability(mutableLive);
  halfTrajectorySteps @2 :UInt32 = 200 $Common.mutability(mutableLive);
  timeStepFs @3 :Float64 = 1.0 $Common.mutability(mutableLive) $Common.meta(dimension = time, unit = femtosecond);
  plateauWindowStartStep @4 :UInt32 = 50 $Common.mutability(mutableLive);
  plateauWindowEndStep @5 :UInt32 = 200 $Common.mutability(mutableLive);
}

struct BennettChandlerPatch {
  temperatureK @0 :Common.OptFloat64;
  numTrajectories @1 :Common.OptUInt32;
  halfTrajectorySteps @2 :Common.OptUInt32;
  timeStepFs @3 :Common.OptFloat64;
  plateauWindowStartStep @4 :Common.OptUInt32;
  plateauWindowEndStep @5 :Common.OptUInt32;
}

struct ProcessSearchConfig {
  minimizationOffsetAngstrom @0 :Float64 = 0.2 $Common.mutability(mutableLive) $Common.meta(
    dimension = length,
    unit = angstrom,
    aliases = [(format = "ini", section = "Process Search", key = "minimization_offset")],
    defaultKind = resolverDerived,
    resolverFunction = "resolve_minimization_offset_from_max_move"
  );
  defaultPrefactorPerSecond @1 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(dimension = frictionRate, unit = invSecond);
  minimizeFirst @2 :Bool = true $Common.mutability(mutableLive);
}

struct ProcessSearchPatch {
  minimizationOffsetAngstrom @0 :Common.OptFloat64;
  defaultPrefactorPerSecond @1 :Common.OptFloat64;
  minimizeFirst @2 :Common.OptBool;
}

struct BasinHoppingConfig {
  steps @0 :UInt32 = 1000 $Common.mutability(mutableLive);
  initialRandomStructureProbability @1 :Float64 = 0.0 $Common.mutability(mutableLive);
  pushApartDistanceAngstrom @2 :Float64 = 0.4 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  displacementAngstrom @3 :Float64 = 0.5 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  initialStatePoolSize @4 :UInt32 = 1 $Common.mutability(restartOnly);
  significantStructure @5 :Bool = true $Common.mutability(mutableLive);
  singleAtomDisplace @6 :Bool = false $Common.mutability(mutableLive);
  displacementAlgorithm @7 :Text = "standard" $Common.mutability(mutableLive);
  displacementDistribution @8 :Text = "gaussian" $Common.mutability(mutableLive);
  swapProbability @9 :Float64 = 0.0 $Common.mutability(mutableLive);
  jumpMax @10 :UInt32 = 10 $Common.mutability(mutableLive);
  jumpSteps @11 :UInt32 = 0 $Common.mutability(mutableLive);
  adjustDisplacement @12 :Bool = true $Common.mutability(mutableLive);
  adjustPeriod @13 :UInt32 = 10 $Common.mutability(mutableLive);
  adjustFraction @14 :Float64 = 0.05 $Common.mutability(mutableLive);
  targetAcceptRatio @15 :Float64 = 0.5 $Common.mutability(mutableLive);
  writeUnique @16 :Bool = false $Common.mutability(mutableLive);
  stopEnergyEv @17 :Float64 = -1e300 $Common.mutability(mutableLive) $Common.meta(dimension = energy, unit = ev);
}

struct ParallelReplicaConfig {
  dephaseLoopStop @0 :Bool = false $Common.mutability(mutableLive);
  dephaseTimeFs @1 :Float64 = 1000.0 $Common.mutability(mutableLive) $Common.meta(dimension = time, unit = femtosecond);
  stateSaveIntervalSteps @2 :UInt32 = 1000 $Common.mutability(mutableLive);
  refineTransition @3 :Bool = true $Common.mutability(mutableLive);
  corrTimeFs @4 :Float64 = 1000.0 $Common.mutability(mutableLive) $Common.meta(dimension = time, unit = femtosecond);
}

struct TadConfig {
  lowTemperatureK @0 :Float64 = 300.0 $Common.mutability(mutableLive) $Common.meta(dimension = temperature, unit = kelvin);
  minPrefactorPerSecond @1 :Float64 = 1e12 $Common.mutability(mutableLive) $Common.meta(dimension = frictionRate, unit = invSecond);
  deltaConfidence @2 :Float64 = 0.001 $Common.mutability(mutableLive);
}

struct ReplicaExchangeConfig {
  samplingTimeFs @0 :Float64 = 100.0 $Common.mutability(mutableLive) $Common.meta(dimension = time, unit = femtosecond);
  replicas @1 :UInt16 = 1 $Common.mutability(restartOnly);
  exchangeTrials @2 :UInt32 = 0 $Common.mutability(mutableLive);
  temperatureDistribution @3 :Text = "exponential" $Common.mutability(restartOnly);
  temperatureLowK @4 :Float64 = 300.0 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = temperature, unit = kelvin);
  temperatureHighK @5 :Float64 = 600.0 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = temperature, unit = kelvin);
  exchangePeriodFs @6 :Float64 = 100.0 $Common.mutability(mutableLive) $Common.meta(dimension = time, unit = femtosecond);
}

enum HyperdynamicsBiasKind {
  none @0;
  bondBoost @1;
}

struct HyperdynamicsConfig {
  biasPotential @0 :HyperdynamicsBiasKind = none $Common.mutability(betweenStepsWithHook);
  rmdTimeFs @1 :Float64 = 100.0 $Common.mutability(mutableLive) $Common.meta(dimension = time, unit = femtosecond);
  boostAtomList @2 :Text = "All" $Common.mutability(restartOnly);
  bondBoostBals @3 :Text = "all" $Common.mutability(restartOnly);
  bondBoostDvmaxEv @4 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(dimension = energy, unit = ev);
  bondBoostQcut @5 :Float64 = 0.2 $Common.mutability(mutableLive);
  bondBoostDs @6 :Float64 = 0.0 $Common.mutability(mutableLive);
  bondBoostRcutAngstrom @7 :Float64 = 3.0 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = length, unit = angstrom);
}

struct AkmcConfig {
  confidence @0 :Float64 = 0.99 $Common.mutability(mutableLive);
  thermallyAccessibleWindowEv @1 :Float64 = 20.0 $Common.mutability(mutableLive) $Common.meta(dimension = energy, unit = ev);
  maxEnersEv @2 :Float64 = 999999.0 $Common.mutability(mutableLive) $Common.meta(dimension = energy, unit = ev);
  serverSideProcessSearch @3 :Bool = false $Common.mutability(restartOnly);
  confidenceScheme @4 :Text = "new" $Common.mutability(mutableLive);
  confidenceCorrection @5 :Bool = false $Common.mutability(mutableLive);
  maxKmcSteps @6 :UInt64 = 100 $Common.mutability(mutableLive);
}

struct KdbConfig {
  useKdb @0 :Bool = false $Common.mutability(mutableLive);
  onlyKdb @1 :Bool = false $Common.mutability(mutableLive);
  kdbScratchPath @2 :Text = "kdbscratch" $Common.mutability(restartOnly);
  kdbPath @3 :Text = "kdb" $Common.mutability(restartOnly);
  nf @4 :Float64 = 0.2 $Common.mutability(mutableLive);
  dc @5 :Float64 = 0.3 $Common.mutability(mutableLive);
  mac @6 :Float64 = 0.7 $Common.mutability(mutableLive);
}

struct RecyclingConfig {
  useSurrogate @0 :Bool = false $Common.mutability(mutableLive);
  saveAllProcesses @1 :Bool = false $Common.mutability(mutableLive);
  moveDistanceAngstrom @2 :Float64 = 0.2 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
}

struct CoarseGrainingConfig {
  useMilestones @0 :Bool = false $Common.mutability(mutableLive);
  rateMethod @1 :Text = "novotny" $Common.mutability(mutableLive);
  absorbingRadius @2 :UInt32 = 1 $Common.mutability(mutableLive);
  projectNotTransient @3 :Bool = false $Common.mutability(mutableLive);
  energyWindowEv @4 :Float64 = 0.5 $Common.mutability(mutableLive) $Common.meta(dimension = energy, unit = ev);
  useAmsel @5 :Bool = false $Common.mutability(mutableLive);
}

struct AmselConfig {
  rateErrorTolerance @0 :Float64 = 0.2 $Common.mutability(mutableLive);
  probabilityFluxShare @1 :Float64 = 0.99 $Common.mutability(mutableLive);
  equilibriumCheckSteps @2 :UInt32 = 10 $Common.mutability(mutableLive);
  equilibriumTolerance @3 :Float64 = 0.05 $Common.mutability(mutableLive);
}

struct BgsdConfig {
  alpha @0 :Float64 = 10.0 $Common.mutability(mutableLive);
  beta @1 :Float64 = 0.1 $Common.mutability(mutableLive);
  gradientFiniteDifference @2 :Float64 = 1e-5 $Common.mutability(mutableLive);
  hForceConverged @3 :Float64 = 1e-4 $Common.mutability(mutableLive);
  gradH2Converged @4 :Float64 = 1e-2 $Common.mutability(mutableLive);
}

struct IraConfig {
  distanceThresholdAngstrom @0 :Float64 = 0.8 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  symmetryThresholdAngstrom @1 :Float64 = 0.3 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
}
