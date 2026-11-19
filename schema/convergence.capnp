# Copyright 2023--present rgconf developers
@0xd782940f9a21b302;

using Common = import "common.capnp";

# Unified force norm and convergence criteria (resolved scalar struct + presence-aware patch).
# Under CompatibilityProfile.eonLegacyIni, optimizer.convergence.norm resolves to l2Euclidean
# ("norm") when unset, while band.convergence.norm and saddleSearch.convergence.norm resolve
# to maxAtomL2. Under rgconfCanonical, all three resolve to maxAtomL2.

enum ForceNorm {
  maxAtomL2 @0;         # max_i ||F_i||_2 (eOn "max_atom", rgmin RGMIN_STOP_NORM_MAX_ATOM, rgsaddle MaxAtomNorm)
  l2Euclidean @1;       # ||F||_2 across all 3N components (eOn "norm", rgmin RGMIN_STOP_NORM_TWO, rgsaddle L2Norm)
  rmsComponent @2;      # ||F||_2 / sqrt(3N) (rgmin RGMIN_STOP_NORM_RMS, rgsaddle RmsNorm)
  rmsAtom @3;           # ||F||_2 / sqrt(N_free) (eOn "rms")
  maxComponentLinf @4;  # max_{i,a} |F_{i,a}| (eOn "max_component", rgmin RGMIN_STOP_NORM_INF, rgsaddle LinfNorm)
}

struct OptForceNorm {
  union {
    unset @0 :Void;
    value @1 :ForceNorm;
  }
}

struct ConvergenceConfig {
  forceToleranceEvPerAngstrom @0 :Float64 = 0.01 $Common.mutability(mutableLive) $Common.meta(
    dimension = force,
    unit = evPerAngstrom,
    bounds = (hasMinExclusive = true, minExclusive = 0.0),
    aliases = [
      (format = "ini", section = "Optimizer", key = "converged_force"),
      (format = "ini", section = "Nudged Elastic Band", key = "converged_force"),
      (format = "ini", section = "Saddle Search", key = "converged_force"),
      (format = "toml", section = "band", key = "converged_force"),
      (format = "toml", section = "run", key = "converged_force")
    ],
    cliFlags = ["--converged-force", "--force-tol"],
    defaultKind = profileDependent,
    resolverFunction = "resolve_force_tolerance_from_profile"
  );
  norm @1 :ForceNorm = maxAtomL2 $Common.mutability(betweenStepsWithHook) $Common.meta(
    dimension = dimensionless,
    unit = none,
    aliases = [
      (format = "ini", section = "Optimizer", key = "convergence_metric"),
      (format = "toml", section = "band", key = "convergence_force_norm")
    ],
    cliFlags = ["--convergence-force-norm", "--stop-norm"],
    defaultKind = profileDependent,
    resolverFunction = "resolve_force_norm_from_profile"
  );
  energyToleranceEv @2 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(
    dimension = energy,
    unit = ev,
    bounds = (hasMinInclusive = true, minInclusive = 0.0)
  );
  stepToleranceAngstrom @3 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(
    dimension = length,
    unit = angstrom,
    bounds = (hasMinInclusive = true, minInclusive = 0.0)
  );
  gradientNoiseFloorEvPerAngstrom @4 :Float64 = 0.0 $Common.mutability(mutableLive) $Common.meta(
    dimension = force,
    unit = evPerAngstrom,
    bounds = (hasMinInclusive = true, minInclusive = 0.0)
  );
  requirePositiveCurvature @5 :Bool = false $Common.mutability(mutableLive);
  maxIterations @6 :UInt32 = 1000 $Common.mutability(mutableLive) $Common.meta(
    dimension = dimensionless,
    unit = none,
    bounds = (hasMinInclusive = true, minInclusive = 1),
    aliases = [
      (format = "ini", section = "Optimizer", key = "max_iterations"),
      (format = "toml", section = "band", key = "max_iterations"),
      (format = "toml", section = "run", key = "max_iterations")
    ],
    cliFlags = ["--max-iterations"]
  );
  maxForceCalls @7 :UInt32 = 0 $Common.mutability(mutableLive) $Common.meta(
    dimension = dimensionless,
    unit = none,
    aliases = [(format = "ini", section = "Main", key = "max_force_calls")],
    cliFlags = ["--max-force-calls"]
  );
}

# Presence-aware patch struct for layered input resolution and live RuntimeControl edits.
struct ConvergencePatch {
  forceToleranceEvPerAngstrom @0 :Common.OptFloat64;
  norm @1 :OptForceNorm;
  energyToleranceEv @2 :Common.OptFloat64;
  stepToleranceAngstrom @3 :Common.OptFloat64;
  gradientNoiseFloorEvPerAngstrom @4 :Common.OptFloat64;
  requirePositiveCurvature @5 :Common.OptBool;
  maxIterations @6 :Common.OptUInt32;
  maxForceCalls @7 :Common.OptUInt32;
}
