# Copyright 2023--present rgconf developers
@0xd782940f9a21b303;

using Common = import "common.capnp";
using Conv = import "convergence.capnp";

# Unified first-order, quasi-Newton, trust-region, manifold, and constrained
# optimization schema (resolved scalar structs + presence-aware patch structs).

enum OptimizerMethod {
  lbfgs @0;
  fire @1;
  fire2 @2;
  cg @3;
  quickMin @4;
  steepestDescent @5;
  ode12r @6;
  rtr @7;
  samm @8;
  lbfgsLineSearch @9;
  spectralCg @10;
  boxLbfgsB @11;
  projectedGradient @12;
  highsLpQp @13;
  prfo @14;
}

struct OptOptimizerMethod {
  union {
    unset @0 :Void;
    value @1 :OptimizerMethod;
  }
}

enum OptimizerBackend {
  rgmin @0;
  rgsaddle @1;
  eonLegacy @2;
}

struct OptOptimizerBackend {
  union {
    unset @0 :Void;
    value @1 :OptimizerBackend;
  }
}

enum StepAcceptPolicy {
  unconditional @0;
  monotoneEnergy @1;
  nonmonotoneHistory @2;
  lineSearchWolfe @3;
  trustRegion @4;
  dimerCurvatureTrust @5;
}

struct OptStepAcceptPolicy {
  union {
    unset @0 :Void;
    value @1 :StepAcceptPolicy;
  }
}

enum LbfgsCurvaturePolicy {
  resetOnNegative @0;
  skipUpdate @1;
  powellDamped @2;
  liFukushimaCautious @3;
}

struct OptLbfgsCurvaturePolicy {
  union {
    unset @0 :Void;
    value @1 :LbfgsCurvaturePolicy;
  }
}

enum LbfgsH0Scaling {
  syOverYy @0;
  fixedInverseCurvature @1;
  bb1SyOverSs @2;
  spectralGeometricMean @3;
}

struct OptLbfgsH0Scaling {
  union {
    unset @0 :Void;
    value @1 :LbfgsH0Scaling;
  }
}

enum LbfgsSecantKind {
  standard @0;
  modifiedZhangXu @1;
  biggsCubic @2;
}

struct OptLbfgsSecantKind {
  union {
    unset @0 :Void;
    value @1 :LbfgsSecantKind;
  }
}

struct LbfgsConfig {
  memory @0 :UInt32 = 20 $Common.mutability(betweenStepsWithHook) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinInclusive = true, minInclusive = 1, hasMaxInclusive = true, maxInclusive = 500),
    aliases = [
      (format = "ini", section = "LBFGS", key = "lbfgs_memory"),
      (format = "ini", section = "Optimizer", key = "lbfgs_memory"),
      (format = "toml", section = "band.lbfgs", key = "memory")
    ],
    cliFlags = ["--lbfgs-memory", "--neb-lbfgs-memory"]
  );
  inverseCurvature @1 :Float64 = 0.01 $Common.mutability(betweenStepsWithHook) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [
      (format = "ini", section = "LBFGS", key = "lbfgs_inverse_curvature"),
      (format = "toml", section = "band.lbfgs", key = "inverse_curvature")
    ],
    cliFlags = ["--neb-lbfgs-inverse-curvature"]
  );
  maxInverseCurvature @2 :Float64 = 0.0 $Common.mutability(betweenStepsWithHook) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinInclusive = true, minInclusive = 0.0),
    aliases = [(format = "ini", section = "LBFGS", key = "lbfgs_max_inverse_curvature")]
  );
  autoScale @3 :Bool = true $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [
      (format = "ini", section = "LBFGS", key = "lbfgs_auto_scale"),
      (format = "toml", section = "band.lbfgs", key = "auto_scale")
    ],
    cliFlags = ["--neb-lbfgs-auto-scale"]
  );
  angleReset @4 :Bool = true $Common.mutability(mutableLive) $Common.meta(
    aliases = [
      (format = "ini", section = "LBFGS", key = "lbfgs_angle_reset"),
      (format = "toml", section = "band.lbfgs", key = "angle_reset")
    ],
    cliFlags = ["--neb-lbfgs-angle-reset"]
  );
  distanceReset @5 :Bool = true $Common.mutability(mutableLive) $Common.meta(
    aliases = [
      (format = "ini", section = "LBFGS", key = "lbfgs_distance_reset"),
      (format = "toml", section = "band.lbfgs", key = "distance_reset")
    ],
    cliFlags = ["--neb-lbfgs-distance-reset"]
  );
  curvaturePolicy @6 :LbfgsCurvaturePolicy = resetOnNegative $Common.mutability(betweenStepsWithHook);
  h0Scaling @7 :LbfgsH0Scaling = syOverYy $Common.mutability(betweenStepsWithHook);
  secantKind @8 :LbfgsSecantKind = standard $Common.mutability(betweenStepsWithHook);
  projectRigidModes @9 :Bool = false $Common.mutability(betweenStepsWithHook);
  cautiousEpsilon @10 :Float64 = 1e-6 $Common.mutability(mutableLive) $Common.meta(bounds = (hasMinExclusive = true, minExclusive = 0.0));
  cautiousAlpha @11 :Float64 = 0.01 $Common.mutability(mutableLive) $Common.meta(bounds = (hasMinExclusive = true, minExclusive = 0.0));
  powellDampingTheta @12 :Float64 = 0.2 $Common.mutability(mutableLive) $Common.meta(bounds = (hasMinExclusive = true, minExclusive = 0.0, hasMaxExclusive = true, maxExclusive = 1.0));
  finiteDifferenceStep @13 :Float64 = 1e-6 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "band.lbfgs", key = "finite_difference")],
    cliFlags = ["--neb-lbfgs-finite-difference"]
  );
  sketchHistory @14 :UInt32 = 0 $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "toml", section = "band.lbfgs", key = "sketch_history")],
    cliFlags = ["--neb-lbfgs-sketch-history"]
  );
  sketchSeed @15 :UInt64 = 1 $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "toml", section = "band.lbfgs", key = "sketch_seed")],
    cliFlags = ["--neb-lbfgs-sketch-seed"]
  );
  sketchH0Rank @16 :UInt32 = 0 $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "toml", section = "band.lbfgs", key = "sketch_h0_rank")],
    cliFlags = ["--neb-lbfgs-sketch-h0-rank"]
  );
  sketchH0Epsilon @17 :Float64 = 1e-3 $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "toml", section = "band.lbfgs", key = "sketch_h0_eps")],
    cliFlags = ["--neb-lbfgs-sketch-h0-eps"]
  );
}

struct LbfgsPatch {
  memory @0 :Common.OptUInt32;
  inverseCurvature @1 :Common.OptFloat64;
  maxInverseCurvature @2 :Common.OptFloat64;
  autoScale @3 :Common.OptBool;
  angleReset @4 :Common.OptBool;
  distanceReset @5 :Common.OptBool;
  curvaturePolicy @6 :OptLbfgsCurvaturePolicy;
  h0Scaling @7 :OptLbfgsH0Scaling;
  secantKind @8 :OptLbfgsSecantKind;
  projectRigidModes @9 :Common.OptBool;
  cautiousEpsilon @10 :Common.OptFloat64;
  cautiousAlpha @11 :Common.OptFloat64;
  powellDampingTheta @12 :Common.OptFloat64;
  finiteDifferenceStep @13 :Common.OptFloat64;
  sketchHistory @14 :Common.OptUInt32;
  sketchSeed @15 :Common.OptUInt64;
  sketchH0Rank @16 :Common.OptUInt32;
  sketchH0Epsilon @17 :Common.OptFloat64;
}

enum FireVariant {
  bitzekStandard @0;
  guenoleFire2 @1;
  variableMetric @2;
}

struct OptFireVariant {
  union {
    unset @0 :Void;
    value @1 :FireVariant;
  }
}

enum FireIntegrator {
  semiImplicitEuler @0;
  velocityVerlet @1;
  explicitEuler @2;
}

struct OptFireIntegrator {
  union {
    unset @0 :Void;
    value @1 :FireIntegrator;
  }
}

struct FireConfig {
  timeStepFs @0 :Float64 = 1.0 $Common.mutability(mutableLive) $Common.meta(
    dimension = time,
    unit = femtosecond,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [
      (format = "ini", section = "Optimizer", key = "time_step"),
      (format = "ini", section = "FIRE", key = "time_step")
    ],
    cliFlags = ["--fire-dt"]
  );
  maxTimeStepFs @1 :Float64 = 2.5 $Common.mutability(mutableLive) $Common.meta(
    dimension = time,
    unit = femtosecond,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [
      (format = "ini", section = "Optimizer", key = "max_time_step"),
      (format = "ini", section = "FIRE", key = "time_step_max")
    ],
    cliFlags = ["--fire-dt-max"]
  );
  minTimeStepFs @2 :Float64 = 0.02 $Common.mutability(mutableLive) $Common.meta(dimension = time, unit = femtosecond);
  eonTimeUnitScaleFs @3 :Float64 = 10.1805055 $Common.mutability(restartOnly) $Common.meta(dimension = time, unit = femtosecond);
  variant @4 :FireVariant = bitzekStandard $Common.mutability(betweenStepsWithHook);
  integrator @5 :FireIntegrator = semiImplicitEuler $Common.mutability(betweenStepsWithHook);
  alphaStart @6 :Float64 = 0.1 $Common.mutability(mutableLive);
  fAlpha @7 :Float64 = 0.99 $Common.mutability(mutableLive);
  fInc @8 :Float64 = 1.1 $Common.mutability(mutableLive);
  fDec @9 :Float64 = 0.5 $Common.mutability(mutableLive);
  nMin @10 :UInt32 = 5 $Common.mutability(mutableLive);
  maxBacktracks @11 :UInt32 = 20 $Common.mutability(mutableLive);
  abcFire @12 :Bool = false $Common.mutability(betweenStepsWithHook);
  capTwoStepOvershoot @13 :Bool = false $Common.mutability(mutableLive);
}

struct FirePatch {
  timeStepFs @0 :Common.OptFloat64;
  maxTimeStepFs @1 :Common.OptFloat64;
  minTimeStepFs @2 :Common.OptFloat64;
  eonTimeUnitScaleFs @3 :Common.OptFloat64;
  variant @4 :OptFireVariant;
  integrator @5 :OptFireIntegrator;
  alphaStart @6 :Common.OptFloat64;
  fAlpha @7 :Common.OptFloat64;
  fInc @8 :Common.OptFloat64;
  fDec @9 :Common.OptFloat64;
  nMin @10 :Common.OptUInt32;
  maxBacktracks @11 :Common.OptUInt32;
  abcFire @12 :Common.OptBool;
  capTwoStepOvershoot @13 :Common.OptBool;
}

enum CgUpdateFormula {
  polakRibierePolyak @0;
  fletcherReeves @1;
  hestenesStiefel @2;
  hagerZhang @3;
  liuStorey @4;
  daiYuan @5;
}

struct OptCgUpdateFormula {
  union {
    unset @0 :Void;
    value @1 :CgUpdateFormula;
  }
}

struct CgConfig {
  formula @0 :CgUpdateFormula = polakRibierePolyak $Common.mutability(betweenStepsWithHook) $Common.meta(applicability = "optimizer.method == cg");
  noOvershooting @1 :Bool = false $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "CG", key = "cg_no_overshooting")],
    applicability = "optimizer.method == cg"
  );
  knockOutMaxMove @2 :Bool = false $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "CG", key = "cg_knock_out_max_move")],
    applicability = "optimizer.method == cg"
  );
  lineSearch @3 :Bool = false $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "ini", section = "CG", key = "cg_line_search")],
    applicability = "optimizer.method == cg"
  );
  lineConverged @4 :Float64 = 0.1 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "CG", key = "cg_line_converged")],
    applicability = "optimizer.method == cg"
  );
  maxIterLineSearch @5 :UInt32 = 10 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "CG", key = "cg_max_iter_line_search")],
    applicability = "optimizer.method == cg"
  );
  maxIterBeforeReset @6 :UInt32 = 0 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "CG", key = "cg_max_iter_before_reset")],
    applicability = "optimizer.method == cg"
  );
  restartThreshold @7 :Float64 = 0.1 $Common.mutability(mutableLive) $Common.meta(applicability = "optimizer.method == cg");
}

struct CgPatch {
  formula @0 :OptCgUpdateFormula;
  noOvershooting @1 :Common.OptBool;
  knockOutMaxMove @2 :Common.OptBool;
  lineSearch @3 :Common.OptBool;
  lineConverged @4 :Common.OptFloat64;
  maxIterLineSearch @5 :Common.OptUInt32;
  maxIterBeforeReset @6 :Common.OptUInt32;
  restartThreshold @7 :Common.OptFloat64;
}

enum QuickMinVariant {
  eonFixedStep @0;
  rgminAdaptiveStep @1;
}

struct OptQuickMinVariant {
  union {
    unset @0 :Void;
    value @1 :QuickMinVariant;
  }
}

struct QuickMinConfig {
  variant @0 :QuickMinVariant = eonFixedStep $Common.mutability(betweenStepsWithHook) $Common.meta(applicability = "optimizer.method == quickMin");
  steepestDescentFallback @1 :Bool = false $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "QuickMin", key = "qm_steepest_descent")],
    applicability = "optimizer.method == quickMin"
  );
  timeStepFs @2 :Float64 = 1.0 $Common.mutability(mutableLive) $Common.meta(dimension = time, unit = femtosecond, applicability = "optimizer.method == quickMin");
  maxTimeStepFs @3 :Float64 = 2.5 $Common.mutability(mutableLive) $Common.meta(dimension = time, unit = femtosecond, applicability = "optimizer.method == quickMin");
}

struct QuickMinPatch {
  variant @0 :OptQuickMinVariant;
  steepestDescentFallback @1 :Common.OptBool;
  timeStepFs @2 :Common.OptFloat64;
  maxTimeStepFs @3 :Common.OptFloat64;
}

struct SdConfig {
  alpha @0 :Float64 = 0.1 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "SD", key = "sd_alpha")],
    applicability = "optimizer.method == steepestDescent"
  );
  twoPointBarzilaiBorwein @1 :Bool = false $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "ini", section = "SD", key = "sd_two_point")],
    applicability = "optimizer.method == steepestDescent"
  );
}

struct SdPatch {
  alpha @0 :Common.OptFloat64;
  twoPointBarzilaiBorwein @1 :Common.OptBool;
}

struct Ode12rConfig {
  rtol @0 :Float64 = 1e-2 $Common.mutability(mutableLive);
  atol @1 :Float64 = 1e-4 $Common.mutability(mutableLive);
  initialStep @2 :Float64 = 0.1 $Common.mutability(mutableLive);
  minStep @3 :Float64 = 1e-6 $Common.mutability(mutableLive);
  maxStep @4 :Float64 = 1.0 $Common.mutability(mutableLive);
}

struct Ode12rPatch {
  rtol @0 :Common.OptFloat64;
  atol @1 :Common.OptFloat64;
  initialStep @2 :Common.OptFloat64;
  minStep @3 :Common.OptFloat64;
  maxStep @4 :Common.OptFloat64;
}

struct RtrConfig {
  initialDelta @0 :Float64 = 0.2 $Common.mutability(mutableLive);
  maxDelta @1 :Float64 = 1.0 $Common.mutability(mutableLive);
  eta1 @2 :Float64 = 0.1 $Common.mutability(mutableLive);
  eta2 @3 :Float64 = 0.75 $Common.mutability(mutableLive);
  gamma1 @4 :Float64 = 0.25 $Common.mutability(mutableLive);
  gamma2 @5 :Float64 = 2.0 $Common.mutability(mutableLive);
  maxInnerCgIters @6 :UInt32 = 30 $Common.mutability(mutableLive);
  theta @7 :Float64 = 1.0 $Common.mutability(mutableLive);
  kappa @8 :Float64 = 0.1 $Common.mutability(mutableLive);
}

struct RtrPatch {
  initialDelta @0 :Common.OptFloat64;
  maxDelta @1 :Common.OptFloat64;
  eta1 @2 :Common.OptFloat64;
  eta2 @3 :Common.OptFloat64;
  gamma1 @4 :Common.OptFloat64;
  gamma2 @5 :Common.OptFloat64;
  maxInnerCgIters @6 :Common.OptUInt32;
  theta @7 :Common.OptFloat64;
  kappa @8 :Common.OptFloat64;
}

struct SammConfig {
  subspaceDim @0 :UInt32 = 8 $Common.mutability(betweenStepsWithHook);
  initialTrustRadius @1 :Float64 = 0.2 $Common.mutability(mutableLive);
  dampingMu @2 :Float64 = 1e-3 $Common.mutability(mutableLive);
}

struct SammPatch {
  subspaceDim @0 :Common.OptUInt32;
  initialTrustRadius @1 :Common.OptFloat64;
  dampingMu @2 :Common.OptFloat64;
}

struct ScgConfig {
  sigma0 @0 :Float64 = 1e-4 $Common.mutability(mutableLive);
  lambda0 @1 :Float64 = 1e-6 $Common.mutability(mutableLive);
  lambdaBar @2 :Float64 = 0.0 $Common.mutability(mutableLive);
  maxStepBounds @3 :Float64 = 10.0 $Common.mutability(mutableLive);
}

struct ScgPatch {
  sigma0 @0 :Common.OptFloat64;
  lambda0 @1 :Common.OptFloat64;
  lambdaBar @2 :Common.OptFloat64;
  maxStepBounds @3 :Common.OptFloat64;
}

enum PreconKind {
  none @0;
  packwoodExp @1;
  morse @2;
  lindh @3;
  vdw @4;
  sparseCholesky @5;
  diagonalInverse @6;
  ssnebCell @7;
}

struct OptPreconKind {
  union {
    unset @0 :Void;
    value @1 :PreconKind;
  }
}

struct PackwoodExpPreconConfig {
  coeffA @0 :Float64 = 3.0 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = dimensionless, unit = none, bounds = (hasMinExclusive = true, minExclusive = 0.0));
  cutoffScaleRcut @1 :Float64 = 2.0 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = dimensionless, unit = none, bounds = (hasMinExclusive = true, minExclusive = 1.0));
  stabilizationMu @2 :Float64 = 1.0 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = forceConstant, unit = evPerAngstrom2, bounds = (hasMinExclusive = true, minExclusive = 0.0));
  rebuildMoveThresholdAngstrom @3 :Float64 = 0.5 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  autoRebuild @4 :Bool = true $Common.mutability(mutableLive);
  energyWeighted @5 :Bool = false $Common.mutability(betweenStepsWithHook);
  minCutoffAngstrom @6 :Float64 = 2.5 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = length, unit = angstrom);
}

struct PackwoodExpPreconPatch {
  coeffA @0 :Common.OptFloat64;
  cutoffScaleRcut @1 :Common.OptFloat64;
  stabilizationMu @2 :Common.OptFloat64;
  rebuildMoveThresholdAngstrom @3 :Common.OptFloat64;
  autoRebuild @4 :Common.OptBool;
  energyWeighted @5 :Common.OptBool;
  minCutoffAngstrom @6 :Common.OptFloat64;
}

enum ManifoldKind {
  euclidean @0;
  rigidQuotient @1;
  massWeightedRigid @2;
  unitSphere @3;
  so3Rotation @4;
  stiefelOrthogonal @5;
  se3RigidBody @6;
  hyperplaneProjected @7;
}

struct OptManifoldKind {
  union {
    unset @0 :Void;
    value @1 :ManifoldKind;
  }
}

struct ManifoldConfig {
  kind @0 :ManifoldKind = euclidean $Common.mutability(restartOnly);
  atomicMassesAmu @1 :List(Float64) $Common.mutability(restartOnly) $Common.meta(dimension = mass, unit = amu);
  stiefelRows @2 :UInt32 = 0 $Common.mutability(restartOnly);
  stiefelCols @3 :UInt32 = 0 $Common.mutability(restartOnly);
  se3RotationScale @4 :Float64 = 1.0 $Common.mutability(betweenStepsWithHook);
  cell @5 :Common.CellConfig $Common.mutability(betweenStepsWithHook);
}

struct ManifoldPatch {
  kind @0 :OptManifoldKind;
  atomicMassesAmu @1 :Common.OptFloat64List;
  stiefelRows @2 :Common.OptUInt32;
  stiefelCols @3 :Common.OptUInt32;
  se3RotationScale @4 :Common.OptFloat64;
  cell @5 :Common.CellPatch;
}

struct BondConstraint {
  atomI @0 :UInt32;
  atomJ @1 :UInt32;
  targetLengthAngstrom @2 :Float64 $Common.meta(dimension = length, unit = angstrom);
  springConstantEvPerAngstrom2 @3 :Float64 = 0.0 $Common.meta(dimension = forceConstant, unit = evPerAngstrom2);
}

struct ConstraintConfig {
  boxLowerBoundsAngstrom @0 :List(Float64) $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = length, unit = angstrom);
  boxUpperBoundsAngstrom @1 :List(Float64) $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = length, unit = angstrom);
  fixedAtomIndices @2 :List(UInt32) $Common.mutability(restartOnly);
  bondConstraints @3 :List(BondConstraint) $Common.mutability(betweenStepsWithHook);
  linearEqualityMatrix @4 :List(Float64) $Common.mutability(restartOnly);
  linearEqualityRhs @5 :List(Float64) $Common.mutability(betweenStepsWithHook);
  highsSimplexStrategy @6 :UInt8 = 1 $Common.mutability(mutableLive);
  highsPresolve @7 :Bool = true $Common.mutability(mutableLive);
  eindirPenaltyWeight @8 :Float64 = 10.0 $Common.mutability(betweenStepsWithHook);
}

struct ConstraintPatch {
  boxLowerBoundsAngstrom @0 :Common.OptFloat64List;
  boxUpperBoundsAngstrom @1 :Common.OptFloat64List;
  fixedAtomIndices @2 :Common.OptUInt32List;
  bondConstraints @3 :List(BondConstraint);
  hasBondConstraints @4 :Bool = false;
  linearEqualityMatrix @5 :Common.OptFloat64List;
  linearEqualityRhs @6 :Common.OptFloat64List;
  highsSimplexStrategy @7 :Common.OptUInt8;
  highsPresolve @8 :Common.OptBool;
  eindirPenaltyWeight @9 :Common.OptFloat64;
}

struct OptimizerPatch {
  method @0 :OptOptimizerMethod;
  backend @1 :OptOptimizerBackend;
  maxAtomMoveAngstrom @2 :Common.OptFloat64;
  maxEuclideanMoveAngstrom @3 :Common.OptFloat64;
  convergence @4 :Conv.ConvergencePatch;
  acceptPolicy @5 :OptStepAcceptPolicy;
  nonmonotoneWindow @6 :Common.OptUInt32;
  preconKind @7 :OptPreconKind;
  precon @8 :PackwoodExpPreconPatch;
  manifold @9 :ManifoldPatch;
  constraints @10 :ConstraintPatch;
  lbfgs @11 :LbfgsPatch;
  fire @12 :FirePatch;
  cg @13 :CgPatch;
  quickMin @14 :QuickMinPatch;
  sd @15 :SdPatch;
  ode12r @16 :Ode12rPatch;
  rtr @17 :RtrPatch;
  samm @18 :SammPatch;
  scg @19 :ScgPatch;
}

# Two-phase first-order optimizer handoff ([Refine] opt_method + threshold in eOn).
# Secondary optimizer overrides are stored as an OptimizerPatch (presence-aware),
# not a raw default-filled struct.
struct OptimizerHandoffConfig {
  enabled @0 :Bool = false $Common.mutability(mutableLive);
  secondaryMethod @1 :OptimizerMethod = fire $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "ini", section = "Refine", key = "opt_method")],
    cliFlags = ["--refine-method"]
  );
  forceThresholdEvPerAngstrom @2 :Float64 = 0.5 $Common.mutability(mutableLive) $Common.meta(
    dimension = force,
    unit = evPerAngstrom,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [(format = "ini", section = "Refine", key = "threshold")],
    cliFlags = ["--refine-threshold"]
  );
  thresholdNorm @3 :Conv.ForceNorm = maxAtomL2 $Common.mutability(mutableLive);
  resetHistoryOnSwitch @4 :Bool = true $Common.mutability(mutableLive);
  allowRevertAboveHysteresisFactor @5 :Float64 = 0.0 $Common.mutability(mutableLive);
  secondaryOptimizerPatch @6 :OptimizerPatch;
}

struct OptimizerHandoffPatch {
  enabled @0 :Common.OptBool;
  secondaryMethod @1 :OptOptimizerMethod;
  forceThresholdEvPerAngstrom @2 :Common.OptFloat64;
  thresholdNorm @3 :Conv.OptForceNorm;
  resetHistoryOnSwitch @4 :Common.OptBool;
  allowRevertAboveHysteresisFactor @5 :Common.OptFloat64;
  secondaryOptimizerPatch @6 :OptimizerPatch;
}

struct OptimizerConfig {
  method @0 :OptimizerMethod = lbfgs $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "ini", section = "Optimizer", key = "opt_method")],
    cliFlags = ["--opt-method"]
  );
  backend @1 :OptimizerBackend = rgmin $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "ini", section = "Optimizer", key = "opt_backend")],
    cliFlags = ["--opt-backend"]
  );
  maxAtomMoveAngstrom @2 :Float64 = 0.2 $Common.mutability(mutableLive) $Common.meta(
    dimension = length,
    unit = angstrom,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [
      (format = "ini", section = "Optimizer", key = "max_move"),
      (format = "toml", section = "band.lbfgs", key = "max_move")
    ],
    cliFlags = ["--max-move", "--neb-lbfgs-max-move"]
  );
  maxEuclideanMoveAngstrom @3 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(
    dimension = length,
    unit = angstrom,
    bounds = (hasMinInclusive = true, minInclusive = 0.0)
  );
  convergence @4 :Conv.ConvergenceConfig;
  acceptPolicy @5 :StepAcceptPolicy = unconditional $Common.mutability(betweenStepsWithHook);
  nonmonotoneWindow @6 :UInt32 = 1 $Common.mutability(mutableLive);
  preconKind @7 :PreconKind = none $Common.mutability(betweenStepsWithHook);
  precon @8 :PackwoodExpPreconConfig;
  manifold @9 :ManifoldConfig;
  constraints @10 :ConstraintConfig;
  lbfgs @11 :LbfgsConfig;
  fire @12 :FireConfig;
  cg @13 :CgConfig;
  quickMin @14 :QuickMinConfig;
  sd @15 :SdConfig;
  ode12r @16 :Ode12rConfig;
  rtr @17 :RtrConfig;
  samm @18 :SammConfig;
  scg @19 :ScgConfig;
  # Single authority for two-phase minimization handoff (no duplicate RgConf.refine).
  handoff @20 :OptimizerHandoffConfig;
}
