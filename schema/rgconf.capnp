# Copyright 2023--present rgconf developers
@0xd782940f9a21b30a;

using Common = import "common.capnp";
using Conv = import "convergence.capnp";
using Opt = import "optimizer.capnp";
using Refine = import "refine.capnp";
using Band = import "band.capnp";
using MinMode = import "minmode.capnp";
using Surrogate = import "surrogate.capnp";
using Pot = import "potential.capnp";
using Dyn = import "dynamics.capnp";
using Prov = import "provenance.capnp";

# Root rgconf resolved configuration (`RgConf`) and presence-aware input patch
# (`RgConfPatch`).
#
# Single-authority invariants enforced in RgConf:
#   1. No top-level `RgConf.refine`: minimization handoff lives in `optimizer.handoff`,
#      band refinement lives in `band.refine`, and saddle refinement lives in `saddleSearch.refine`.
#   2. No top-level `RgConf.thermostat`: molecular dynamics thermostatting lives in
#      `dynamics.thermostat`, and flux methods (`ohTst`, `pafi`, `ffs`) hold `ThermostatPatch`
#      overrides that inherit from `dynamics.thermostat` when unset.
#   3. No redundant `potential.kind`: the active union discriminant on `Pot.PotentialConfig`
#      is the sole authority for the potential backend.

enum JobKind {
  minimization @0;
  saddleSearch @1;
  nudgedElasticBand @2;
  pointEvaluation @3;
  hessianFrequencies @4;
  prefactorHtst @5;
  molecularDynamics @6;
  parallelReplica @7;
  temperatureAcceleratedDynamics @8;
  replicaExchange @9;
  hyperdynamics @10;
  basinHopping @11;
  globalOptimization @12;
  monteCarlo @13;
  akmcProcessSearch @14;
  bgsd @15;
  instanton @16;
  ohTstFlux @17;
  pafiFreeEnergy @18;
  forwardFluxSampling @19;
  bennettChandler @20;
  intrinsicReactionCoordinate @21;
  rpcPotentialServer @22;
}

struct OptJobKind {
  union {
    unset @0 :Void;
    value @1 :JobKind;
  }
}

struct RuntimeConfig {
  job @0 :JobKind = minimization $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "ini", section = "Main", key = "job")],
    cliFlags = ["--job"],
    defaultKind = profileDependent,
    resolverFunction = "resolve_job_kind_from_profile"
  );
  temperatureK @1 :Float64 = 300.0 $Common.mutability(betweenStepsWithHook) $Common.meta(
    dimension = temperature,
    unit = kelvin,
    bounds = (hasMinInclusive = true, minInclusive = 0.0),
    aliases = [(format = "ini", section = "Main", key = "temperature")],
    cliFlags = ["--temperature"]
  );
  randomSeed @2 :UInt64 = 42 $Common.mutability(restartOnly) $Common.meta(
    aliases = [
      (format = "ini", section = "Main", key = "random_seed"),
      (format = "toml", section = "run", key = "seed")
    ],
    cliFlags = ["--seed"]
  );
  checkpoint @3 :Bool = false $Common.mutability(mutableLive);
  quiet @4 :Bool = false $Common.mutability(mutableLive);
  writeLog @5 :Bool = true $Common.mutability(mutableLive);
  maxForceCalls @6 :UInt64 = 0 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "Main", key = "max_force_calls")],
    cliFlags = ["--max-force-calls"]
  );
  finiteDifference @7 :Common.FiniteDifferenceConfig;
}

struct RuntimePatch {
  job @0 :OptJobKind;
  temperatureK @1 :Common.OptFloat64;
  randomSeed @2 :Common.OptUInt64;
  checkpoint @3 :Common.OptBool;
  quiet @4 :Common.OptBool;
  writeLog @5 :Common.OptBool;
  maxForceCalls @6 :Common.OptUInt64;
  finiteDifference @7 :Common.FiniteDifferencePatch;
}

struct StructureComparisonConfig {
  distanceDifferenceAngstrom @0 :Float64 = 0.1 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  neighborCutoffAngstrom @1 :Float64 = 3.3 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = length, unit = angstrom);
  checkRotation @2 :Bool = false $Common.mutability(mutableLive);
  indistinguishableAtoms @3 :Bool = true $Common.mutability(mutableLive);
  energyDifferenceEv @4 :Float64 = 0.01 $Common.mutability(mutableLive) $Common.meta(dimension = energy, unit = ev);
  removeTranslation @5 :Bool = true $Common.mutability(mutableLive);
  useCovalent @6 :Bool = false $Common.mutability(betweenStepsWithHook);
  covalentScale @7 :Float64 = 1.3 $Common.mutability(betweenStepsWithHook);
  bruteNeighbors @8 :Bool = false $Common.mutability(betweenStepsWithHook);
}

struct StructureComparisonPatch {
  distanceDifferenceAngstrom @0 :Common.OptFloat64;
  neighborCutoffAngstrom @1 :Common.OptFloat64;
  checkRotation @2 :Common.OptBool;
  indistinguishableAtoms @3 :Common.OptBool;
  energyDifferenceEv @4 :Common.OptFloat64;
  removeTranslation @5 :Common.OptBool;
  useCovalent @6 :Common.OptBool;
  covalentScale @7 :Common.OptFloat64;
  bruteNeighbors @8 :Common.OptBool;
}

struct ServeInstanceSpec {
  potentialConfig @0 :Pot.PotentialConfig;
  host @1 :Text = "localhost";
  port @2 :UInt16 = 12345;
  configSectionOverride @3 :Text = "";
}

struct ServeConfig {
  mode @0 :Text = "single" $Common.mutability(restartOnly);
  host @1 :Text = "localhost" $Common.mutability(restartOnly);
  port @2 :UInt16 = 12345 $Common.mutability(restartOnly);
  controlPort @3 :UInt16 = 0 $Common.mutability(restartOnly); # Port for Prov.RuntimeControl RPC
  controlUnixSocket @4 :Text = "" $Common.mutability(restartOnly);
  endpointsSpec @5 :Text = "" $Common.mutability(restartOnly);
  gatewayPort @6 :UInt16 = 0 $Common.mutability(restartOnly);
  replicas @7 :UInt16 = 1 $Common.mutability(restartOnly);
  instances @8 :List(ServeInstanceSpec);
}

struct DebugConfig {
  writeMovies @0 :Bool = false $Common.mutability(mutableLive);
  writeMoviesInterval @1 :UInt32 = 1 $Common.mutability(mutableLive);
  keepAllResultFiles @2 :Bool = false $Common.mutability(mutableLive);
  registerExtraResults @3 :Bool = false $Common.mutability(mutableLive);
  useMeanTime @4 :Bool = false $Common.mutability(mutableLive);
  estimateRate @5 :Bool = false $Common.mutability(mutableLive);
}

struct DebugPatch {
  writeMovies @0 :Common.OptBool;
  writeMoviesInterval @1 :Common.OptUInt32;
  keepAllResultFiles @2 :Common.OptBool;
  registerExtraResults @3 :Common.OptBool;
  useMeanTime @4 :Common.OptBool;
  estimateRate @5 :Common.OptBool;
}

enum ConCompressionKind {
  none @0;
  gzip @1;
  bzip2 @2;
  xz @3;
  zstd @4;
}

enum ConWriteMode {
  overwrite @0;
  append @1;
}

struct ConIoConfig {
  reactantPath @0 :Text = "reactant.con" $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "toml", section = "run", key = "reactant")],
    cliFlags = ["--reactant"]
  );
  productPath @1 :Text = "product.con" $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "toml", section = "run", key = "product")],
    cliFlags = ["--product"]
  );
  conserveHeaderComments @2 :Bool = true $Common.mutability(mutableLive);
  compression @3 :ConCompressionKind = none $Common.mutability(mutableLive);
  writeMode @4 :ConWriteMode = overwrite $Common.mutability(mutableLive);
  sqliteWalMode @5 :Bool = true $Common.mutability(restartOnly);
  sqliteBusyTimeoutMs @6 :UInt32 = 5000 $Common.mutability(mutableLive);
}

struct ConIoPatch {
  reactantPath @0 :Common.OptText;
  productPath @1 :Common.OptText;
  conserveHeaderComments @2 :Common.OptBool;
  sqliteWalMode @3 :Common.OptBool;
  sqliteBusyTimeoutMs @4 :Common.OptUInt32;
}

# Resolved, plain scalar-valued root configuration tree with one authority per domain.
struct RgConf {
  schemaVersion @0 :Common.SchemaVersion;
  activeProfile @1 :Common.CompatibilityProfile = rgconfCanonical;
  units @2 :Common.UnitSystem;
  runtime @3 :RuntimeConfig;
  structureComparison @4 :StructureComparisonConfig;
  potential @5 :Pot.PotentialConfig;
  optimizer @6 :Opt.OptimizerConfig;
  band @7 :Band.BandConfig;
  saddleSearch @8 :MinMode.SaddleSearchConfig;
  surrogateGp @9 :Surrogate.GpConfig;
  uncertainty @10 :Surrogate.UncertaintyConfig;
  acquisition @11 :Surrogate.AcquisitionConfig;
  trust @12 :Surrogate.TrustConfig;
  linearAlgebra @13 :Surrogate.LinearAlgebraConfig;
  dynamics @14 :Dyn.DynamicsConfig;
  hessian @15 :Dyn.HessianConfig;
  prefactor @16 :Dyn.PrefactorConfig;
  instanton @17 :Dyn.InstantonConfig;
  ohTst @18 :Dyn.OhTstConfig;
  pafi @19 :Dyn.PafiConfig;
  ffs @20 :Dyn.FfsConfig;
  bennettChandler @21 :Dyn.BennettChandlerConfig;
  processSearch @22 :Dyn.ProcessSearchConfig;
  akmc @23 :Dyn.AkmcConfig;
  kdb @24 :Dyn.KdbConfig;
  recycling @25 :Dyn.RecyclingConfig;
  coarseGraining @26 :Dyn.CoarseGrainingConfig;
  amsel @27 :Dyn.AmselConfig;
  basinHopping @28 :Dyn.BasinHoppingConfig;
  parallelReplica @29 :Dyn.ParallelReplicaConfig;
  tad @30 :Dyn.TadConfig;
  replicaExchange @31 :Dyn.ReplicaExchangeConfig;
  hyperdynamics @32 :Dyn.HyperdynamicsConfig;
  bgsd @33 :Dyn.BgsdConfig;
  ira @34 :Dyn.IraConfig;
  serve @35 :ServeConfig;
  debug @36 :DebugConfig;
  conIo @37 :ConIoConfig;
  hydratedArchive @38 :Prov.HydratedRunArchive;
  # Runtime third-party parameters. A compiled companion schema (see
  # gpridge.capnp) is the typed form. This list carries namespaced keys
  # `ext.<namespace>.<name>` that no companion schema declared.
  extensions @39 :List(ThirdPartyExtension);
}

# Namespaced parameter a third party adds without forking RgConf ordinals.
struct ThirdPartyExtension {
  namespace @0 :Text;
  name @1 :Text;
  doc @2 :Text = "";
  deprecated @3 :Bool = false;
  deprecationNote @4 :Text = "";
  value @5 :Prov.ParamValue;
}

# Presence-aware input overlay for each ConfigLayer and for live RuntimeControl.applyPatch calls.
struct RgConfPatch {
  sourceLayer @0 :Prov.ConfigLayer = jobConfigFile;
  sourceOrigin @1 :Text = "";
  runtime @2 :RuntimePatch;
  structureComparison @3 :StructureComparisonPatch;
  potential @4 :Pot.PotentialPatch;
  optimizer @5 :Opt.OptimizerPatch;
  band @6 :Band.BandPatch;
  saddleSearch @7 :MinMode.SaddleSearchPatch;
  surrogateGp @8 :Surrogate.GpPatch;
  uncertainty @9 :Surrogate.UncertaintyPatch;
  acquisition @10 :Surrogate.AcquisitionPatch;
  trust @11 :Surrogate.TrustPatch;
  linearAlgebra @12 :Surrogate.LinearAlgebraPatch;
  dynamics @13 :Dyn.DynamicsPatch;
  hessian @14 :Dyn.HessianPatch;
  prefactor @15 :Dyn.PrefactorPatch;
  instanton @16 :Dyn.InstantonPatch;
  ohTst @17 :Dyn.OhTstPatch;
  pafi @18 :Dyn.PafiPatch;
  ffs @19 :Dyn.FfsPatch;
  bennettChandler @20 :Dyn.BennettChandlerPatch;
  processSearch @21 :Dyn.ProcessSearchPatch;
  debug @22 :DebugPatch;
  conIo @23 :ConIoPatch;
  extensions @24 :List(ThirdPartyExtension);
}
