# Copyright 2023--present rgconf developers
@0xd782940f9a21b305;

using Common = import "common.capnp";
using Conv = import "convergence.capnp";
using Opt = import "optimizer.capnp";
using Refine = import "refine.capnp";

# Unified elastic band and string method schema (resolved scalar structs + presence-aware patches).
# BandConfig.imageCount is ALWAYS the total image count including both fixed endpoints
# (minimum 3: reactant + 1 interior image + product). Source formats that specify
# intermediate images only (such as gpridge `band.nimages = 8`) map via `total = nimages + 2`
# (`10` total images) on ingestion and subtract `2` on reverse projection.

enum BandMethodKind {
  nudgedElasticBand @0;
  zeroTemperatureString @1;
  growingString @2;
  freezingString @3;
}

struct OptBandMethodKind {
  union {
    unset @0 :Void;
    value @1 :BandMethodKind;
  }
}

enum TangentKind {
  simpleFiniteDifference @0;
  upwindEnergyWeighted @1;
  higherOrder @2;
}

struct OptTangentKind {
  union {
    unset @0 :Void;
    value @1 :TangentKind;
  }
}

enum ProjectionKind {
  nudged @0;
  plainElastic @1;
  doublyNudged @2;
  doublyNudgedSwitching @3;
}

struct OptProjectionKind {
  union {
    unset @0 :Void;
    value @1 :ProjectionKind;
  }
}

enum SpringKind {
  uniform @0;
  weightedSegment @1;
  energyWeighted @2;
  onsagerMachlup @3;
  elasticBand @4;
  geometric @5;
}

struct OptSpringKind {
  union {
    unset @0 :Void;
    value @1 :SpringKind;
  }
}

struct SpringConfig {
  kind @0 :SpringKind = uniform $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [
      (format = "ini", section = "Nudged Elastic Band", key = "energy_weighted"),
      (format = "ini", section = "Nudged Elastic Band", key = "om_spring"),
      (format = "toml", section = "band", key = "geometric_spring")
    ],
    cliFlags = ["--geometric-spring"]
  );
  constantEvPerAngstrom2 @1 :Float64 = 5.0 $Common.mutability(betweenStepsWithHook) $Common.meta(
    dimension = forceConstant,
    unit = evPerAngstrom2,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [
      (format = "ini", section = "Nudged Elastic Band", key = "spring"),
      (format = "toml", section = "band", key = "spring_constant")
    ],
    cliFlags = ["--spring-constant"]
  );
  segmentWeights @2 :List(Float64) $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = forceConstant, unit = evPerAngstrom2);
  ewKMinEvPerAngstrom2 @3 :Float64 = 0.972 $Common.mutability(betweenStepsWithHook) $Common.meta(
    dimension = forceConstant,
    unit = evPerAngstrom2,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [
      (format = "ini", section = "Nudged Elastic Band", key = "ew_ksp_min"),
      (format = "toml", section = "band", key = "spring_constant_min")
    ],
    cliFlags = ["--spring-constant-min"],
    defaultKind = resolverDerived,
    resolverFunction = "resolve_ew_k_min_or_tenth_ceiling"
  );
  ewKMaxEvPerAngstrom2 @4 :Float64 = 9.72 $Common.mutability(betweenStepsWithHook) $Common.meta(
    dimension = forceConstant,
    unit = evPerAngstrom2,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [(format = "ini", section = "Nudged Elastic Band", key = "ew_ksp_max")]
  );
  ewTriggerForceEvPerAngstrom @5 :Float64 = 0.5 $Common.mutability(mutableLive) $Common.meta(
    dimension = force,
    unit = evPerAngstrom,
    aliases = [(format = "ini", section = "Nudged Elastic Band", key = "ew_trigger")]
  );
  omActionScale @6 :Float64 = 0.01 $Common.mutability(betweenStepsWithHook);
  omOptimizeK @7 :Bool = false $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "ini", section = "Nudged Elastic Band", key = "om_optimize_k")]
  );
  omKMin @8 :Float64 = 0.1 $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "ini", section = "Nudged Elastic Band", key = "om_k_min")]
  );
  omKMax @9 :Float64 = 10.0 $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "ini", section = "Nudged Elastic Band", key = "om_k_max")]
  );
  elasticBandScale @10 :Float64 = 0.1 $Common.mutability(betweenStepsWithHook);
  restorePerpendicularSpringWhenNoCi @11 :Bool = false $Common.mutability(betweenStepsWithHook);
}

struct SpringPatch {
  kind @0 :OptSpringKind;
  constantEvPerAngstrom2 @1 :Common.OptFloat64;
  segmentWeights @2 :Common.OptFloat64List;
  ewKMinEvPerAngstrom2 @3 :Common.OptFloat64;
  ewKMaxEvPerAngstrom2 @4 :Common.OptFloat64;
  ewTriggerForceEvPerAngstrom @5 :Common.OptFloat64;
  omActionScale @6 :Common.OptFloat64;
  omOptimizeK @7 :Common.OptBool;
  omKMin @8 :Common.OptFloat64;
  omKMax @9 :Common.OptFloat64;
  elasticBandScale @10 :Common.OptFloat64;
  restorePerpendicularSpringWhenNoCi @11 :Common.OptBool;
}

enum PeakSelectionPolicy {
  globalMaxEnergy @0;
  allLocalMaxima @1;
  topKLocalMaxima @2;
}

struct OptPeakSelectionPolicy {
  union {
    unset @0 :Void;
    value @1 :PeakSelectionPolicy;
  }
}

struct ClimbingImageConfig {
  enabled @0 :Bool = true $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [
      (format = "ini", section = "Nudged Elastic Band", key = "climbing_image_method"),
      (format = "toml", section = "band", key = "climbing_image")
    ],
    cliFlags = ["--climbing-image"]
  );
  triggerForceEvPerAngstrom @1 :Float64 = 0.5 $Common.mutability(mutableLive) $Common.meta(
    dimension = force,
    unit = evPerAngstrom,
    bounds = (hasMinInclusive = true, minInclusive = 0.0),
    aliases = [
      (format = "ini", section = "Nudged Elastic Band", key = "ci_after"),
      (format = "toml", section = "band", key = "ci_activation_tol")
    ],
    cliFlags = ["--ci-activation-tol"]
  );
  triggerRelativeFactor @2 :Float64 = 0.8 $Common.mutability(mutableLive) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinInclusive = true, minInclusive = 0.0),
    aliases = [
      (format = "ini", section = "Nudged Elastic Band", key = "ci_after_rel"),
      (format = "toml", section = "band", key = "ci_activation_factor")
    ],
    cliFlags = ["--ci-activation-factor"]
  );
  armWhenEitherTriggerHolds @3 :Bool = true $Common.mutability(mutableLive);
  acceleratedSteepestStep @4 :Bool = false $Common.mutability(mutableLive);
  stepScale @5 :Float64 = 1.0 $Common.mutability(mutableLive);
  allowEndpointClimbing @6 :Bool = false $Common.mutability(mutableLive);
  peakPolicy @7 :PeakSelectionPolicy = globalMaxEnergy $Common.mutability(betweenStepsWithHook);
  maxClimbingPeaks @8 :UInt16 = 1 $Common.mutability(betweenStepsWithHook);
  convergedOnlyOnClimbingImage @9 :Bool = false $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "band", key = "climbing_image_converged_only")],
    cliFlags = ["--climbing-image-converged-only"]
  );
}

struct ClimbingImagePatch {
  enabled @0 :Common.OptBool;
  triggerForceEvPerAngstrom @1 :Common.OptFloat64;
  triggerRelativeFactor @2 :Common.OptFloat64;
  armWhenEitherTriggerHolds @3 :Common.OptBool;
  acceleratedSteepestStep @4 :Common.OptBool;
  stepScale @5 :Common.OptFloat64;
  allowEndpointClimbing @6 :Common.OptBool;
  peakPolicy @7 :OptPeakSelectionPolicy;
  maxClimbingPeaks @8 :Common.OptUInt16;
  convergedOnlyOnClimbingImage @9 :Common.OptBool;
}

enum PathInitMethod {
  linear @0;
  idpp @1;
  sidpp @2;
  filePath @3;
  nebPath @4;
  chemparseplot @5;
  internalCoords @6;
}

struct OptPathInitMethod {
  union {
    unset @0 :Void;
    value @1 :PathInitMethod;
  }
}

struct PathInitConfig {
  method @0 :PathInitMethod = linear $Common.mutability(restartOnly) $Common.meta(
    aliases = [
      (format = "ini", section = "Nudged Elastic Band", key = "initializer"),
      (format = "toml", section = "band", key = "idpp"),
      (format = "toml", section = "band", key = "sidpp")
    ],
    cliFlags = ["--idpp", "--sidpp"]
  );
  idppMaxIterations @1 :UInt32 = 300 $Common.mutability(restartOnly);
  idppForceTolerance @2 :Float64 = 0.05 $Common.mutability(restartOnly) $Common.meta(dimension = force, unit = evPerAngstrom);
  idppMaxMoveAngstrom @3 :Float64 = 0.05 $Common.mutability(restartOnly) $Common.meta(dimension = length, unit = angstrom);
  idppSpringConstant @4 :Float64 = 1.0 $Common.mutability(restartOnly) $Common.meta(dimension = forceConstant, unit = evPerAngstrom2);
  idppOptimizer @5 :Opt.OptimizerMethod = lbfgs $Common.mutability(restartOnly);
  sidppAlphaGrow @6 :Float64 = 0.5 $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "toml", section = "band", key = "sidpp_alpha")],
    cliFlags = ["--sidpp-alpha"]
  );
  sidppRefineFrames @7 :Bool = false $Common.mutability(restartOnly);
  sidppSpringConstant @8 :Float64 = 1.0 $Common.mutability(restartOnly);
  sidppGeodesic @9 :Bool = false $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "toml", section = "band", key = "sidpp_geodesic")],
    cliFlags = ["--sidpp-geodesic"]
  );
  geodesicImagePrior @10 :Bool = false $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "toml", section = "band", key = "geodesic_image_prior")],
    cliFlags = ["--gp-geodesic-image-prior"]
  );
  initialPathInput @11 :Text = "" $Common.mutability(restartOnly);
  useInternalCoords @12 :Bool = false $Common.mutability(restartOnly);
}

struct PathInitPatch {
  method @0 :OptPathInitMethod;
  idppMaxIterations @1 :Common.OptUInt32;
  idppForceTolerance @2 :Common.OptFloat64;
  idppMaxMoveAngstrom @3 :Common.OptFloat64;
  idppSpringConstant @4 :Common.OptFloat64;
  idppOptimizer @5 :Opt.OptOptimizerMethod;
  sidppAlphaGrow @6 :Common.OptFloat64;
  sidppRefineFrames @7 :Common.OptBool;
  sidppSpringConstant @8 :Common.OptFloat64;
  sidppGeodesic @9 :Common.OptBool;
  geodesicImagePrior @10 :Common.OptBool;
  initialPathInput @11 :Common.OptText;
  useInternalCoords @12 :Common.OptBool;
}

struct ZoomNebConfig {
  enabled @0 :Bool = false $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "ini", section = "Nudged Elastic Band", key = "zoom_neb")]
  );
  maxDepth @1 :UInt16 = 1 $Common.mutability(mutableLive);
  numSubImages @2 :UInt16 = 5 $Common.mutability(betweenStepsWithHook);
  windowAlpha @3 :Float64 = 0.5 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "Nudged Elastic Band", key = "zoom_alpha")]
  );
  triggerForceEvPerAngstrom @4 :Float64 = 0.5 $Common.mutability(mutableLive) $Common.meta(
    dimension = force,
    unit = evPerAngstrom,
    defaultKind = resolverDerived,
    resolverFunction = "resolve_zoom_trigger_from_band_tolerance"
  );
  triggerForceMultiplier @5 :Float64 = 10.0 $Common.mutability(mutableLive);
  preserveEndpoints @6 :Bool = false $Common.mutability(mutableLive);
  resetOptimizerOnZoom @7 :Bool = true $Common.mutability(mutableLive);
}

struct ZoomNebPatch {
  enabled @0 :Common.OptBool;
  maxDepth @1 :Common.OptUInt16;
  numSubImages @2 :Common.OptUInt16;
  windowAlpha @3 :Common.OptFloat64;
  triggerForceEvPerAngstrom @4 :Common.OptFloat64;
  triggerForceMultiplier @5 :Common.OptFloat64;
  preserveEndpoints @6 :Common.OptBool;
  resetOptimizerOnZoom @7 :Common.OptBool;
}

struct AutoNebConfig {
  enabled @0 :Bool = false $Common.mutability(restartOnly);
  kMaxImages @1 :UInt16 = 9 $Common.mutability(mutableLive);
  nSimulImages @2 :UInt16 = 3 $Common.mutability(betweenStepsWithHook);
  preliminaryForceTolEvPerAngstrom @3 :Float64 = 0.5 $Common.mutability(mutableLive) $Common.meta(dimension = force, unit = evPerAngstrom);
  finalForceTolEvPerAngstrom @4 :Float64 = 0.05 $Common.mutability(mutableLive) $Common.meta(dimension = force, unit = evPerAngstrom);
  springConstantEvPerAngstrom2 @5 :Float64 = 5.0 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = forceConstant, unit = evPerAngstrom2);
  maxCycles @6 :UInt32 = 10 $Common.mutability(mutableLive);
}

struct AutoNebPatch {
  enabled @0 :Common.OptBool;
  kMaxImages @1 :Common.OptUInt16;
  nSimulImages @2 :Common.OptUInt16;
  preliminaryForceTolEvPerAngstrom @3 :Common.OptFloat64;
  finalForceTolEvPerAngstrom @4 :Common.OptFloat64;
  springConstantEvPerAngstrom2 @5 :Common.OptFloat64;
  maxCycles @6 :Common.OptUInt32;
}

struct SolidStateBandConfig {
  enabled @0 :Bool = false $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "ini", section = "Nudged Elastic Band", key = "cell_neb")]
  );
  cellWeight @1 :Float64 = 1.0 $Common.mutability(betweenStepsWithHook);
  lengthsAnglesCell @2 :Bool = false $Common.mutability(restartOnly);
  externalPressureEvPerAngstrom3 @3 :Float64 = 0.0 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = pressureStress, unit = evPerAngstrom3);
  hydrostaticOnly @4 :Bool = false $Common.mutability(betweenStepsWithHook);
  fixVolume @5 :Bool = false $Common.mutability(betweenStepsWithHook);
  preconditionCell @6 :Bool = false $Common.mutability(betweenStepsWithHook);
}

struct SolidStateBandPatch {
  enabled @0 :Common.OptBool;
  cellWeight @1 :Common.OptFloat64;
  lengthsAnglesCell @2 :Common.OptBool;
  externalPressureEvPerAngstrom3 @3 :Common.OptFloat64;
  hydrostaticOnly @4 :Common.OptBool;
  fixVolume @5 :Common.OptBool;
  preconditionCell @6 :Common.OptBool;
}

struct BandConfig {
  methodKind @0 :BandMethodKind = nudgedElasticBand $Common.mutability(restartOnly);
  # Total images including both fixed endpoints (minimum 3: reactant + >=1 interior + product).
  # gpridge `band.nimages` (intermediate images) maps via `imageCount = nimages + 2`.
  imageCount @1 :UInt32 = 5 $Common.mutability(betweenStepsWithHook) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinInclusive = true, minInclusive = 3, hasMaxInclusive = true, maxInclusive = 1000),
    aliases = [
      (format = "ini", section = "Nudged Elastic Band", key = "images", transform = "total_from_intermediate_plus_2"),
      (format = "toml", section = "band", key = "nimages", transform = "total_from_intermediate_plus_2")
    ],
    cliFlags = ["--images", "--nimages"],
    defaultKind = profileDependent,
    resolverFunction = "resolve_band_image_count_from_profile"
  );
  optimizer @2 :Opt.OptimizerConfig;
  convergence @3 :Conv.ConvergenceConfig;
  tangent @4 :TangentKind = upwindEnergyWeighted $Common.mutability(betweenStepsWithHook);
  projection @5 :ProjectionKind = nudged $Common.mutability(betweenStepsWithHook);
  spring @6 :SpringConfig;
  climbingImage @7 :ClimbingImageConfig;
  pathInit @8 :PathInitConfig;
  zoom @9 :ZoomNebConfig;
  autoNeb @10 :AutoNebConfig;
  solidState @11 :SolidStateBandConfig;
  # Single authority for band refinement (handoff, OCI-NEB, and post-band saddle check).
  refine @12 :Refine.BandRefineConfig;
  minimizeEndpoints @13 :Bool = false $Common.mutability(restartOnly);
  removeTranslationRotation @14 :Bool = false $Common.mutability(betweenStepsWithHook);
  growingStringFrontierTol @15 :Float64 = 0.1 $Common.mutability(mutableLive);
}

struct BandPatch {
  methodKind @0 :OptBandMethodKind;
  imageCount @1 :Common.OptUInt32;
  optimizer @2 :Opt.OptimizerPatch;
  convergence @3 :Conv.ConvergencePatch;
  tangent @4 :OptTangentKind;
  projection @5 :OptProjectionKind;
  spring @6 :SpringPatch;
  climbingImage @7 :ClimbingImagePatch;
  pathInit @8 :PathInitPatch;
  zoom @9 :ZoomNebPatch;
  autoNeb @10 :AutoNebPatch;
  solidState @11 :SolidStateBandPatch;
  refine @12 :Refine.BandRefinePatch;
  minimizeEndpoints @13 :Common.OptBool;
  removeTranslationRotation @14 :Common.OptBool;
  growingStringFrontierTol @15 :Common.OptFloat64;
}
