# Copyright 2023--present rgconf developers
@0xdb9dd5c47d0cf5a1;

# gpridge component extension for rgconf.
#
# Uses relative imports so `capnp compile -o- -I/cursor/stores/self/rgconf-schema`
# compiles this file directly alongside the core rgconf schemas.
#
# Fixes all structural review items:
#   1. Relative imports (`import "rgconf.capnp";` etc.).
#   2. Removes duplicate `GpridgeDefaultPolicy` (handled by `Common.CompatibilityProfile.gpridgePaper`
#      materializing `canonical.surrogateGp.modelProfile = gprdZbl`,
#      `canonical.band.convergence.forceToleranceEvPerAngstrom = 0.0514221`, and
#      `canonical.band.imageCount = nimages + 2 = 10` directly into `canonical`).
#   3. Removes duplicate `MetatomicDispatchConfig` (resolved directly into `canonical.potential.metatomic`
#      or `canonical.potential.uma`).
#   4. Reuses shared `Surrogate` structs (`canonical.surrogateGp`, `canonical.uncertainty`,
#      `canonical.acquisition`, `canonical.trust`, `canonical.linearAlgebra`) and
#      `canonical.band.refine.ociNeb` (`Common.CountLimit` instead of signed sentinels).
#   5. Deprecates unconsumed `ci_stability_count` and `use_prune` in source alias metadata only,
#      and adds `runSelector` to `LegacyDimerWireExtras`.

using Rg = import "rgconf.capnp";
using Common = import "common.capnp";

enum GpridgeBandMode {
  standard @0;
  aie @1;
  oie @2;
  gpAie @3;
  gpOie @4;
  gpLocal @5;
}

enum IrcSaddleHessian {
  measured @0;
  model @1;
}

enum MetatomicSo3Policy {
  raw @0;
  adaptive @1;
  always @2;
}

# Only gpridge-specific band execution controls not already in Band.BandConfig.
# `band.nimages` resolves into `canonical.band.imageCount = nimages + 2` (10 for default 8).
struct GpridgeBandExtensions {
  mode @0 :GpridgeBandMode = gpOie $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "toml", section = "band", key = "mode")],
    cliFlags = ["--nebmode"]
  );
  ircSaddleHessian @1 :IrcSaddleHessian = measured $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "band", key = "irc_saddle_hessian")],
    cliFlags = ["--irc-saddle-hessian"]
  );
  maxInnerIterations @2 :Common.CountLimit $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "band", key = "max_inner_iter")],
    cliFlags = ["--neb-max-inner-iter"]
  );
  endpointAssign @3 :Bool = true $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "toml", section = "band", key = "endpoint_assign")],
    cliFlags = ["--endpoint-assign"]
  );
  endpointAssignKmax @4 :Float64 = 1.8 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "band", key = "endpoint_assign_kmax")],
    cliFlags = ["--endpoint-assign-kmax"]
  );
  midrunRealign @5 :Bool = false $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "toml", section = "band", key = "midrun_realign")],
    cliFlags = ["--midrun-realign"]
  );
  reparamPeriod @6 :Common.CountLimit $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "band", key = "reparam_period")],
    cliFlags = ["--neb-reparam-period"]
  );
  reparamTolerance @7 :Common.OptFloat64 $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "band", key = "reparam_tol")],
    cliFlags = ["--neb-reparam-tol"]
  );
  eonInnerMode @8 :Bool = false $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "toml", section = "band", key = "eon_inner_mode")],
    cliFlags = ["--neb-eon-inner-mode"]
  );
  rgMinmodeDimer @9 :Bool = false $Common.mutability(betweenStepsWithHook) $Common.meta(
    aliases = [(format = "toml", section = "band.mmf", key = "rg_minmode_dimer")],
    cliFlags = ["--rg-minmode-dimer"]
  );
  metatomicSo3Policy @10 :MetatomicSo3Policy = adaptive $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "oracle.metatomic", key = "so3_policy")],
    cliFlags = ["--metatomic-so3-policy"]
  );
}

struct GpridgeRunConfig {
  outputPath @0 :Text = "gpr_neb_out.h5" $Common.mutability(restartOnly) $Common.meta(
    aliases = [(format = "toml", section = "run", key = "output")],
    cliFlags = ["--output"]
  );
  conOutputPath @1 :Text = "" $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "run", key = "con_output")],
    cliFlags = ["--write-con", "--con-output"]
  );
  traceBandHistory @2 :Bool = false $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "run", key = "trace_band_history")],
    cliFlags = ["--trace-band-history"]
  );
  traceConOutputPath @3 :Text = "" $Common.mutability(mutableLive) $Common.meta(
    aliases = [(format = "toml", section = "run", key = "trace_con_output")],
    cliFlags = ["--trace-con"]
  );
}

# Isolated legacy gprd wire transport fields (not part of resolved GpridgeConfig).
struct LegacyDimerWireExtras {
  configFile @0 :Text = "";
  cell @1 :Common.CellConfig;
  orientationInitial @2 :List(Float64);
  orientationStart @3 :List(Float64);
  displacedPositions @4 :List(Float64);
  runDistanceIndex @5 :Int32 = 0;
  runIndex @6 :Int32 = 0;
  separationDistancesAngstrom @7 :List(Float64);
  debugOutputDirectory @8 :Text = "";
  profileDimerTimes @9 :Bool = false;
  runSelector @10 :Int32 = 0;
}

# Unconsumed legacy keys. They still load, and the resolver marks them
# ValueStatus.deprecatedAlias so a job file is not rejected on sight.
struct GpridgeDeprecated {
  ciStabilityCount @0 :UInt32 = 0 $Common.mutability(mutableLive) $Common.meta(
    deprecated = true,
    deprecationReason = "The band solver does not read ci_stability_count.",
    doc = "Retained so older gpridge job files keep loading.",
    aliases = [
      (format = "toml", section = "band", key = "ci_stability_count", deprecated = true, deprecationNote = "Ignored by the solver. Remove it from the job file.")
    ]
  );
  usePrune @1 :Bool = false $Common.mutability(mutableLive) $Common.meta(
    deprecated = true,
    deprecationReason = "The band solver does not read use_prune.",
    doc = "Retained so older gpridge job files keep loading.",
    aliases = [
      (format = "toml", section = "band", key = "use_prune", deprecated = true, deprecationNote = "Ignored by the solver. Remove it from the job file.")
    ]
  );
}

struct GpridgeConfig {
  # Single resolved authority for all shared settings: band, optimizer, refine,
  # saddleSearch (including gprDimer), surrogateGp, uncertainty, acquisition,
  # trust, linearAlgebra, potential, conIo, runtime, and hydratedArchive.
  canonical @0 :Rg.RgConf;
  bandExtensions @1 :GpridgeBandExtensions;
  run @2 :GpridgeRunConfig;
  deprecated @3 :GpridgeDeprecated;
}
