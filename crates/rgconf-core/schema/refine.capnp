# Copyright 2023--present rgconf developers
@0xd782940f9a21b304;

using Common = import "common.capnp";
using Conv = import "convergence.capnp";
using Opt = import "optimizer.capnp";

# Context-specific refinement structs (with one owner per context):
#   - Minimization handoff lives on OptimizerConfig.handoff (in optimizer.capnp).
#   - Band refinement lives on BandConfig.refine (BandRefineConfig: handoff + ociNeb + saddleCheck).
#   - Single-ended saddle refinement lives on SaddleSearchConfig.refine (SaddleRefineConfig: handoff + prfo + saddleCheck).
# There is no duplicate top-level RgConf.refine field.

enum HessianUpdateScheme {
  bofill @0;
  sr1 @1;
  psb @2;
  bfgs @3;
  exactFiniteDifference @4;
}

struct OptHessianUpdateScheme {
  union {
    unset @0 :Void;
    value @1 :HessianUpdateScheme;
  }
}

struct PrfoRefineConfig {
  enabled @0 :Bool = false $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "Saddle Search", key = "refine_method")],
    cliFlags = ["--prfo-refine"]
  );
  triggerForceEvPerAngstrom @1 :Float64 = 0.15 $Common.mutability(mutableLive) $Common.meta(
    dimension = force,
    unit = evPerAngstrom,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [(format = "ini", section = "Saddle Search", key = "refine_after")]
  );
  stepToleranceAngstrom @2 :Float64 = 1e-4 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  gradientToleranceEvPerAngstrom @3 :Float64 = 1e-4 $Common.mutability(mutableLive) $Common.meta(dimension = force, unit = evPerAngstrom);
  trustRadiusInitialAngstrom @4 :Float64 = 0.1 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  trustRadiusMinAngstrom @5 :Float64 = 1e-4 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  trustRadiusMaxAngstrom @6 :Float64 = 0.5 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  hessianUpdate @7 :HessianUpdateScheme = bofill $Common.mutability(betweenStepsWithHook);
  recalcHessianEvery @8 :UInt32 = 0 $Common.mutability(mutableLive);
  targetSaddleIndex @9 :UInt16 = 1 $Common.mutability(betweenStepsWithHook);
  shiftFloor @10 :Float64 = 1e-4 $Common.mutability(mutableLive);
  maxIterations @11 :UInt32 = 100 $Common.mutability(mutableLive);
  finiteDifference @12 :Common.FiniteDifferenceConfig;
}

struct PrfoRefinePatch {
  enabled @0 :Common.OptBool;
  triggerForceEvPerAngstrom @1 :Common.OptFloat64;
  stepToleranceAngstrom @2 :Common.OptFloat64;
  gradientToleranceEvPerAngstrom @3 :Common.OptFloat64;
  trustRadiusInitialAngstrom @4 :Common.OptFloat64;
  trustRadiusMinAngstrom @5 :Common.OptFloat64;
  trustRadiusMaxAngstrom @6 :Common.OptFloat64;
  hessianUpdate @7 :OptHessianUpdateScheme;
  recalcHessianEvery @8 :Common.OptUInt32;
  targetSaddleIndex @9 :Common.OptUInt16;
  shiftFloor @10 :Common.OptFloat64;
  maxIterations @11 :Common.OptUInt32;
  finiteDifference @12 :Common.FiniteDifferencePatch;
}

enum OciNebMode {
  disabled @0;
  inlinePeakImage @1;
  postBandRefinement @2;
  hybridInlineAndPost @3;
}

struct OptOciNebMode {
  union {
    unset @0 :Void;
    value @1 :OciNebMode;
  }
}

enum OciMinModeKind {
  dimer @0;
  improvedDimer @1;
  lanczos @2;
  davidson @3;
}

struct OptOciMinModeKind {
  union {
    unset @0 :Void;
    value @1 :OciMinModeKind;
  }
}

struct OciNebRefineConfig {
  mode @0 :OciNebMode = disabled $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [
      (format = "ini", section = "Nudged Elastic Band", key = "ci_mmf"),
      (format = "toml", section = "band.mmf", key = "use_mmf")
    ],
    cliFlags = ["--use-mmf"]
  );
  triggerForceEvPerAngstrom @1 :Float64 = 0.1 $Common.mutability(mutableLive) $Common.meta(
    dimension = force,
    unit = evPerAngstrom,
    bounds = (hasMinInclusive = true, minInclusive = 0.0),
    aliases = [
      (format = "ini", section = "Nudged Elastic Band", key = "ci_mmf_after"),
      (format = "toml", section = "band.mmf", key = "trigger_force")
    ],
    cliFlags = ["--mmf-trigger-force"]
  );
  triggerRelativeFactor @2 :Float64 = 0.5 $Common.mutability(mutableLive) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinInclusive = true, minInclusive = 0.0),
    aliases = [
      (format = "ini", section = "Nudged Elastic Band", key = "ci_mmf_after_rel"),
      (format = "toml", section = "band.mmf", key = "trigger_factor")
    ],
    cliFlags = ["--mmf-trigger-factor"]
  );
  # Uses CountLimit (conditionDriven | limited(count)) rather than signed 0/-1 sentinels.
  maxSteps @3 :Common.CountLimit $Common.mutability(mutableLive) $Common.meta(
    aliases = [
      (format = "ini", section = "Nudged Elastic Band", key = "ci_mmf_nsteps"),
      (format = "toml", section = "band.mmf", key = "max_steps")
    ],
    cliFlags = ["--mmf-max-steps"]
  );
  angleToleranceCos @4 :Float64 = 0.7071 $Common.mutability(mutableLive) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinInclusive = true, minInclusive = -1.0, hasMaxInclusive = true, maxInclusive = 1.0),
    aliases = [
      (format = "ini", section = "Nudged Elastic Band", key = "ci_mmf_angle"),
      (format = "toml", section = "band.mmf", key = "angle_tol")
    ],
    cliFlags = ["--mmf-angle-tol"]
  );
  estimator @5 :OciMinModeKind = improvedDimer $Common.mutability(betweenStepsWithHook);
  divergedForceFactor @6 :Float64 = 10.0 $Common.mutability(mutableLive);
  lanczosRelTol @7 :Float64 = 0.01 $Common.mutability(mutableLive);
  lanczosMaxSteps @8 :UInt32 = 20 $Common.mutability(mutableLive);
  fallbackToStandardCiOnModeLoss @9 :Bool = true $Common.mutability(mutableLive);
  maxPhases @10 :Common.CountLimit $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "band.mmf", key = "max_phases")],
    cliFlags = ["--mmf-max-phases"]
  );
  maxInnerIterations @11 :Common.CountLimit $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "band.mmf", key = "max_inner_iter")],
    cliFlags = ["--mmf-max-inner-iter"]
  );
  maxRotationIterations @12 :UInt32 = 10 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "band.mmf", key = "max_rot_iter")],
    cliFlags = ["--mmf-max-rot-iter"]
  );
  evaluateImage1 @13 :Bool = true $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "band.mmf", key = "eval_image1")],
    cliFlags = ["--mmf-eval-image1"]
  );
  perturbationAngstrom @14 :Float64 = 0.05 $Common.mutability(mutableLive) $Common.meta(
    dimension = length,
    unit = angstrom,
    aliases = [(format = "toml", section = "band.mmf", key = "perturb")],
    cliFlags = ["--mmf-perturb"]
  );
  keepImproving @15 :Bool = true $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "band.mmf", key = "keep_improving")],
    cliFlags = ["--mmf-keep-improving"]
  );
  imspeSoftThreshold @16 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "band.mmf", key = "imspe_soft_threshold")],
    cliFlags = ["--mmf-imspe-soft-threshold"]
  );
}

struct OciNebRefinePatch {
  mode @0 :OptOciNebMode;
  triggerForceEvPerAngstrom @1 :Common.OptFloat64;
  triggerRelativeFactor @2 :Common.OptFloat64;
  hasMaxSteps @3 :Bool = false;
  maxSteps @4 :Common.CountLimit;
  angleToleranceCos @5 :Common.OptFloat64;
  estimator @6 :OptOciMinModeKind;
  divergedForceFactor @7 :Common.OptFloat64;
  lanczosRelTol @8 :Common.OptFloat64;
  lanczosMaxSteps @9 :Common.OptUInt32;
  fallbackToStandardCiOnModeLoss @10 :Common.OptBool;
  hasMaxPhases @11 :Bool = false;
  maxPhases @12 :Common.CountLimit;
  hasMaxInnerIterations @13 :Bool = false;
  maxInnerIterations @14 :Common.CountLimit;
  maxRotationIterations @15 :Common.OptUInt32;
  evaluateImage1 @16 :Common.OptBool;
  perturbationAngstrom @17 :Common.OptFloat64;
  keepImproving @18 :Common.OptBool;
  imspeSoftThreshold @19 :Common.OptFloat64;
}

struct SaddleIndexCheckConfig {
  enabled @0 :Bool = false $Common.mutability(mutableLive);
  expectedIndex @1 :UInt16 = 1 $Common.mutability(mutableLive);
  numModesToCompute @2 :UInt16 = 2 $Common.mutability(mutableLive);
  krylovTolerance @3 :Float64 = 1e-3 $Common.mutability(mutableLive);
  maxKrylovIterations @4 :UInt32 = 100 $Common.mutability(mutableLive);
  zeroEigenvalueCutoff @5 :Float64 = 1e-3 $Common.mutability(mutableLive);
  finiteDifference @6 :Common.FiniteDifferenceConfig;
  maxRecoveryCycles @7 :UInt32 = 0 $Common.mutability(mutableLive);
  pushStepAngstrom @8 :Float64 = 0.05 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  abortOnMismatch @9 :Bool = false $Common.mutability(mutableLive);
}

struct SaddleIndexCheckPatch {
  enabled @0 :Common.OptBool;
  expectedIndex @1 :Common.OptUInt16;
  numModesToCompute @2 :Common.OptUInt16;
  krylovTolerance @3 :Common.OptFloat64;
  maxKrylovIterations @4 :Common.OptUInt32;
  zeroEigenvalueCutoff @5 :Common.OptFloat64;
  finiteDifference @6 :Common.FiniteDifferencePatch;
  maxRecoveryCycles @7 :Common.OptUInt32;
  pushStepAngstrom @8 :Common.OptFloat64;
  abortOnMismatch @9 :Common.OptBool;
}

# Owned exclusively by BandConfig.refine.
struct BandRefineConfig {
  handoff @0 :Opt.OptimizerHandoffConfig;
  ociNeb @1 :OciNebRefineConfig;
  saddleCheck @2 :SaddleIndexCheckConfig;
}

struct BandRefinePatch {
  handoff @0 :Opt.OptimizerHandoffPatch;
  ociNeb @1 :OciNebRefinePatch;
  saddleCheck @2 :SaddleIndexCheckPatch;
}

# Owned exclusively by SaddleSearchConfig.refine.
struct SaddleRefineConfig {
  handoff @0 :Opt.OptimizerHandoffConfig;
  prfo @1 :PrfoRefineConfig;
  saddleCheck @2 :SaddleIndexCheckConfig;
}

struct SaddleRefinePatch {
  handoff @0 :Opt.OptimizerHandoffPatch;
  prfo @1 :PrfoRefinePatch;
  saddleCheck @2 :SaddleIndexCheckPatch;
}
