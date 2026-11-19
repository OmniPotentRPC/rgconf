# Copyright 2023--present rgconf developers
@0xd782940f9a21b306;

using Common = import "common.capnp";
using Conv = import "convergence.capnp";
using Opt = import "optimizer.capnp";
using Refine = import "refine.capnp";
using Surrogate = import "surrogate.capnp";

# Unified minimum-mode eigenmode estimation and single-ended saddle search schema
# (resolved scalar structs + presence-aware patches).
# GprDimerConfig reuses Surrogate.GpConfig and Surrogate.LinearAlgebraConfig rather
# than duplicating GP hyperparameter fitting controls.

enum MinModeEstimatorKind {
  dimer @0;
  improvedDimer @1;
  lanczos @2;
  davidson @3;
  artn @4;
  sella @5;
  gprDimer @6;
  exactHessian @7;
}

struct OptMinModeEstimatorKind {
  union {
    unset @0 :Void;
    value @1 :MinModeEstimatorKind;
  }
}

enum DimerRotationBackend {
  classicalFourier @0;
  lanczosSubspace @1;
  davidsonSubspace @2;
  locallyOptimalRiccati @3;
}

struct OptDimerRotationBackend {
  union {
    unset @0 :Void;
    value @1 :DimerRotationBackend;
  }
}

struct DimerConfig {
  improved @0 :Bool = true $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "ini", section = "Dimer", key = "improved")]
  );
  rotationBackend @1 :DimerRotationBackend = classicalFourier $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "ini", section = "Dimer", key = "rotation_backend")]
  );
  rotationOptimizer @2 :Opt.OptimizerMethod = cg $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "ini", section = "Dimer", key = "opt_method")]
  );
  finiteDifference @3 :Common.FiniteDifferenceConfig;
  finiteAngleRad @4 :Float64 = 0.005 $Common.mutability(mutableLive) $Common.meta(
    dimension = angle,
    unit = radian,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [(format = "ini", section = "Dimer", key = "finite_angle")]
  );
  convergedAngleDeg @5 :Float64 = 5.0 $Common.mutability(mutableLive) $Common.meta(
    dimension = angle,
    unit = degree,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [(format = "ini", section = "Dimer", key = "converged_angle")]
  );
  torqueMinEvPerAngstrom @6 :Float64 = 0.1 $Common.mutability(mutableLive) $Common.meta(
    dimension = force,
    unit = evPerAngstrom,
    aliases = [(format = "ini", section = "Dimer", key = "torque_min")]
  );
  torqueMaxEvPerAngstrom @7 :Float64 = 1.0 $Common.mutability(mutableLive) $Common.meta(
    dimension = force,
    unit = evPerAngstrom,
    aliases = [(format = "ini", section = "Dimer", key = "torque_max")]
  );
  rotationsMin @8 :UInt32 = 1 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "Dimer", key = "rotations_min")]
  );
  rotationsMax @9 :UInt32 = 10 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "Dimer", key = "rotations_max")]
  );
  removeRotation @10 :Bool = false $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "ini", section = "Dimer", key = "remove_rotation")]
  );
  lorResidualTol @11 :Float64 = 0.1 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "Dimer", key = "lor_residual_tol")]
  );
  modeLossCosineTol @12 :Float64 = 0.7071 $Common.mutability(mutableLive);
  reseedRotationHistoryOnPullback @13 :Bool = true $Common.mutability(mutableLive);
}

struct DimerPatch {
  improved @0 :Common.OptBool;
  rotationBackend @1 :OptDimerRotationBackend;
  rotationOptimizer @2 :Opt.OptOptimizerMethod;
  finiteDifference @3 :Common.FiniteDifferencePatch;
  finiteAngleRad @4 :Common.OptFloat64;
  convergedAngleDeg @5 :Common.OptFloat64;
  torqueMinEvPerAngstrom @6 :Common.OptFloat64;
  torqueMaxEvPerAngstrom @7 :Common.OptFloat64;
  rotationsMin @8 :Common.OptUInt32;
  rotationsMax @9 :Common.OptUInt32;
  removeRotation @10 :Common.OptBool;
  lorResidualTol @11 :Common.OptFloat64;
  modeLossCosineTol @12 :Common.OptFloat64;
  reseedRotationHistoryOnPullback @13 :Common.OptBool;
}

struct LanczosConfig {
  tolerance @0 :Float64 = 0.01 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "Lanczos", key = "tolerance")]
  );
  maxIterations @1 :UInt32 = 20 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "Lanczos", key = "max_iterations")]
  );
  quitEarly @2 :Bool = true $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "Lanczos", key = "quit_early")]
  );
  finiteDifference @3 :Common.FiniteDifferenceConfig;
  phvaMobileAtoms @4 :List(UInt32) $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "ini", section = "Lanczos", key = "phva_atoms")]
  );
  minCosineKeepHistory @5 :Float64 = 0.5 $Common.mutability(mutableLive);
}

struct LanczosPatch {
  tolerance @0 :Common.OptFloat64;
  maxIterations @1 :Common.OptUInt32;
  quitEarly @2 :Common.OptBool;
  finiteDifference @3 :Common.FiniteDifferencePatch;
  phvaMobileAtoms @4 :Common.OptUInt32List;
  minCosineKeepHistory @5 :Common.OptFloat64;
}

struct DavidsonConfig {
  tolerance @0 :Float64 = 0.001 $Common.mutability(mutableLive);
  maxIterations @1 :UInt32 = 32 $Common.mutability(mutableLive);
  maxSubspace @2 :UInt32 = 12 $Common.mutability(betweenStepsWithHook);
  numLowestModes @3 :UInt16 = 1 $Common.mutability(betweenStepsWithHook);
  preconCutoffAngstrom @4 :Float64 = 4.0 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = length, unit = angstrom);
  finiteDifference @5 :Common.FiniteDifferenceConfig;
}

struct DavidsonPatch {
  tolerance @0 :Common.OptFloat64;
  maxIterations @1 :Common.OptUInt32;
  maxSubspace @2 :Common.OptUInt32;
  numLowestModes @3 :Common.OptUInt16;
  preconCutoffAngstrom @4 :Common.OptFloat64;
  finiteDifference @5 :Common.FiniteDifferencePatch;
}

struct ArtnConfig {
  pushStepSizeAngstrom @0 :Float64 = 0.2 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  eigenvalueThreshold @1 :Float64 = -0.5 $Common.mutability(mutableLive) $Common.meta(dimension = forceConstant, unit = evPerAngstrom2);
  maxPerpRelaxIters @2 :UInt32 = 5 $Common.mutability(mutableLive);
  ninitPushSteps @3 :UInt32 = 3 $Common.mutability(mutableLive);
  nlanczosSteps @4 :UInt32 = 16 $Common.mutability(mutableLive);
  pushOverFraction @5 :Float64 = 1.0 $Common.mutability(mutableLive);
  pushIds @6 :List(UInt32) $Common.mutability(restartOnly);
  pushMode @7 :Text = "all" $Common.mutability(restartOnly);
  useFortranPartn @8 :Bool = false $Common.mutability(restartOnly);
}

struct ArtnPatch {
  pushStepSizeAngstrom @0 :Common.OptFloat64;
  eigenvalueThreshold @1 :Common.OptFloat64;
  maxPerpRelaxIters @2 :Common.OptUInt32;
  ninitPushSteps @3 :Common.OptUInt32;
  nlanczosSteps @4 :Common.OptUInt32;
  pushOverFraction @5 :Common.OptFloat64;
  pushIds @6 :Common.OptUInt32List;
  pushMode @7 :Common.OptText;
  useFortranPartn @8 :Common.OptBool;
}

struct SellaConfig {
  delta0Angstrom @0 :Float64 = 0.1 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  rhoInc @1 :Float64 = 1.035 $Common.mutability(mutableLive);
  rhoDec @2 :Float64 = 5.0 $Common.mutability(mutableLive);
  sigmaInc @3 :Float64 = 1.15 $Common.mutability(mutableLive);
  sigmaDec @4 :Float64 = 0.65 $Common.mutability(mutableLive);
  eigensolverTol @5 :Float64 = 1e-3 $Common.mutability(mutableLive);
  useInternalCoords @6 :Bool = false $Common.mutability(restartOnly);
}

struct SellaPatch {
  delta0Angstrom @0 :Common.OptFloat64;
  rhoInc @1 :Common.OptFloat64;
  rhoDec @2 :Common.OptFloat64;
  sigmaInc @3 :Common.OptFloat64;
  sigmaDec @4 :Common.OptFloat64;
  eigensolverTol @5 :Common.OptFloat64;
  useInternalCoords @6 :Common.OptBool;
}

# Refactored to reference Surrogate.GpConfig and Surrogate.LinearAlgebraConfig
# rather than duplicating GP noise, jitter, prior, and hyperparameter optimizer fields.
struct GprDimerConfig {
  gp @0 :Surrogate.GpConfig;
  linearAlgebra @1 :Surrogate.LinearAlgebraConfig;
  divisorT @2 :Float64 = 10.0 $Common.mutability(mutableLive);
  maxOuterIterations @3 :UInt32 = 1000 $Common.mutability(mutableLive);
  maxInnerIterations @4 :UInt32 = 1000 $Common.mutability(mutableLive);
  midpointFactor @5 :Float64 = 0.1 $Common.mutability(mutableLive);
  relaxationOffset @6 :Float64 = 0.1 $Common.mutability(mutableLive);
  activeRadiusAngstrom @7 :Float64 = 0.0 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = length, unit = angstrom);
  activeBulkDisplaceThreshold @8 :Float64 = 0.05 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  dimerSepAngstrom @9 :Float64 = 0.01 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  convStepAngstrom @10 :Float64 = 0.01 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  maxStepAngstrom @11 :Float64 = 0.1 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  initRotationsWithGp @12 :Bool = false $Common.mutability(mutableLive);
  initTranslationsWithGp @13 :Bool = false $Common.mutability(mutableLive);
  manyIterationsTol @14 :UInt32 = 100 $Common.mutability(mutableLive);
  checkDerivative @15 :Bool = false $Common.mutability(mutableLive);
  reportLevel @16 :UInt8 = 0 $Common.mutability(mutableLive);
  debugLevel @17 :UInt8 = 0 $Common.mutability(mutableLive);
  pruneBegin @18 :UInt32 = 30 $Common.mutability(mutableLive);
  pruneMaxVals @19 :UInt32 = 10 $Common.mutability(mutableLive);
  pruneThreshold @20 :Float64 = 1e-4 $Common.mutability(mutableLive);
}

struct GprDimerPatch {
  gp @0 :Surrogate.GpPatch;
  linearAlgebra @1 :Surrogate.LinearAlgebraPatch;
  divisorT @2 :Common.OptFloat64;
  maxOuterIterations @3 :Common.OptUInt32;
  maxInnerIterations @4 :Common.OptUInt32;
  midpointFactor @5 :Common.OptFloat64;
  relaxationOffset @6 :Common.OptFloat64;
  activeRadiusAngstrom @7 :Common.OptFloat64;
  activeBulkDisplaceThreshold @8 :Common.OptFloat64;
  dimerSepAngstrom @9 :Common.OptFloat64;
  convStepAngstrom @10 :Common.OptFloat64;
  maxStepAngstrom @11 :Common.OptFloat64;
  initRotationsWithGp @12 :Common.OptBool;
  initTranslationsWithGp @13 :Common.OptBool;
  manyIterationsTol @14 :Common.OptUInt32;
  checkDerivative @15 :Common.OptBool;
  reportLevel @16 :Common.OptUInt8;
  debugLevel @17 :Common.OptUInt8;
  pruneBegin @18 :Common.OptUInt32;
  pruneMaxVals @19 :Common.OptUInt32;
  pruneThreshold @20 :Common.OptFloat64;
}

struct IndexKConfig {
  targetSaddleOrder @0 :UInt16 = 1 $Common.mutability(betweenStepsWithHook);
  numTrackedEigenmodes @1 :UInt16 = 1 $Common.mutability(betweenStepsWithHook);
  reorthogonalizeEvery @2 :UInt32 = 1 $Common.mutability(mutableLive);
}

struct IndexKPatch {
  targetSaddleOrder @0 :Common.OptUInt16;
  numTrackedEigenmodes @1 :Common.OptUInt16;
  reorthogonalizeEvery @2 :Common.OptUInt32;
}

enum IrcDirection {
  both @0;
  forwardOnly @1;
  reverseOnly @2;
}

struct OptIrcDirection {
  union {
    unset @0 :Void;
    value @1 :IrcDirection;
  }
}

enum IrcIntegrator {
  eulerPredictorCorrector @0;
  localQuadraticApproximation @1;
  rungeKutta4 @2;
  velocityVerletDamped @3;
}

struct OptIrcIntegrator {
  union {
    unset @0 :Void;
    value @1 :IrcIntegrator;
  }
}

struct IrcConfig {
  stepSizeAngstrom @0 :Float64 = 0.05 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  maxSteps @1 :UInt32 = 200 $Common.mutability(mutableLive);
  direction @2 :IrcDirection = both $Common.mutability(restartOnly);
  integrator @3 :IrcIntegrator = eulerPredictorCorrector $Common.mutability(betweenStepsWithHook);
  endpointForceTolerance @4 :Float64 = 0.01 $Common.mutability(mutableLive) $Common.meta(dimension = force, unit = evPerAngstrom);
  endpointMaxRelaxSteps @5 :UInt32 = 200 $Common.mutability(mutableLive);
}

struct IrcPatch {
  stepSizeAngstrom @0 :Common.OptFloat64;
  maxSteps @1 :Common.OptUInt32;
  direction @2 :OptIrcDirection;
  integrator @3 :OptIrcIntegrator;
  endpointForceTolerance @4 :Common.OptFloat64;
  endpointMaxRelaxSteps @5 :Common.OptUInt32;
}

enum DisplacementStrategy {
  loadFromFile @0;
  randomAll @1;
  randomSingleAtom @2;
  undercoordinated @3;
  leastCoordinated @4;
  lastAtom @5;
  atomList @6;
  typeList @7;
  molecular @8;
}

struct OptDisplacementStrategy {
  union {
    unset @0 :Void;
    value @1 :DisplacementStrategy;
  }
}

struct DisplacementConfig {
  strategy @0 :DisplacementStrategy = loadFromFile $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "ini", section = "Saddle Search", key = "client_displace_type")]
  );
  radiusAngstrom @1 :Float64 = 5.0 $Common.mutability(restartOnly) $Common.meta(dimension = length, unit = angstrom);
  magnitudeAngstrom @2 :Float64 = 0.1 $Common.mutability(restartOnly) $Common.meta(dimension = length, unit = angstrom);
  maxCoordination @3 :UInt16 = 11 $Common.mutability(restartOnly);
  atomIndices @4 :List(Int32) $Common.mutability(restartOnly);
  elementTypes @5 :List(Text) $Common.mutability(restartOnly);
  randomDirection @6 :Bool = false $Common.mutability(restartOnly);
  voidBiasFraction @7 :Float64 = 0.0 $Common.mutability(restartOnly);
}

struct DisplacementPatch {
  strategy @0 :OptDisplacementStrategy;
  radiusAngstrom @1 :Common.OptFloat64;
  magnitudeAngstrom @2 :Common.OptFloat64;
  maxCoordination @3 :Common.OptUInt16;
  atomIndices @4 :Common.OptInt32List;
  elementTypes @5 :Common.OptTextList;
  randomDirection @6 :Common.OptBool;
  voidBiasFraction @7 :Common.OptFloat64;
}

struct SaddleSearchConfig {
  estimator @0 :MinModeEstimatorKind = improvedDimer $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "ini", section = "Saddle Search", key = "min_mode_method")]
  );
  translationOptimizer @1 :Opt.OptimizerConfig;
  convergence @2 :Conv.ConvergenceConfig;
  # Single authority for single-ended saddle refinement (handoff, PRFO, and saddle index check).
  refine @3 :Refine.SaddleRefineConfig;
  displacement @4 :DisplacementConfig;
  maxEnergyEv @5 :Float64 = 20.0 $Common.mutability(mutableLive) $Common.meta(dimension = energy, unit = ev);
  maxSingleDisplaceAngstrom @6 :Float64 = 10.0 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  minModeIterations @7 :UInt32 = 1000 $Common.mutability(mutableLive);
  usePositiveStepScale @8 :Bool = true $Common.mutability(mutableLive);
  positiveStepScale @9 :Float64 = 0.5 $Common.mutability(mutableLive);
  maxPositiveStepAngstrom @10 :Float64 = 0.2 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  zeroModeAbortCurvature @11 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(dimension = forceConstant, unit = evPerAngstrom2);
  confinePositive @12 :Bool = false $Common.mutability(mutableLive);
  confinePositiveMinForce @13 :Float64 = 0.5 $Common.mutability(mutableLive) $Common.meta(dimension = force, unit = evPerAngstrom);
  confinePositiveScaleRatio @14 :Float64 = 0.5 $Common.mutability(mutableLive);
  confinePositiveBoost @15 :Float64 = 10.0 $Common.mutability(mutableLive);
  confinePositiveMinActive @16 :UInt32 = 30 $Common.mutability(mutableLive);
  bowlBreakout @17 :Bool = false $Common.mutability(mutableLive);
  bowlActiveAtoms @18 :UInt32 = 20 $Common.mutability(mutableLive);
  nonlocalCountAbort @19 :UInt32 = 0 $Common.mutability(mutableLive);
  nonlocalDistanceAbortAngstrom @20 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(dimension = length, unit = angstrom);
  removeRotation @21 :Bool = false $Common.mutability(betweenStepsWithHook);
  dynamicsLinearInterpolation @22 :Bool = true $Common.mutability(restartOnly);
  dynamicsStateOutput @23 :Bool = true $Common.mutability(mutableLive);
  dynamicsTemperatureK @24 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(dimension = temperature, unit = kelvin);
  dimer @25 :DimerConfig;
  lanczos @26 :LanczosConfig;
  davidson @27 :DavidsonConfig;
  artn @28 :ArtnConfig;
  sella @29 :SellaConfig;
  gprDimer @30 :GprDimerConfig;
  indexK @31 :IndexKConfig;
  irc @32 :IrcConfig;
}

struct SaddleSearchPatch {
  estimator @0 :OptMinModeEstimatorKind;
  translationOptimizer @1 :Opt.OptimizerPatch;
  convergence @2 :Conv.ConvergencePatch;
  refine @3 :Refine.SaddleRefinePatch;
  displacement @4 :DisplacementPatch;
  maxEnergyEv @5 :Common.OptFloat64;
  maxSingleDisplaceAngstrom @6 :Common.OptFloat64;
  minModeIterations @7 :Common.OptUInt32;
  usePositiveStepScale @8 :Common.OptBool;
  positiveStepScale @9 :Common.OptFloat64;
  maxPositiveStepAngstrom @10 :Common.OptFloat64;
  zeroModeAbortCurvature @11 :Common.OptFloat64;
  confinePositive @12 :Common.OptBool;
  confinePositiveMinForce @13 :Common.OptFloat64;
  confinePositiveScaleRatio @14 :Common.OptFloat64;
  confinePositiveBoost @15 :Common.OptFloat64;
  confinePositiveMinActive @16 :Common.OptUInt32;
  bowlBreakout @17 :Common.OptBool;
  bowlActiveAtoms @18 :Common.OptUInt32;
  nonlocalCountAbort @19 :Common.OptUInt32;
  nonlocalDistanceAbortAngstrom @20 :Common.OptFloat64;
  removeRotation @21 :Common.OptBool;
  dynamicsLinearInterpolation @22 :Common.OptBool;
  dynamicsStateOutput @23 :Common.OptBool;
  dynamicsTemperatureK @24 :Common.OptFloat64;
  dimer @25 :DimerPatch;
  lanczos @26 :LanczosPatch;
  davidson @27 :DavidsonPatch;
  artn @28 :ArtnPatch;
  sella @29 :SellaPatch;
  gprDimer @30 :GprDimerPatch;
  indexK @31 :IndexKPatch;
  irc @32 :IrcPatch;
}
