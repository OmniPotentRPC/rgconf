# Copyright 2023--present rgconf developers
@0xd782940f9a21b309;

using Common = import "common.capnp";

# Provenance, 8-level input precedence attribution, RFC 9162 Merkle log integration
# (via stackhands/rfc9162, rfc9162-tiles, and rfc9162-log), append-only mid-run
# ParamEdit leaves, live RuntimeControl RPC interface, Software Bill of
# Materials (SBOM), and fat hydrated reproducibility archive schema.
#
# All hashing, inclusion proofs, consistency proofs, C2SP tlog-tiles storage,
# sorted index runs, and Ed25519 signed checkpoints are delegated to
# stackhands/rfc9162 (v0.11.0); rgconf defines zero custom hash chains.

# Strict input precedence order (lowest = 0 to highest = 7).
# Derived values and C ABI struct defaults are NOT precedence layers above CLI;
# they apply only while a field is still unset and are recorded via ValueStatus.
enum ConfigLayer {
  schemaDefault @0;         # Literal wire default in .capnp
  compatibilityProfile @1;  # Named CompatibilityProfile (e.g. eonLegacyIni, gpridgePaper)
  userGlobalToml @2;        # $XDG_CONFIG_HOME/rgpkgs/config.toml
  projectToml @3;           # rgpkgs.toml / .rgpkgs.toml in project tree
  jobConfigFile @4;         # config.ini, job.toml, config.json, or RGCONF_CONFIG
  environmentVar @5;        # RGCONF_* / RGPKGS_* environment variables
  cliFlag @6;               # Explicit CLI flags (--set, --converged-force, --lock, --sbom)
  liveRuntimeEdit @7;       # Mid-run parameter edit via Python API, RuntimeControl RPC, hook, or restart
}

enum ValueStatus {
  defaultValue @0;            # Never set by user; retains literal wire default
  profileDefault @1;          # Unset by user; filled from active CompatibilityProfile
  derivedFromUnset @2;        # Unset by user; derived from another resolved field (never overrides explicit CLI/file/env)
  cAbiFallbackFromUnset @3;   # Unset by user; filled from a C ABI struct initializer
  explicitlySet @4;           # Set in one input layer with no higher-layer override
  layeredOverride @5;         # Set in multiple input layers; highest ConfigLayer won
  mutatedMidRun @6;           # Modified after initial resolution during execution (see HydratedRunArchive.editLog)
  silentlyBypassed @7;        # Explicitly set by user on an inactive backend/method (rejected in strict mode; logged in audit mode)
  deprecatedAlias @8;         # Populated via a deprecated legacy alias
}

# Mirrors `rfc9162::Head` (`crates/rfc9162/src/lib.rs:72-79`) and
# `rfc9162_tiles::TreeState` (`crates/rfc9162-tiles/src/log.rs:30-35`).
struct Rfc9162Head {
  origin @0 :Text;         # Schema-less C2SP checkpoint origin, e.g. "rgconf.omnipotent.rpc/run/<uuid>"
  size @1 :UInt64 = 0;     # Number of leaves covered by the Merkle root
  root @2 :Data;           # 32-byte RFC 9162 Merkle root (`rfc9162::root`)
  rootHex @3 :Text;        # 64-char lowercase hex root (`rfc9162::hex(&root)`)
}

# Mirrors `rfc9162_log::Inclusion` and `rfc9162_log::Located`
# (`crates/rfc9162-log/src/lib.rs:80-99`).
# Verified via `rfc9162::verify_inclusion(&leafHash, index, treeSize, &path, &rootHex)`
# (`crates/rfc9162/src/lib.rs:127-159`).
struct Rfc9162InclusionProof {
  index @0 :UInt64 = 0;             # Log leaf index (`Inclusion.index`)
  treeSize @1 :UInt64 = 0;          # Tree size the path proves against (`Inclusion.tree_size`)
  leafHash @2 :Data;                # 32-byte `SHA-256(0x00 || canonical_entry_bytes)` (`rfc9162::leaf_hash`)
  path @3 :List(Data);              # Sibling 32-byte subtree roots, innermost first (`Inclusion.path`)
  runManifestIndex @4 :Common.OptUInt64; # Log index of the sealed `RunManifest` (`Located.run.index`)
  runManifestPath @5 :List(Data);   # Inclusion path of the `RunManifest` (`Located.run.path`)
}

# Mirrors `rfc9162::consistency_proof` (`crates/rfc9162/src/lib.rs:166-171`) and
# `rfc9162_log::Recovery` (`crates/rfc9162-log/src/lib.rs:121-134`).
# Verified via `rfc9162::verify_consistency(oldSize, &oldRootHex, newSize, &newRootHex, &path)`
# (`crates/rfc9162/src/lib.rs:201-255`).
struct Rfc9162ConsistencyProof {
  oldSize @0 :UInt64 = 0;
  oldRootHex @1 :Text;
  newSize @2 :UInt64 = 0;
  newRootHex @3 :Text;
  path @4 :List(Data); # 32-byte RFC 9162 section 2.1.4 subproof nodes
}

# Mirrors `rfc9162_tiles::Note` and `rfc9162_tiles::Checkpoint`
# (`crates/rfc9162-tiles/src/checkpoint.rs:53-170`).
# Verified via `rfc9162_tiles::checkpoint::verify(&note, &trusted)` and
# `rfc9162_tiles::checkpoint::verify_cosignatures(&note, &witnesses)`
# (`crates/rfc9162-tiles/src/checkpoint.rs:382-450`).
struct Rfc9162CheckpointNote {
  head @0 :Rfc9162Head;
  noteText @1 :Text;               # Full C2SP signed-note text (`Note::to_text()`)
  verifierKeyLines @2 :List(Text); # `<name>+<8 hex key_id>+<base64(type || pubkey)>` (`Verifier::to_text()`)
  witnessKeyLines @3 :List(Text);  # Optional witness cosigner verifier lines (`ED25519_COSIGNATURE = 0x04`)
}

struct ParamValue {
  union {
    unset @0 :Void;
    boolVal @1 :Bool;
    int64Val @2 :Int64;
    uint64Val @3 :UInt64;
    float64Val @4 :Float64;
    textVal @5 :Text;
    intList @6 :List(Int64);
    floatList @7 :List(Float64);
    textList @8 :List(Text);
  }
}

struct LayerAssignment {
  layer @0 :ConfigLayer;
  sourceOrigin @1 :Text; # e.g. "config.ini:42", "rgpkgs.toml:18", "ENV:RGPKGS_LOCK", "CLI:--converged-force"
  rawValue @2 :ParamValue;
}

struct ResolvedParam {
  canonicalKey @0 :Text;
  component @1 :Text;
  legacySection @2 :Text;
  legacyKey @3 :Text;
  value @4 :ParamValue;
  defaultValue @5 :ParamValue;
  dimension @6 :Common.DimensionKind = dimensionless;
  unit @7 :Common.CanonicalUnit = none;
  unitSymbol @8 :Text;
  mutability @9 :Common.MutabilityTier = mutableLive;
  status @10 :ValueStatus = defaultValue;
  winningLayer @11 :ConfigLayer = schemaDefault;
  winningOrigin @12 :Text = "schema";
  layerHistory @13 :List(LayerAssignment);
  bypassWarning @14 :Text = "";
}

# Origin of a mid-run parameter edit recorded as a leaf in the run's rfc9162 log.
enum EditOrigin {
  userPythonApi @0;           # Python solver.set(...) / calc.set(...) / band.apply_patch(...)
  userRpcControl @1;          # Cap'n Proto RuntimeControl.applyPatch over UNIX/TCP socket
  betweenStepHook @2;         # Registered between-step callback hook
  checkpointRestart @3;       # Restart from checkpoint with edited config.toml / RgConfPatch
  internalRefineHandoff @4;   # OptimizerHandoffConfig switching primary -> secondary method at forceThreshold
  internalClimbingImageArm @5;# ClimbingImageConfig arming climbing image when force < triggerForce
  internalZoomRedistribute @6;# ZoomNebConfig redistributing band images onto window around peak
  internalOciNebEnter @7;     # OciNebRefineConfig entering inline or post-band min-mode phase
  internalOciNebExit @8;      # OciNebRefineConfig exiting min-mode phase
  internalSaddlePushOff @9;   # SaddleIndexCheckConfig pushing off higher-order saddle (index > expectedIndex)
  internalTrustRadiusAdapt @10;# Trust-region controller adapting trust radius
}

# Deterministic state-transition actions executed by a component when a parameter changes mid-run.
enum StateTransitionAction {
  none @0;
  resetLbfgsHistory @1;          # Clears secant pairs (s_k, y_k) and resets H_0 scale
  rebuildPreconditioner @2;      # Rebuilds Packwood Exp / sparse Cholesky preconditioner at current geometry
  recomputeClimbingImage @3;     # Re-evaluates interior image energies and updates ci_index / tangents
  redistributeBandImages @4;     # Interpolates band onto zoomed window and clears optimizer + endpoint cache
  clearEndpointEnergyCache @5;   # Invalidates cached endpoint energies [E_0, E_{N-1}]
  reinitializeThermostatRng @6;  # Rescales velocities / re-seeds thermostat Ornstein-Uhlenbeck state
  reopenPotentialSession @7;     # Reconfigures or reopens underlying rgpot/cpmdc/nwchemc/lammpc session
  reseedDimerRotation @8;        # Clears rotational CG search direction and resets trial angle state
}

# Append-only event record for every mid-run parameter change (external or internal handoff).
# Each ParamEdit is appended as a canonical Cap'n Proto leaf entry to the run's
# `rfc9162_log::Store` (`AuthenticatedLog::append`, `crates/rfc9162-log/src/store.rs:1141-1143`)
# and carries its `Rfc9162InclusionProof` against the run's signed checkpoint.
# Post-transition state vectors are stored as a content-addressed blob `blob/<hex>` in the
# `PackStore` (`Sidecar::put`, `crates/rfc9162-log/src/sidecar.rs:157-181`).
struct ParamEdit {
  editSeq @0 :UInt64;
  outerIteration @1 :UInt64;
  innerStep @2 :UInt64;
  forceEvalCount @3 :UInt64;
  wallTimeNanoseconds @4 :UInt64;
  canonicalKey @5 :Text;
  component @6 :Text;
  oldValue @7 :ParamValue;
  newValue @8 :ParamValue;
  unit @9 :Common.CanonicalUnit = none;
  mutabilityTier @10 :Common.MutabilityTier = mutableLive;
  origin @11 :EditOrigin;
  actorOrHookName @12 :Text;
  reason @13 :Text;
  stateActionsTaken @14 :List(StateTransitionAction);
  stateBlobHex @15 :Text;                 # `blob/<hex>` key in PackStore for post-transition state vector
  inclusionProof @16 :Rfc9162InclusionProof; # Inclusion proof of this ParamEdit leaf in the run's rfc9162 log
}

enum PackageEcosystem {
  pypi @0;
  cargo @1;
  mesonWrap @2;
  cmakeFetch @3;
  genericNative @4;
  condaPixi @5;
  systemLibrary @6;
  pythonEditable @7;
}

struct HashDigest {
  algorithm @0 :Text = "SHA-256";
  hexDigest @1 :Text;
}

struct ComponentSbomRecord {
  name @0 :Text;
  version @1 :Text;
  purl @2 :Text;
  bomRef @3 :Text;
  ecosystem @4 :PackageEcosystem = genericNative;
  vcsUrl @5 :Text = "";
  gitCommitSha @6 :Text = "";
  gitBranchOrTag @7 :Text = "";
  gitTreeDirty @8 :Bool = false;
  gitDiffBlobHex @9 :Text = ""; # `blob/<hex>` in PackStore if gitTreeDirty is true
  appliedPatches @10 :List(Text);
  compileFlags @11 :List(Text);
  featureGates @12 :List(Text);
  abiMajor @13 :UInt16 = 0;
  abiMinor @14 :UInt16 = 0;
  hashes @15 :List(HashDigest);
}

enum ArtifactRole {
  potentialModel @0;
  pseudopotential @1;
  forceFieldFile @2;
  reactantStructure @3;
  productStructure @4;
  initialPathFrame @5;
  displacementVector @6;
  gleMatrixA @7;
  gleMatrixC @8;
  inputLockFile @9;
  rawUserConfig @10;
  checkpointState @11;
}

enum CompressionCodec {
  none @0;
  zstd @1;
  gzip @2;
  xz @3;
  deflatePackStore @4; # Stored in `rfc9162_tiles::PackStore` (`blob/<hex>`, DEFLATE-compressed per record)
}

struct InlineBlob {
  codec @0 :CompressionCodec = deflatePackStore;
  uncompressedSizeBytes @1 :UInt64 = 0;
  blobHex @2 :Text;        # `blob/<hex>` key inside `repro/.sidecar/pack` (`Sidecar::blob`)
  compressedBytes @3 :Data;# Populated only when exporting a standalone single-message wire stream
}

struct ExternalBlob {
  archiveRelPath @0 :Text; # e.g. "artifacts/sha256/e3b0c442..."
  originalPath @1 :Text;
}

# Each ArtifactRecord is logged as a leaf in the run's `rfc9162_log::Store`
# (`Sidecar::put` at `crates/rfc9162-log/src/sidecar.rs:157-181`) and carries its
# `Rfc9162InclusionProof` against the run's signed checkpoint.
struct ArtifactRecord {
  role @0 :ArtifactRole;
  logicalName @1 :Text;
  sizeBytes @2 :UInt64 = 0;
  blobHex @3 :Text;        # Lowercase hex SHA-256 of artifact bytes (`rfc9162::hex`)
  mediaType @4 :Text = "application/octet-stream";
  lengthUnit @5 :Common.CanonicalUnit = angstrom;
  energyUnit @6 :Common.CanonicalUnit = ev;
  inclusionProof @7 :Rfc9162InclusionProof;
  union {
    inline @8 :InlineBlob;
    external @9 :ExternalBlob;
  }
}

enum LockFormatKind {
  cyclonedxJson @0;
  pep751PylockToml @1;
  uvLockToml @2;
  pixiLockYaml @3;
  cargoLockToml @4;
}

struct PackagePinOverride {
  packageName @0 :Text;
  lockVersion @1 :Text;
  overrideVersion @2 :Text;
  sourceLayer @3 :ConfigLayer;
  sourceOrigin @4 :Text;
}

struct LockInputRecord {
  format @0 :LockFormatKind;
  resolvedLayer @1 :ConfigLayer;
  path @2 :Text;
  blobHex @3 :Text;
  rawContent @4 :Text;
  pinOverrides @5 :List(PackagePinOverride);
}

enum RuntimeVerifyMode {
  strictEnforceAll @0;
  warnOnDrift @1;
  auditOnly @2;
}

struct SbomManifest {
  cyclonedxSpecVersion @0 :Text = "1.5";
  bomSerialNumberUrn @1 :Text;
  timestampIso8601 @2 :Text;
  verifyMode @3 :RuntimeVerifyMode = strictEnforceAll;
  inlineArtifactThresholdBytes @4 :UInt64 = 67108864; # 64 MiB default threshold
  components @5 :List(ComponentSbomRecord);
  artifacts @6 :List(ArtifactRecord);
  lockInputs @7 :List(LockInputRecord);
  # Child signed checkpoints from upstream toolchain / dataset logs included via
  # `Sidecar::include_checkpoint` (`crates/rfc9162-log/src/sidecar.rs:309-341`).
  includedChildCheckpoints @8 :List(Rfc9162CheckpointNote);
  cyclonedxJson @9 :Text;
  hostKernel @10 :Text;
  hostCpuArch @11 :Text;
  cpuModelName @12 :Text;
  mpiLibraryVersion @13 :Text;
  blasLapackBackend @14 :Text;
  gpuRuntimeVersion @15 :Text;
  inclusionProof @16 :Rfc9162InclusionProof; # Proof of the SbomManifest leaf in the run's rfc9162 log
}

struct EnvVarRecord {
  name @0 :Text;
  value @1 :Text;
  consumedByLayer @2 :Bool = true;
  redacted @3 :Bool = false;
}

# Authenticated reproducibility archive backed by `stackhands/rfc9162` (`v0.11.0`).
# All hand-rolled SHA-256 concatenation chains are replaced by the RFC 9162 Merkle tree:
#   - Leaf 0: canonical resolved parameter manifest (`resolvedParams` + CLI/env/file chain),
#     proven by `resolvedConfigProof`.
#   - Leaf 1: `sbom` (`SbomManifest`), proven by `sbom.inclusionProof`.
#   - Leaves 2..2+A-1: each `ArtifactRecord` in `sbom.artifacts`, proven by `artifact.inclusionProof`.
#   - Leaf 2+A: initial `RunManifest` sealed by `Store::seal()` (`initCheckpointNote` signs size `2+A+1`).
#   - Subsequent leaves: each mid-run `ParamEdit` in `editLog` (proven by `edit.inclusionProof`),
#     followed by the final `RunManifest` sealed at run completion (`finalCheckpointNote`).
#   - `initToFinalConsistency` (`Rfc9162ConsistencyProof`) proves via `rfc9162::verify_consistency`
#     that the tree at `finalCheckpointNote` is a strict append-only extension of `initCheckpointNote`.
struct HydratedRunArchive {
  archiveVersion @0 :UInt16 = 1;
  runUuid @1 :Text;
  parentCheckpointNote @2 :Rfc9162CheckpointNote; # Populated when resumed from a prior run's checkpoint
  hasParentCheckpoint @3 :Bool = false;
  createdAtIso8601 @4 :Text;
  activeProfile @5 :Common.CompatibilityProfile = rgconfCanonical;
  sidecarPackRelPath @6 :Text = "repro/.sidecar/pack"; # `rfc9162_tiles::PackStore` file path
  initCheckpointNote @7 :Rfc9162CheckpointNote;        # Signed C2SP checkpoint before first force evaluation
  finalCheckpointNote @8 :Rfc9162CheckpointNote;       # Latest signed C2SP checkpoint covering all edits
  initToFinalConsistency @9 :Rfc9162ConsistencyProof;  # Proves edit history only appended to initCheckpointNote
  resolvedConfigProof @10 :Rfc9162InclusionProof;      # Inclusion proof of the resolved-config leaf (index 0)
  resolvedParams @11 :List(ResolvedParam);
  sbom @12 :SbomManifest;
  overriddenCanonicalKeys @13 :List(Text);
  bypassedCanonicalKeys @14 :List(Text);
  editLog @15 :List(ParamEdit);
  configFileChain @16 :List(Text);
  redactedCliArgv @17 :List(Text);
  environmentVars @18 :List(EnvVarRecord);
}

# Live Cap'n Proto RPC steering interface exposed by a running eonclient,
# rgsaddle, rgmin, or rgpot server.
interface RuntimeControl {
  # Return current iteration counters, current signed C2SP checkpoint note, and serialized RgConf.
  getSnapshot @0 () -> (
    outerIteration :UInt64,
    innerStep :UInt64,
    forceEvalCount :UInt64,
    checkpointNote :Rfc9162CheckpointNote,
    rgconfBytes :Data
  );

  # Apply a presence-aware RgConfPatch at the next step boundary (or at applyAtIteration if > current),
  # append each resulting ParamEdit leaf to the run's rfc9162_log::Store, seal and sign a new
  # C2SP checkpoint note, and return the applied ParamEdit records (with inclusion proofs) plus
  # the consistency proof from the previous checkpoint head.
  applyPatch @1 (patchBytes :Data, reason :Text, actor :Text, applyAtIteration :UInt64) -> (
    accepted :Bool,
    message :Text,
    appliedEdits :List(ParamEdit),
    newCheckpointNote :Rfc9162CheckpointNote,
    consistencyFromPrevious :Rfc9162ConsistencyProof
  );

  # Pause stepping cleanly at the next step boundary so an external controller can inspect state.
  pauseAtNextStep @2 () -> (pausedIteration :UInt64, pausedForceEvalCount :UInt64, checkpointNote :Rfc9162CheckpointNote);

  # Resume stepping after pauseAtNextStep.
  resume @3 () -> (ok :Bool);

  # Seal the rfc9162_log::Store (`Sidecar::snapshot`), write a self-contained checkpoint bundle to path,
  # and return the signed C2SP checkpoint note.
  checkpointNow @4 (path :Text) -> (ok :Bool, checkpointNote :Rfc9162CheckpointNote);
}
