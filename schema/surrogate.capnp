# Copyright 2023--present rgconf developers
@0xd782940f9a21b30b;

using Common = import "common.capnp";

# Shared Gaussian Process (GP) surrogate, hyperparameter fitting, posterior
# uncertainty, active-learning acquisition, trust-region controller, and
# distributed linear-algebra schema reused across eOn ([GPR Dimer]), gpridge,
# rgpot (GprParams), and rgsaddle.

enum GpModelProfile {
  chemgp @0;
  gprdZbl @1;
  extended @2;
}

struct OptGpModelProfile {
  union {
    unset @0 :Void;
    value @1 :GpModelProfile;
  }
}

# Resolved workflow has NO unresolved `profileDefault` state; `OptHyperparameterWorkflow`
# in `GpPatch` distinguishes `unset` (resolved from `modelProfile`) from explicit `inla`.
enum HyperparameterWorkflow {
  mapLbfgs @0;
  scg @1;
  inla @2;
}

struct OptHyperparameterWorkflow {
  union {
    unset @0 :Void;
    value @1 :HyperparameterWorkflow;
  }
}

enum HyperparameterObjective {
  nlml @0;
  loo @1;
  blockLoo @2;
}

struct OptHyperparameterObjective {
  union {
    unset @0 :Void;
    value @1 :HyperparameterObjective;
  }
}

struct HsSvdConfig {
  useSolver @0 :Bool = true $Common.mutability(betweenStepsWithHook) $Common.meta(cliFlags = ["--gp-hssvd"]);
  useObjectiveContractions @1 :Bool = true $Common.mutability(betweenStepsWithHook) $Common.meta(cliFlags = ["--gp-hssvd-objective-contractions"]);
  dualChopTolerance @2 :Float64 = 1e-6 $Common.mutability(mutableLive) $Common.meta(bounds = (hasMinExclusive = true, minExclusive = 0.0), cliFlags = ["--gp-hssvd-dual-chop-tol"]);
  dualRandomizedSketch @3 :UInt32 = 0 $Common.mutability(betweenStepsWithHook) $Common.meta(cliFlags = ["--gp-hssvd-dual-randomized-sketch"]);
  dualLapackRrr @4 :Bool = true $Common.mutability(betweenStepsWithHook) $Common.meta(cliFlags = ["--gp-hssvd-dual-lapack-rrr"]);
  localActiveSetSize @5 :UInt32 = 0 $Common.mutability(betweenStepsWithHook) $Common.meta(cliFlags = ["--gp-hssvd-local-active-set-size"]);
  localPosteriorActiveSetSize @6 :UInt32 = 0 $Common.mutability(betweenStepsWithHook) $Common.meta(cliFlags = ["--gp-hssvd-local-posterior-active-set-size"]);
}

struct HsSvdPatch {
  useSolver @0 :Common.OptBool;
  useObjectiveContractions @1 :Common.OptBool;
  dualChopTolerance @2 :Common.OptFloat64;
  dualRandomizedSketch @3 :Common.OptUInt32;
  dualLapackRrr @4 :Common.OptBool;
  localActiveSetSize @5 :Common.OptUInt32;
  localPosteriorActiveSetSize @6 :Common.OptUInt32;
}

struct GpConfig {
  modelProfile @0 :GpModelProfile = gprdZbl $Common.mutability(restartOnly) $Common.meta(
    cliFlags = ["--gp-model-profile"],
    defaultKind = profileDependent,
    resolverFunction = "resolve_gp_model_profile"
  );
  covarianceNoiseVariance @1 :Float64 = 1e-4 $Common.mutability(betweenStepsWithHook) $Common.meta(
    bounds = (hasMinInclusive = true, minInclusive = 0.0),
    aliases = [(format = "toml", section = "gp", key = "gp_sigma2")]
  );
  jitterVariance @2 :Float64 = 1e-4 $Common.mutability(betweenStepsWithHook) $Common.meta(
    bounds = (hasMinInclusive = true, minInclusive = 0.0),
    aliases = [
      (format = "toml", section = "gp", key = "jitter_sigma2"),
      (format = "ini", section = "GPR Dimer", key = "gpr_jitter_variance")
    ]
  );
  likelihoodNoiseVariance @3 :Float64 = 1e-4 $Common.mutability(betweenStepsWithHook) $Common.meta(
    bounds = (hasMinInclusive = true, minInclusive = 0.0),
    aliases = [
      (format = "toml", section = "gp", key = "noise_sigma2"),
      (format = "ini", section = "GPR Dimer", key = "gpr_noise_variance")
    ]
  );
  priorMean @4 :Float64 = 0.0 $Common.mutability(betweenStepsWithHook);
  priorVariance @5 :Float64 = 1.0 $Common.mutability(betweenStepsWithHook);
  lengthScaleAngstrom @6 :Float64 = 0.3 $Common.mutability(betweenStepsWithHook) $Common.meta(dimension = length, unit = angstrom);
  disableOutputNormalization @7 :Bool = false $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "toml", section = "gp", key = "no_output_norm")],
    cliFlags = ["--gp-no-output-norm"]
  );
  useZblPriorMean @8 :Bool = false $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "toml", section = "gp", key = "zbl_prior_mean")],
    cliFlags = ["--gp-zbl-prior-mean"]
  );
  profilePredictMean @9 :Bool = false $Common.mutability(mutableLive);
  profileEnergySplit @10 :Bool = false $Common.mutability(mutableLive);
  profileGradientSplit @11 :Bool = false $Common.mutability(mutableLive);
  hyperparameterWorkflow @12 :HyperparameterWorkflow = inla $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "toml", section = "gp", key = "hyperparameter_mode")],
    cliFlags = ["--hyperparameter-mode"],
    defaultKind = profileDependent,
    resolverFunction = "resolve_hyperparameter_workflow_from_profile"
  );
  hyperparameterObjective @13 :HyperparameterObjective = nlml $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "toml", section = "gp", key = "hyperparameter_objective")],
    cliFlags = ["--hyperparameter-objective"]
  );
  maxIterations @14 :UInt32 = 400 $Common.mutability(mutableLive) $Common.meta(
    bounds = (hasMinInclusive = true, minInclusive = 1),
    aliases = [
      (format = "toml", section = "gp", key = "max_iter"),
      (format = "ini", section = "GPR Dimer", key = "opt_max_iterations")
    ]
  );
  functionTolerance @15 :Float64 = 1e-3 $Common.mutability(mutableLive) $Common.meta(
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [
      (format = "toml", section = "gp", key = "tol_func"),
      (format = "ini", section = "GPR Dimer", key = "opt_tol_func")
    ],
    cliFlags = ["--gp-opt-tol"]
  );
  solutionTolerance @16 :Float64 = 1e-3 $Common.mutability(mutableLive) $Common.meta(
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [
      (format = "toml", section = "gp", key = "tol_sol"),
      (format = "ini", section = "GPR Dimer", key = "opt_tol_sol")
    ],
    cliFlags = ["--gp-opt-tol"]
  );
  lambdaLimit @17 :Float64 = 1e17 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "ini", section = "GPR Dimer", key = "opt_lambda_limit")]
  );
  lengthScalePrior @18 :Bool = true $Common.mutability(betweenStepsWithHook) $Common.meta(cliFlags = ["--ls-prior"]);
  principledNugget @19 :Bool = true $Common.mutability(betweenStepsWithHook) $Common.meta(cliFlags = ["--principled-nugget"]);
  kappaMax @20 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--kappa-max"]);
  hsSvd @21 :HsSvdConfig;
}

struct GpPatch {
  modelProfile @0 :OptGpModelProfile;
  covarianceNoiseVariance @1 :Common.OptFloat64;
  jitterVariance @2 :Common.OptFloat64;
  likelihoodNoiseVariance @3 :Common.OptFloat64;
  priorMean @4 :Common.OptFloat64;
  priorVariance @5 :Common.OptFloat64;
  lengthScaleAngstrom @6 :Common.OptFloat64;
  disableOutputNormalization @7 :Common.OptBool;
  useZblPriorMean @8 :Common.OptBool;
  profilePredictMean @9 :Common.OptBool;
  profileEnergySplit @10 :Common.OptBool;
  profileGradientSplit @11 :Common.OptBool;
  hyperparameterWorkflow @12 :OptHyperparameterWorkflow;
  hyperparameterObjective @13 :OptHyperparameterObjective;
  maxIterations @14 :Common.OptUInt32;
  functionTolerance @15 :Common.OptFloat64;
  solutionTolerance @16 :Common.OptFloat64;
  lambdaLimit @17 :Common.OptFloat64;
  lengthScalePrior @18 :Common.OptBool;
  principledNugget @19 :Common.OptBool;
  kappaMax @20 :Common.OptFloat64;
  hsSvd @21 :HsSvdPatch;
}

enum UncertaintyMode {
  map @0;
  laplace @1;
  laplaceSlice @2;
  bma @3;
  svgd @4;
  eb @5;
}

struct OptUncertaintyMode {
  union {
    unset @0 :Void;
    value @1 :UncertaintyMode;
  }
}

enum VarianceArm {
  psisDeterministic @0;
  psisRandom @1;
  deltaMethod @2;
  none @3;
}

struct OptVarianceArm {
  union {
    unset @0 :Void;
    value @1 :VarianceArm;
  }
}

struct UncertaintyConfig {
  mode @0 :UncertaintyMode = map $Common.mutability(betweenStepsWithHook) $Common.meta(cliFlags = ["--uncertainty-mode"]);
  varianceArm @1 :VarianceArm = psisDeterministic $Common.mutability(betweenStepsWithHook);
  # Uses FeatureCount (disabled | autoSelect | count(n)) rather than negative -1 sentinel.
  rffFeatures @2 :Common.FeatureCount $Common.mutability(betweenStepsWithHook) $Common.meta(cliFlags = ["--rff-features"]);
  rffForceToleranceFraction @3 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(
    bounds = (hasMinInclusive = true, minInclusive = 0.0, hasMaxExclusive = true, maxExclusive = 1.0),
    cliFlags = ["--rff-force-tol-fraction"]
  );
  acceptTau @4 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--variance-accept-tau"]);
  acceptTauRelative @5 :Common.OptFloat64 $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--variance-accept-tau-relative"]);
  innerUncertaintyRelative @6 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--inner-unc-relative"]);
}

struct UncertaintyPatch {
  mode @0 :OptUncertaintyMode;
  varianceArm @1 :OptVarianceArm;
  hasRffFeatures @2 :Bool = false;
  rffFeatures @3 :Common.FeatureCount;
  rffForceToleranceFraction @4 :Common.OptFloat64;
  acceptTau @5 :Common.OptFloat64;
  acceptTauRelative @6 :Common.OptFloat64;
  innerUncertaintyRelative @7 :Common.OptFloat64;
}

enum AcquisitionKind {
  maxVariance @0;
  maxForce @1;
  ucb @2;
  thompsonSampling @3;
  imspe @4;
  expectedImprovement @5;
  priorityCascade @6;
  tpe @7;
  psbax @8;
}

struct OptAcquisitionKind {
  union {
    unset @0 :Void;
    value @1 :AcquisitionKind;
  }
}

struct AcquisitionConfig {
  kind @0 :AcquisitionKind = tpe $Common.mutability(betweenStepsWithHook) $Common.meta(cliFlags = ["--acquisition"]);
  threePhasePicker @1 :Bool = true $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--three-phase-picker"]);
  seedFullChain @2 :Bool = false $Common.mutability(restartOnly) $Common.meta(cliFlags = ["--seed-full-chain"]);
  evaluationsPerIteration @3 :UInt32 = 1 $Common.mutability(mutableLive) $Common.meta(
    bounds = (hasMinInclusive = true, minInclusive = 1),
    cliFlags = ["--oie-evals"]
  );
  perImageMinimumTrainingRows @4 :UInt32 = 3 $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--neb-per-image-min-train"]);
  perImageKnn @5 :UInt32 = 10 $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--neb-per-image-knn"]);
  imspeRefitSkipRelativeTolerance @6 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--imspe-refit-skip-relative-tol"]);
  thetaShiftSkipTolerance @7 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--theta-shift-skip-tol"]);
  thetaShiftSkipAutomatic @8 :Bool = false $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--theta-shift-skip-auto"]);
}

struct AcquisitionPatch {
  kind @0 :OptAcquisitionKind;
  threePhasePicker @1 :Common.OptBool;
  seedFullChain @2 :Common.OptBool;
  evaluationsPerIteration @3 :Common.OptUInt32;
  perImageMinimumTrainingRows @4 :Common.OptUInt32;
  perImageKnn @5 :Common.OptUInt32;
  imspeRefitSkipRelativeTolerance @6 :Common.OptFloat64;
  thetaShiftSkipTolerance @7 :Common.OptFloat64;
  thetaShiftSkipAutomatic @8 :Common.OptBool;
}

enum TrustMetric {
  max1DLog @0;
  rmsd @1;
  emd @2;
}

struct OptTrustMetric {
  union {
    unset @0 :Void;
    value @1 :TrustMetric;
  }
}

struct TrustConfig {
  metric @0 :TrustMetric = emd $Common.mutability(betweenStepsWithHook) $Common.meta(cliFlags = ["--trust-metric"]);
  useAdaptive @1 :Bool = false $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--use-adaptive-trust"]);
  radius @2 :Float64 = 0.5 $Common.mutability(mutableLive) $Common.meta(bounds = (hasMinExclusive = true, minExclusive = 0.0), cliFlags = ["--trust-radius"]);
  adaptiveA @3 :Float64 = 1.3 $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--trust-adaptive-A"]);
  adaptiveFloor @4 :Float64 = 0.25 $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--trust-adaptive-floor"]);
  minimumRadius @5 :Float64 = 0.15 $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--trust-t-min"]);
  explorationDelta @6 :Float64 = 0.35 $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--trust-delta-explore"]);
  halfLifeRows @7 :UInt32 = 50 $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--trust-half-life"]);
  ratioAtLimit @8 :Float64 = 0.4 $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--trust-ratio-at-limit"]);
  scaleFree @9 :Bool = false $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--scalefree-trust"]);
  scaleFreeTau @10 :Float64 = 0.5 $Common.mutability(mutableLive) $Common.meta(cliFlags = ["--scalefree-tau"]);
}

struct TrustPatch {
  metric @0 :OptTrustMetric;
  useAdaptive @1 :Common.OptBool;
  radius @2 :Common.OptFloat64;
  adaptiveA @3 :Common.OptFloat64;
  adaptiveFloor @4 :Common.OptFloat64;
  minimumRadius @5 :Common.OptFloat64;
  explorationDelta @6 :Common.OptFloat64;
  halfLifeRows @7 :Common.OptUInt32;
  ratioAtLimit @8 :Common.OptFloat64;
  scaleFree @9 :Common.OptBool;
  scaleFreeTau @10 :Common.OptFloat64;
}

enum LinalgMode {
  reproducible @0;
  library @1;
}

struct OptLinalgMode {
  union {
    unset @0 :Void;
    value @1 :LinalgMode;
  }
}

struct LinearAlgebraConfig {
  mode @0 :LinalgMode = reproducible $Common.mutability(betweenStepsWithHook) $Common.meta(cliFlags = ["--linalg-mode"]);
  threads @1 :UInt32 = 1 $Common.mutability(betweenStepsWithHook) $Common.meta(bounds = (hasMinInclusive = true, minInclusive = 1), cliFlags = ["--linalg-threads"]);
  trainingProductLibrary @2 :Text = "" $Common.mutability(restartOnly) $Common.meta(cliFlags = ["--training-product-library"]);
  scalapackBlockSize @3 :UInt32 = 0 $Common.mutability(restartOnly) $Common.meta(cliFlags = ["--scalapack-block-size"]);
  scalapackNprow @4 :UInt32 = 0 $Common.mutability(restartOnly) $Common.meta(cliFlags = ["--scalapack-nprow"]);
}

struct LinearAlgebraPatch {
  mode @0 :OptLinalgMode;
  threads @1 :Common.OptUInt32;
  trainingProductLibrary @2 :Common.OptText;
  scalapackBlockSize @3 :Common.OptUInt32;
  scalapackNprow @4 :Common.OptUInt32;
}
