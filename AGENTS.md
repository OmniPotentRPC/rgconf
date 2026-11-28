# Agents

Generated from docs/orgmode by tools/generate_llm_docs.py.

Build with cargo. The workspace crate is `rgconf`. The C ABI is `rgconf-c-abi` and the header is `include/rgconf.h`. Python lives in `python/` (PyO3, module `_core`) and `bind/ext.cpp` (nanobind).

```
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
emacs --batch -l docs/export.el
python3 tools/generate_llm_docs.py
```

Public config structs are `#[non_exhaustive]` with `Default` and `with_*` setters. Provenance leaves go through the `rfc9162` crate. Production code does not call `unwrap_or` or `clamp`, and it does not introduce a bare `1e-N` literal. New files carry `Copyright 2023--present rgconf developers`.

# home.org

rgconf is the parameter and provenance library for the OmniPotentRPC stack.
One Cap'n Proto schema describes a run. A Rust resolver folds defaults, files,
environment variables, command line flags, and runtime edits. Provenance
leaves are RFC 9162 hashes from the =rfc9162= crate.

* Read next
- The tutorial shows one client file becoming a resolved band.
- The extension how-to adds a third party key without forking ordinals.
- The deprecation how-to covers =eon-schema= and the rgpot schema.
- The reference lists the public types.
- The precedence explanation is the rule the resolver implements.

# tutorials/resolve.org

This walk resolves an eOn client file with the =eonLegacyIni= profile.

* Write the file

[Main]
job = minimization
temperature = 300

[Optimizer]
opt_method = cg
converged_force = 0.03
max_move = 0.2

[Refine]
opt_method = fire
threshold = 0.2

[Nudged Elastic Band]
images = 7

=images= counts intermediate images. The resolver adds the two endpoints, so 7 becomes a total of 9.

* Resolve it

rgconf resolve --profile eonLegacyIni --ini client.ini

The printed line for the band force is derived from the optimizer force, because that profile inherits the optimizer tolerance when the band value is unset. =[Refine]= is routed to =optimizer.handoff= while the job is minimization. A nudged elastic band job routes the same section to =band.refine.handoff=.

* Ask for the proof

rgconf explain --profile eonLegacyIni --ini client.ini

=explain= appends the resolved fields to an RFC 9162 log and checks the inclusion proof of the first leaf with =rfc9162::verify_inclusion=.

# howto/extend-schema.org

A third party adds a parameter without taking a new ordinal on =RgConf=.

* Companion schema

A compiled companion, such as =gpridge.capnp=, imports =rgconf.capnp= and stores its extra fields beside =canonical=. That is the typed form. The file id of the companion is its own. =RgConf= ordinals stay put.

* Namespaced keys

A key that no companion declared is a =ThirdPartyExtension= on =RgConf.extensions= (@39) and on =RgConfPatch.extensions= (@24). The canonical name is =ext.<namespace>.<name>=.

[ext.acme]
alpha = 1.5

The resolver keeps that key, marks it mutable for a live edit, and does not invent a unit. Validation of bounds stays with the companion schema when the field is declared there.

* What not to do

Do not insert a field into the middle of =RgConf=. Do not reuse an ordinal. Do not put a second copy of =refine= or =thermostat= at the top level. Those domains already have one owner.

# howto/deprecation.org

=eon-schema= and the hand written rgpot parameter blocks are inputs to rgconf. They stop being a second source of truth.

* eon-schema

The INI sections =Main=, =Optimizer=, =Refine=, =Nudged Elastic Band=, and =Saddle Search= keep working through aliases on the Cap'n Proto fields. A removed key is a deprecated alias: the resolver still loads it and reports =deprecatedAlias=, as =ci_stability_count= and =use_prune= already do on the gpridge companion.

The removal order is:

- Ship rgconf in the clients that still read those sections.
- Keep the aliases for one release while jobs print the canonical key beside the old one.
- Delete the alias from the schema only after that release, in a schema file that bumps nothing underneath an existing ordinal.

=eon-schema= itself is not imported by this crate. The alias table is the migration.

* rgpot schema

=potential.capnp= is the evolved rgpot parameter schema. The union arm is the backend. There is no =kind= field beside it. New backends are new arms at the next free ordinal, not a parallel struct in the rgpot tree.

Fields that exist only as comments in a C++ header stay in the generated parameter map until the schema field that replaces them is in a release. The map records the header line. It is not a second schema.

=potential.capnp= is the only copy of the potentials schema (file id =0xbd1f89fa17369103=). rgpot vendors it byte for byte, with =common.capnp=, instead of carrying the =potentials-schema= file and its local diff series. It holds every member those two carried, at their ordinals: the =composite= (18), =d2= (19), =d2cpmd= (20) and =mullerBrown= (21) arms, =ForceInput.hasOrbitalKey= (10) and =orbitalKey= (11), =D4Params.periodic=, and the =hasCharge= flags of =CommonMethodSpec=, =CPMDParams= and =CPMDSystemSection=.

rgconf 0.1.0 had used some of those ordinals for its own members, so they moved:

| Member                          | 0.1.0 | Now |
|---------------------------------+-------+-----|
| =PotentialConfig.gpr=           |    18 |  27 |
| =PotentialConfig.potlib=        |    19 |  28 |
| =PotentialConfig.external=      |    20 |  29 |
| =PotentialConfig.plugin=        |    21 |  30 |
| =ForceInput.hessianDisplacement= |   10 |  12 |
| =ForceInput.wantStress=         |    11 |  13 |
| =ForceInput.wantHessianVector=  |    12 |  14 |

Keys, INI and TOML aliases and the resolved =potential.arm= value are names, so a resolution reads the same before and after. rgconf 0.1.0 neither encoded nor decoded a Cap'n Proto message, so it wrote no binary with the old ordinals. A binary message that another tool encoded against the 0.1.0 file carries those arms or fields at the old ordinals and must be encoded again from its text form.

The =orca= arm (=PotentialConfig= 31, =PotentialSpec= 7) with =ORCAParams= is the configuration of the =orcac= engine shim.

* Defaults must agree

The Lennard Jones and Morse =switchWidth= literals are 0.0, matching
=LJConfig.switch_width=, =LJClusterConfig.switch_width=,
=MorseConfig.switch_width=, and the long-range dispersion switch in rgpot.
The importer keeps the source value and its exact line.

# reference/parameters.org

* Profiles

=rgconfCanonical=, =eonLegacyIni=, =eonServerYaml=, =gpridgePaper=, =gpridgeLegacyToml=, and =aseDropin=.

=eonLegacyIni= uses =l2Euclidean= for the optimizer norm and =maxAtomL2= for the band and the saddle search. The band force copies the optimizer force when the band value is unset. =eonServerYaml= selects the =akmcProcessSearch= job and the =external= potential. Both gpridge profiles use 10 images. The paper force is one thousandth of a hartree per bohr, converted with the CODATA 2018 constants, not the rounded 0.0514221 figure.

* Layers

User layers, from lowest to highest, are the global file, the project file, the job file, the environment, the command line, and a live edit. Schema defaults and the profile sit underneath. Two aliases in one layer that normalize to different numbers are an error and the message names both keys. Equal values keep both origins.

* Derived values

=band.zoom.triggerForceEvPerAngstrom= is the band force tolerance times =triggerForceMultiplier= (10 unless set). =band.spring.ewKMinEvPerAngstrom2= is one tenth of =constantEvPerAngstrom2=. The schema file still carries the literals 0.5 and 0.972. The resolver uses the formulas when the user left the field unset. Those literals are not the runtime default.

* Fixed-energy LI-NEB

The =lineb= job uses the =lineb.*= parameters in the eOn =[Lineb]= section. Energy is in eV and force tolerances are in √eV. Unset tolerances resolve to 0.001.

For a fixed endpoint restraint, set =endpoint_kappa= to the coefficient in inverse square-root eV. Zero selects the gradient-normalized =endpoint_scale=. Set =endpoint_convergence= to test =opt_tol= on the two endpoints and =perp_tol= on the interior images. Set =linear_seed= to join the two MEP energy crossings with a straight line. The Cu(110) example uses this straight seed, while the ammonia borane example follows the MEP. Set =projected_verlet= to use velocity projection with velocity Verlet. Each Boolean defaults to false.

* Public Rust types

=RuntimeConfig=, =ConvergenceConfig=, =OptimizerConfig=, =LbfgsConfig=, =FireConfig=, =HandoffConfig=, =BandConfig=, =SpringConfig=, =ClimbingImageConfig=, =UnitSystem=, and =RgConf= are =#[non_exhaustive]=, implement =Default=, and expose =with_*= setters.

* C ABI

=include/rgconf.h= defines =RGCONF_ABI_VERSION_MAJOR=, =RGCONF_ABI_VERSION_MINOR=, =RGCONF_ABI_VERSION_PATCH=, and =RGCONF_ABI_VERSION=. =rgconf_abi_version= returns the packed value. =rgconf_resolve_ini= and =rgconf_apply_edit= write a heap buffer released with =rgconf_free=.

* Python

=python/= is a PyO3 crate named =_core=, the same shape as rgmin. =bind/ext.cpp= is the nanobind module in the rgsaddle style and links the C ABI. =rgconf.resolve_ini= uses the extension when it is installed and the C ABI otherwise.

* Provenance

Config fields, SBOM purls, archive digests, and =ParamEdit= records are appended to =rfc9162_log::Store=. The leaf hash is =rfc9162::leaf_hash=. rgconf does not hash a leaf itself. Gaps are listed in the upstream page.

# explanation/precedence.org

A field starts at its schema literal with status =defaultValue=. A profile function then replaces it with =profileDefault= or, when the value is copied from another field, =derivedFromUnset=. A user layer that sets the field wins over both. A higher user layer replaces a lower one and the status becomes =layeredOverride=. One user layer is =explicitlySet=. A winning deprecated alias is =deprecatedAlias=. A =ParamEdit= is =mutatedMidRun= at the live edit layer.

=restartOnly= fields reject a live edit. A checkpoint restart may change them. =mutableLive= edits record no hook. =betweenStepsWithHook= edits record the hook for that key: LBFGS history, the preconditioner, climbing images, band spacing, the thermostat, the potential, the dimer, or the cell.

Environment names are =RGCONF_= plus the canonical key, with a capital letter after a lower case letter gaining an underscore. =band.imageCount= is =RGCONF_BAND_IMAGE_COUNT=. That variable is the total image count. The =--images= flag is the intermediate count and adds two.

# explanation/upstream.org

* rfc9162 v0.11.0

These calls return =Error::UpstreamPatch= and are not implemented here.

- =rfc9162::canonicalize_capnp= should pack a Cap'n Proto message before =leaf_hash=.
- =rfc9162_log::Store::put_blob_and_append= should store an entry longer than 65535 bytes as a blob and append a short pointer leaf. =Sidecar::put= writes a file and does not append that leaf.
- =rfc9162_tiles::Writer::append_shared= should append without holding =treeState.lock= from open until drop.
- C and Python bindings for the log belong in the =rfc9162= repository.

* Notes on the design text

The schema =ConfigLayer= ordinals stop at =liveRuntimeEdit= (@7). =derived= and the C ABI fallback are =ValueStatus= values, not layers. A map section that calls the fallback a layer is the part that does not match the schema.

=OciNebRefinePatch= still carries =hasMaxSteps=, =hasMaxPhases=, and =hasMaxInnerIterations= beside its counts. That is the presence shape the rest of the schema left behind. The ordinals are unchanged.

The zoom trigger literal in the schema is 0.5 and the =ewKMin= literal is 0.972. The resolver text says 10 times the band force and one tenth of the spring constant. The formulas win when the user did not set the field.

The gpridge companion did not declare =ci_stability_count= or =use_prune=. Those deprecated aliases are on =GpridgeDeprecated= so an old job file still loads.

=RgConf= had no list for a namespaced key that no companion schema declared. =extensions= (@39) is that list.

# reference/source-parameter-map.rst

Imported source parameter map
=============================

Generated from ``generated/parameter_map.tsv``. Each entry records the
source declaration, type, default, and source section. A blank default
means the source declaration has no default. Units remain those stated
by the named source API when the source map has no unit annotation.
Entries that carry a mapping name the rgconf schema field (``unmapped``
when rgconf has none, ``abi`` for ABI bookkeeping) and the eOn option
(``none`` when eOn has no counterpart).

* ``eon:constants_t.kB`` — ``double``, default ``8.6173324e-5``; ``include/eon/ParametersOptions.h:25``
* ``eon:constants_t.timeUnit`` — ``double``, default ``10.1805055``; ``include/eon/ParametersOptions.h:26``
* ``eon:main_options_t.job`` — ``JobType``, default ``JobType::Process_Search``; ``include/eon/ParametersOptions.h:31``; section Main
* ``eon:main_options_t.randomSeed`` — ``long``, default ``-1``; ``include/eon/ParametersOptions.h:32``; section Main
* ``eon:main_options_t.temperature`` — ``double``, default ``300.0``; ``include/eon/ParametersOptions.h:33``; section Main
* ``eon:main_options_t.quiet`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:34``; section Main
* ``eon:main_options_t.writeLog`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:35``; section Main
* ``eon:main_options_t.checkpoint`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:36``; section Main
* ``eon:main_options_t.iniFilename`` — ``std::string``, default ``config.ini``; ``include/eon/ParametersOptions.h:37``; section Main
* ``eon:main_options_t.conFilename`` — ``std::string``, default ``pos.con``; ``include/eon/ParametersOptions.h:38``; section Main
* ``eon:main_options_t.finiteDifference`` — ``double``, default ``0.01``; ``include/eon/ParametersOptions.h:39``; section Main
* ``eon:main_options_t.maxForceCalls`` — ``long``, default ``0``; ``include/eon/ParametersOptions.h:40``; section Main
* ``eon:main_options_t.removeNetForce`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:41``; section Main
* ``eon:main_options_t.parallel`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:42``; section Main
* ``eon:main_options_t.writeConForces`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:46``; section Main
* ``eon:potential_options_t.potential`` — ``PotType``, default ``PotType::LJ``; ``include/eon/ParametersOptions.h:51``; section Potential
* ``eon:potential_options_t.MPIPollPeriod`` — ``double``, default ``0.25``; ``include/eon/ParametersOptions.h:52``; section Potential
* ``eon:potential_options_t.LAMMPSLogging`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:53``; section Potential
* ``eon:potential_options_t.LAMMPSThreads`` — ``int``, default ``0``; ``include/eon/ParametersOptions.h:54``; section Potential
* ``eon:potential_options_t.EMTRasmussen`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:55``; section Potential
* ``eon:potential_options_t.LogPotential`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:56``; section Potential
* ``eon:potential_options_t.thread_safe`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:59``; section Potential
* ``eon:potential_options_t.extPotPath`` — ``std::string``, default ``./ext_pot``; ``include/eon/ParametersOptions.h:60``; section Potential
* ``eon:potential_options_t.MPIPotentialRank`` — ``int``, default ``-1``; ``include/eon/ParametersOptions.h:63``; section Potential
* ``eon:potential_options_t.MPIClientComm`` — ``std::uintptr_t``, default ``0``; ``include/eon/ParametersOptions.h:65``; section Potential
* ``eon:ams_options_t.engine`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:70``; section AMS
* ``eon:ams_options_t.forcefield`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:71``; section AMS
* ``eon:ams_options_t.model`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:72``; section AMS
* ``eon:ams_options_t.resources`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:73``; section AMS
* ``eon:ams_options_t.xc`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:74``; section AMS
* ``eon:ams_options_t.basis`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:75``; section AMS
* ``eon:env_t.amshome`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:77``; section AMS
* ``eon:env_t.scm_tmpdir`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:78``; section AMS
* ``eon:env_t.scmlicense`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:79``; section AMS
* ``eon:env_t.scm_pythondir`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:80``; section AMS
* ``eon:env_t.amsbin`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:81``; section AMS
* ``eon:env_t.amsresources`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:82``; section AMS
* ``eon:xtb_options_t.paramset`` — ``std::string``, default ``GFNFF``; ``include/eon/ParametersOptions.h:88``; section XTBPot
* ``eon:xtb_options_t.elec_temperature`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:89``; section XTBPot
* ``eon:xtb_options_t.maxiter`` — ``size_t``, default ``250``; ``include/eon/ParametersOptions.h:90``; section XTBPot
* ``eon:xtb_options_t.acc`` — ``double``, default ``1.0``; ``include/eon/ParametersOptions.h:91``; section XTBPot
* ``eon:xtb_options_t.charge`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:92``; section XTBPot
* ``eon:xtb_options_t.uhf`` — ``int``, default ``0``; ``include/eon/ParametersOptions.h:93``; section XTBPot
* ``eon:zbl_options_t.cut_inner`` — ``double``, default ``2.0``; ``include/eon/ParametersOptions.h:98``; section ZBLPot
* ``eon:zbl_options_t.cut_global`` — ``double``, default ``2.5``; ``include/eon/ParametersOptions.h:99``; section ZBLPot
* ``eon:dftd_options_t.functional`` — ``std::string``, default ``pbe``; ``include/eon/ParametersOptions.h:104``; section D3Pot
* ``eon:dftd_options_t.atm`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:105``; section D3Pot
* ``eon:dftd_options_t.d3_damping`` — ``std::string``, default ``bj``; ``include/eon/ParametersOptions.h:106``; section D3Pot
* ``eon:dftd_options_t.d4_charge`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:107``; section D3Pot
* ``eon:expr_options_t.expression`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:112``; section ExprPot
* ``eon:expr_options_t.terms`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:113``; section ExprPot
* ``eon:mopac_options_t.charge`` — ``int``, default ``0``; ``include/eon/ParametersOptions.h:118``; section MOPACPot
* ``eon:mopac_options_t.spin`` — ``int``, default ``0``; ``include/eon/ParametersOptions.h:119``; section MOPACPot
* ``eon:mopac_options_t.model`` — ``int``, default ``4``; ``include/eon/ParametersOptions.h:120``; section MOPACPot
* ``eon:mopac_options_t.engine_path`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:121``; section MOPACPot
* ``eon:socket_nwchem_options_t.host`` — ``std::string``, default ``127.0.0.1``; ``include/eon/ParametersOptions.h:126``; section SocketNWChemPot
* ``eon:socket_nwchem_options_t.port`` — ``int``, default ``9999``; ``include/eon/ParametersOptions.h:127``; section SocketNWChemPot
* ``eon:socket_nwchem_options_t.mem_in_gb`` — ``int``, default ``2``; ``include/eon/ParametersOptions.h:128``; section SocketNWChemPot
* ``eon:socket_nwchem_options_t.nwchem_settings`` — ``std::string``, default ``nwchem_settings.nwi``; ``include/eon/ParametersOptions.h:129``; section SocketNWChemPot
* ``eon:socket_nwchem_options_t.unix_socket_path`` — ``std::string``, default ``eon_nwchem``; ``include/eon/ParametersOptions.h:130``; section SocketNWChemPot
* ``eon:socket_nwchem_options_t.unix_socket_mode`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:131``; section SocketNWChemPot
* ``eon:socket_nwchem_options_t.make_template_input`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:132``; section SocketNWChemPot
* ``eon:rgpot_options_t.backend`` — ``std::string``, default ``nwchemc``; ``include/eon/ParametersOptions.h:139``; section RgpotPot
* ``eon:rgpot_options_t.basis`` — ``std::string``, default ``sto-3g``; ``include/eon/ParametersOptions.h:140``; section RgpotPot
* ``eon:rgpot_options_t.theory`` — ``std::string``, default ``scf``; ``include/eon/ParametersOptions.h:141``; section RgpotPot
* ``eon:rgpot_options_t.scf_type`` — ``std::string``, default ``rhf``; ``include/eon/ParametersOptions.h:142``; section RgpotPot
* ``eon:rgpot_options_t.functional`` — ``std::string``, default ``BLYP``; ``include/eon/ParametersOptions.h:143``; section RgpotPot
* ``eon:rgpot_options_t.cutoff_ry`` — ``double``, default ``70.0``; ``include/eon/ParametersOptions.h:146``; section cpmd
* ``eon:rgpot_options_t.charge`` — ``int``, default ``0``; ``include/eon/ParametersOptions.h:147``; section cpmd
* ``eon:rgpot_options_t.multiplicity`` — ``int``, default ``1``; ``include/eon/ParametersOptions.h:148``; section cpmd
* ``eon:rgpot_options_t.engine_path`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:149``; section cpmd
* ``eon:rgpot_options_t.engine_library`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:150``; section cpmd
* ``eon:rgpot_options_t.engine_root`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:151``; section cpmd
* ``eon:rgpot_options_t.title`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:152``; section cpmd
* ``eon:rgpot_options_t.memory_mb`` — ``int``, default ``0``; ``include/eon/ParametersOptions.h:153``; section cpmd
* ``eon:rgpot_options_t.scratch_dir`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:154``; section cpmd
* ``eon:rgpot_options_t.input_block`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:157``; section cpmd
* ``eon:rgpot_options_t.permanent_dir`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:158``; section cpmd
* ``eon:rgpot_options_t.params_path`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:162``; section cpmd
* ``eon:rgpot_options_t.ranks_per_image`` — ``int``, default ``0``; ``include/eon/ParametersOptions.h:166``; section cpmd
* ``eon:rgpot_options_t.model_path`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:168``; section cpmd
* ``eon:rgpot_options_t.device`` — ``std::string``, default ``cpu``; ``include/eon/ParametersOptions.h:169``; section cpmd
* ``eon:rgpot_options_t.length_unit`` — ``std::string``, default ``angstrom``; ``include/eon/ParametersOptions.h:170``; section cpmd
* ``eon:rgpot_options_t.extensions_directory`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:171``; section cpmd
* ``eon:rgpot_options_t.check_consistency`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:172``; section cpmd
* ``eon:rgpot_options_t.uncertainty_threshold`` — ``double``, default ``-1.0``; ``include/eon/ParametersOptions.h:173``; section cpmd
* ``eon:rgpot_options_t.torch_determinism_strict`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:174``; section cpmd
* ``eon:rgpot_options_t.xtb_paramset`` — ``std::string``, default ``GFN2xTB``; ``include/eon/ParametersOptions.h:176``; section cpmd
* ``eon:rgpot_options_t.xtb_accuracy`` — ``double``, default ``1.0``; ``include/eon/ParametersOptions.h:177``; section cpmd
* ``eon:rgpot_options_t.xtb_electronic_temperature`` — ``double``, default ``300.0``; ``include/eon/ParametersOptions.h:178``; section cpmd
* ``eon:rgpot_options_t.xtb_max_iterations`` — ``int``, default ``250``; ``include/eon/ParametersOptions.h:179``; section cpmd
* ``eon:rgpot_options_t.xtb_charge`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:180``; section cpmd
* ``eon:rgpot_options_t.xtb_uhf`` — ``int``, default ``0``; ``include/eon/ParametersOptions.h:181``; section cpmd
* ``eon:rgpot_options_t.task_name`` — ``std::string``, default ``omol``; ``include/eon/ParametersOptions.h:182``; section cpmd
* ``eon:process_search_options_t.minimize_first`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:190``; section Process Search
* ``eon:process_search_options_t.minimization_offset`` — ``double``, default ``0.2``; ``include/eon/ParametersOptions.h:191``; section Process Search
* ``eon:saddle_search_options_t.max_jump_attempts`` — ``long``, default ``0``; ``include/eon/ParametersOptions.h:196``; section Saddle Search
* ``eon:saddle_search_options_t.max_iterations`` — ``long``, default ``1000``; ``include/eon/ParametersOptions.h:197``; section Saddle Search
* ``eon:saddle_search_options_t.method`` — ``std::string``, default ``min_mode``; ``include/eon/ParametersOptions.h:198``; section Saddle Search
* ``eon:saddle_search_options_t.minmode_method`` — ``std::string``, default ``dimer``; ``include/eon/ParametersOptions.h:199``; section Saddle Search
* ``eon:saddle_search_options_t.displace_type`` — ``std::string``, default ``load``; ``include/eon/ParametersOptions.h:200``; section Saddle Search
* ``eon:saddle_search_options_t.displace_atom_list`` — ``std::vector<long>``, default ``(no source default)``; ``include/eon/ParametersOptions.h:201``; section Saddle Search
* ``eon:saddle_search_options_t.max_energy`` — ``double``, default ``20.0``; ``include/eon/ParametersOptions.h:202``; section Saddle Search
* ``eon:saddle_search_options_t.displace_magnitude`` — ``double``, default ``0.1``; ``include/eon/ParametersOptions.h:203``; section Saddle Search
* ``eon:saddle_search_options_t.max_single_displace`` — ``double``, default ``10.0``; ``include/eon/ParametersOptions.h:204``; section Saddle Search
* ``eon:saddle_search_options_t.displace_radius`` — ``double``, default ``4.0``; ``include/eon/ParametersOptions.h:205``; section Saddle Search
* ``eon:saddle_search_options_t.converged_force`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:206``; section Saddle Search
* ``eon:saddle_search_options_t.perp_force_ratio`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:207``; section Saddle Search
* ``eon:saddle_search_options_t.nonnegative_displacement_abort`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:208``; section Saddle Search
* ``eon:saddle_search_options_t.nonlocal_count_abort`` — ``long``, default ``0``; ``include/eon/ParametersOptions.h:209``; section Saddle Search
* ``eon:saddle_search_options_t.nonlocal_distance_abort`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:210``; section Saddle Search
* ``eon:saddle_search_options_t.remove_rotation`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:211``; section Saddle Search
* ``eon:saddle_search_options_t.zero_mode_abort_curvature`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:212``; section Saddle Search
* ``eon:dynamics_t.temperature`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:214``; section Saddle Search
* ``eon:dynamics_t.state_check_interval_input`` — ``double``, default ``100.0``; ``include/eon/ParametersOptions.h:215``; section Saddle Search
* ``eon:dynamics_t.state_check_interval`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:216``; section Saddle Search
* ``eon:dynamics_t.record_interval_input`` — ``double``, default ``10.0``; ``include/eon/ParametersOptions.h:217``; section Saddle Search
* ``eon:dynamics_t.record_interval`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:218``; section Saddle Search
* ``eon:dynamics_t.linear_interpolation`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:219``; section Saddle Search
* ``eon:dynamics_t.max_init_curvature`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:220``; section Saddle Search
* ``eon:confine_positive_t.enabled`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:223``; section Saddle Search
* ``eon:confine_positive_t.bowl_breakout`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:224``; section Saddle Search
* ``eon:confine_positive_t.bowl_active`` — ``long``, default ``20``; ``include/eon/ParametersOptions.h:225``; section Saddle Search
* ``eon:confine_positive_t.min_force`` — ``double``, default ``0.5``; ``include/eon/ParametersOptions.h:226``; section Saddle Search
* ``eon:confine_positive_t.scale_ratio`` — ``double``, default ``0.9``; ``include/eon/ParametersOptions.h:227``; section Saddle Search
* ``eon:confine_positive_t.boost`` — ``double``, default ``10.0``; ``include/eon/ParametersOptions.h:228``; section Saddle Search
* ``eon:confine_positive_t.min_active`` — ``long``, default ``30``; ``include/eon/ParametersOptions.h:229``; section Saddle Search
* ``eon:optimizer_options_t.method`` — ``OptType``, default ``OptType::CG``; ``include/eon/ParametersOptions.h:235``; section Optimizer
* ``eon:optimizer_options_t.xtsci_method`` — ``std::string``, default ``lbfgs``; ``include/eon/ParametersOptions.h:236``; section Optimizer
* ``eon:xtsci_t.method`` — ``std::string``, default ``lbfgs``; ``include/eon/ParametersOptions.h:238``; section Optimizer
* ``eon:xtsci_t.qn_step`` — ``std::string``, default ``lbfgs``; ``include/eon/ParametersOptions.h:239``; section Optimizer
* ``eon:xtsci_t.precon`` — ``std::string``, default ``none``; ``include/eon/ParametersOptions.h:240``; section Optimizer
* ``eon:xtsci_t.accept`` — ``std::string``, default ``energy``; ``include/eon/ParametersOptions.h:241``; section Optimizer
* ``eon:xtsci_t.highs`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:242``; section Optimizer
* ``eon:xtsci_t.manifold`` — ``std::string``, default ``euclidean``; ``include/eon/ParametersOptions.h:243``; section Optimizer
* ``eon:optimizer_options_t.convergence_metric`` — ``std::string``, default ``norm``; ``include/eon/ParametersOptions.h:245``; section Optimizer
* ``eon:optimizer_options_t.convergence_metric_label`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:246``; section Optimizer
* ``eon:optimizer_options_t.max_iterations`` — ``size_t``, default ``1000``; ``include/eon/ParametersOptions.h:247``; section Optimizer
* ``eon:optimizer_options_t.max_move`` — ``double``, default ``0.2``; ``include/eon/ParametersOptions.h:248``; section Optimizer
* ``eon:optimizer_options_t.converged_force`` — ``double``, default ``0.01``; ``include/eon/ParametersOptions.h:249``; section Optimizer
* ``eon:optimizer_options_t.time_step_input`` — ``double``, default ``1.0``; ``include/eon/ParametersOptions.h:250``; section Optimizer
* ``eon:optimizer_options_t.time_step`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:251``; section Optimizer
* ``eon:optimizer_options_t.max_time_step_input`` — ``double``, default ``2.5``; ``include/eon/ParametersOptions.h:252``; section Optimizer
* ``eon:optimizer_options_t.max_time_step`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:253``; section Optimizer
* ``eon:refine_t.method`` — ``OptType``, default ``OptType::None``; ``include/eon/ParametersOptions.h:255``; section Optimizer
* ``eon:refine_t.threshold`` — ``double``, default ``0.5``; ``include/eon/ParametersOptions.h:256``; section Optimizer
* ``eon:lbfgs_t.memory`` — ``long``, default ``20``; ``include/eon/ParametersOptions.h:259``; section Optimizer
* ``eon:lbfgs_t.inverse_curvature`` — ``double``, default ``0.01``; ``include/eon/ParametersOptions.h:260``; section Optimizer
* ``eon:lbfgs_t.max_inverse_curvature`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:261``; section Optimizer
* ``eon:lbfgs_t.auto_scale`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:262``; section Optimizer
* ``eon:lbfgs_t.angle_reset`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:263``; section Optimizer
* ``eon:lbfgs_t.distance_reset`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:264``; section Optimizer
* ``eon:lbfgs_t.curvature`` — ``std::string``, default ``reset``; ``include/eon/ParametersOptions.h:267``; section Optimizer
* ``eon:lbfgs_t.project_rigid`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:270``; section Optimizer
* ``eon:lbfgs_t.secant`` — ``std::string``, default ``standard``; ``include/eon/ParametersOptions.h:271``; section Optimizer
* ``eon:lbfgs_t.precon`` — ``std::string``, default ``none``; ``include/eon/ParametersOptions.h:272``; section Optimizer
* ``eon:lbfgs_t.step`` — ``std::string``, default ``lbfgs``; ``include/eon/ParametersOptions.h:273``; section Optimizer
* ``eon:lbfgs_t.h0`` — ``std::string``, default ``sy_yy``; ``include/eon/ParametersOptions.h:274``; section Optimizer
* ``eon:lbfgs_t.accept`` — ``std::string``, default ``energy``; ``include/eon/ParametersOptions.h:275``; section Optimizer
* ``eon:lbfgs_t.extra_updates`` — ``long``, default ``0``; ``include/eon/ParametersOptions.h:276``; section Optimizer
* ``eon:lbfgs_t.cautious_eps`` — ``double``, default ``1.0e-6``; ``include/eon/ParametersOptions.h:277``; section Optimizer
* ``eon:lbfgs_t.cautious_alpha`` — ``double``, default ``0.01``; ``include/eon/ParametersOptions.h:278``; section Optimizer
* ``eon:lbfgs_t.precon_A`` — ``double``, default ``3.0``; ``include/eon/ParametersOptions.h:279``; section Optimizer
* ``eon:lbfgs_t.precon_mu`` — ``double``, default ``1.0``; ``include/eon/ParametersOptions.h:280``; section Optimizer
* ``eon:lbfgs_t.precon_rcut`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:281``; section Optimizer
* ``eon:cg_t.no_overshooting`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:284``; section Optimizer
* ``eon:cg_t.knock_out_max_move`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:285``; section Optimizer
* ``eon:cg_t.line_search`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:286``; section Optimizer
* ``eon:cg_t.line_converged`` — ``double``, default ``0.1``; ``include/eon/ParametersOptions.h:287``; section Optimizer
* ``eon:cg_t.line_search_max_iter`` — ``long``, default ``10``; ``include/eon/ParametersOptions.h:288``; section Optimizer
* ``eon:cg_t.max_iter_before_reset`` — ``long``, default ``0``; ``include/eon/ParametersOptions.h:289``; section Optimizer
* ``eon:quickmin_t.steepest_descent`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:292``; section Optimizer
* ``eon:sd_t.alpha`` — ``double``, default ``0.1``; ``include/eon/ParametersOptions.h:295``; section Optimizer
* ``eon:sd_t.two_point`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:296``; section Optimizer
* ``eon:dimer_options_t.rotation_angle`` — ``double``, default ``0.005``; ``include/eon/ParametersOptions.h:302``; section Dimer
* ``eon:dimer_options_t.improved`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:303``; section Dimer
* ``eon:dimer_options_t.converged_angle`` — ``double``, default ``5.0``; ``include/eon/ParametersOptions.h:304``; section Dimer
* ``eon:dimer_options_t.max_iterations`` — ``long``, default ``1000``; ``include/eon/ParametersOptions.h:305``; section Dimer
* ``eon:dimer_options_t.opt_method`` — ``OptType``, default ``OptType::CG``; ``include/eon/ParametersOptions.h:306``; section Dimer
* ``eon:dimer_options_t.rotations_max`` — ``long``, default ``10``; ``include/eon/ParametersOptions.h:307``; section Dimer
* ``eon:dimer_options_t.rotations_min`` — ``long``, default ``1``; ``include/eon/ParametersOptions.h:308``; section Dimer
* ``eon:dimer_options_t.torque_max`` — ``double``, default ``1.0``; ``include/eon/ParametersOptions.h:309``; section Dimer
* ``eon:dimer_options_t.torque_min`` — ``double``, default ``0.1``; ``include/eon/ParametersOptions.h:310``; section Dimer
* ``eon:dimer_options_t.remove_rotation`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:311``; section Dimer
* ``eon:dimer_options_t.rotation_backend`` — ``DimerRotationBackend``, default ``DimerRotationBackend::Classical``; ``include/eon/ParametersOptions.h:314``; section Dimer
* ``eon:dimer_options_t.lor_residual_tol`` — ``double``, default ``0.1``; ``include/eon/ParametersOptions.h:317``; section Dimer
* ``eon:gpr_dimer_options_t.rotation_angle`` — ``double``, default ``0.005``; ``include/eon/ParametersOptions.h:322``; section GPR Dimer
* ``eon:gpr_dimer_options_t.converged_angle`` — ``double``, default ``0.08``; ``include/eon/ParametersOptions.h:323``; section GPR Dimer
* ``eon:gpr_dimer_options_t.relax_conv_angle`` — ``double``, default ``0.001``; ``include/eon/ParametersOptions.h:324``; section GPR Dimer
* ``eon:gpr_dimer_options_t.init_rotations_max`` — ``long``, default ``6``; ``include/eon/ParametersOptions.h:325``; section GPR Dimer
* ``eon:gpr_dimer_options_t.relax_rotations_max`` — ``long``, default ``10``; ``include/eon/ParametersOptions.h:326``; section GPR Dimer
* ``eon:gpr_dimer_options_t.divisor_t_dimer_gp`` — ``long``, default ``10``; ``include/eon/ParametersOptions.h:327``; section GPR Dimer
* ``eon:gpr_dimer_options_t.max_outer_iterations`` — ``long``, default ``300``; ``include/eon/ParametersOptions.h:328``; section GPR Dimer
* ``eon:gpr_dimer_options_t.max_inner_iterations`` — ``long``, default ``1000``; ``include/eon/ParametersOptions.h:329``; section GPR Dimer
* ``eon:gpr_dimer_options_t.midpoint_max_disp`` — ``double``, default ``0.5``; ``include/eon/ParametersOptions.h:330``; section GPR Dimer
* ``eon:gpr_dimer_options_t.rot_opt_method`` — ``std::string``, default ``lbfgs``; ``include/eon/ParametersOptions.h:331``; section GPR Dimer
* ``eon:gpr_dimer_options_t.trans_opt_method`` — ``std::string``, default ``lbfgs``; ``include/eon/ParametersOptions.h:332``; section GPR Dimer
* ``eon:gpr_dimer_options_t.active_radius`` — ``double``, default ``5.0``; ``include/eon/ParametersOptions.h:333``; section GPR Dimer
* ``eon:gpr_dimer_options_t.dimer_sep`` — ``double``, default ``0.01``; ``include/eon/ParametersOptions.h:334``; section GPR Dimer
* ``eon:gpr_dimer_options_t.conv_step`` — ``double``, default ``0.1``; ``include/eon/ParametersOptions.h:335``; section GPR Dimer
* ``eon:gpr_dimer_options_t.max_step`` — ``double``, default ``0.1``; ``include/eon/ParametersOptions.h:336``; section GPR Dimer
* ``eon:gpr_dimer_options_t.ratio_at_limit`` — ``double``, default ``0.66667``; ``include/eon/ParametersOptions.h:337``; section GPR Dimer
* ``eon:gpr_dimer_options_t.init_rot_gp`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:338``; section GPR Dimer
* ``eon:gpr_dimer_options_t.init_trans_gp`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:339``; section GPR Dimer
* ``eon:gpr_dimer_options_t.many_iterations`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:340``; section GPR Dimer
* ``eon:gpr_params_t.hyper_opt_method`` — ``std::string``, default ``scg``; ``include/eon/ParametersOptions.h:342``; section GPR Dimer
* ``eon:gpr_params_t.sigma2`` — ``double``, default ``1e-8``; ``include/eon/ParametersOptions.h:343``; section GPR Dimer
* ``eon:gpr_params_t.jitter_sigma2`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:344``; section GPR Dimer
* ``eon:gpr_params_t.noise_sigma2`` — ``double``, default ``1e-8``; ``include/eon/ParametersOptions.h:345``; section GPR Dimer
* ``eon:gpr_params_t.prior_mu`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:346``; section GPR Dimer
* ``eon:gpr_params_t.prior_sigma2`` — ``double``, default ``1.0``; ``include/eon/ParametersOptions.h:347``; section GPR Dimer
* ``eon:gpr_params_t.prior_nu`` — ``long``, default ``20``; ``include/eon/ParametersOptions.h:348``; section GPR Dimer
* ``eon:opt_params_t.check_derivatives`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:351``; section GPR Dimer
* ``eon:opt_params_t.max_iterations`` — ``int``, default ``400``; ``include/eon/ParametersOptions.h:352``; section GPR Dimer
* ``eon:opt_params_t.tol_func`` — ``double``, default ``1e-4``; ``include/eon/ParametersOptions.h:353``; section GPR Dimer
* ``eon:opt_params_t.tol_sol`` — ``double``, default ``1e-4``; ``include/eon/ParametersOptions.h:354``; section GPR Dimer
* ``eon:opt_params_t.lambda_limit`` — ``long``, default ``static_cast<long>(1e17)``; ``include/eon/ParametersOptions.h:355``; section GPR Dimer
* ``eon:opt_params_t.lambda_init`` — ``long``, default ``10``; ``include/eon/ParametersOptions.h:356``; section GPR Dimer
* ``eon:prune_params_t.use_prune`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:359``; section GPR Dimer
* ``eon:prune_params_t.begin`` — ``int``, default ``8``; ``include/eon/ParametersOptions.h:360``; section GPR Dimer
* ``eon:prune_params_t.n_vals`` — ``int``, default ``3``; ``include/eon/ParametersOptions.h:361``; section GPR Dimer
* ``eon:prune_params_t.threshold`` — ``double``, default ``0.5``; ``include/eon/ParametersOptions.h:362``; section GPR Dimer
* ``eon:debug_params_t.report_level`` — ``int``, default ``1``; ``include/eon/ParametersOptions.h:365``; section GPR Dimer
* ``eon:debug_params_t.debug_level`` — ``int``, default ``2``; ``include/eon/ParametersOptions.h:366``; section GPR Dimer
* ``eon:debug_params_t.out_dir`` — ``std::string``, default ``output``; ``include/eon/ParametersOptions.h:367``; section GPR Dimer
* ``eon:debug_params_t.pos_file`` — ``std::string``, default ``position``; ``include/eon/ParametersOptions.h:368``; section GPR Dimer
* ``eon:debug_params_t.energy_file`` — ``std::string``, default ``energy``; ``include/eon/ParametersOptions.h:369``; section GPR Dimer
* ``eon:debug_params_t.grad_file`` — ``std::string``, default ``gradient``; ``include/eon/ParametersOptions.h:370``; section GPR Dimer
* ``eon:debug_params_t.out_ext`` — ``std::string``, default ``dat``; ``include/eon/ParametersOptions.h:371``; section GPR Dimer
* ``eon:debug_params_t.offset_mid_point`` — ``double``, default ``3.0``; ``include/eon/ParametersOptions.h:372``; section GPR Dimer
* ``eon:debug_params_t.dy`` — ``double``, default ``0.1``; ``include/eon/ParametersOptions.h:373``; section GPR Dimer
* ``eon:debug_params_t.dz`` — ``double``, default ``0.1``; ``include/eon/ParametersOptions.h:374``; section GPR Dimer
* ``eon:gp_surrogate_options_t.enabled`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:380``; section GP Surrogate
* ``eon:gp_surrogate_options_t.sub_job`` — ``JobType``, default ``JobType::Unknown``; ``include/eon/ParametersOptions.h:381``; section GP Surrogate
* ``eon:gp_surrogate_options_t.uncertainty`` — ``double``, default ``0.05``; ``include/eon/ParametersOptions.h:382``; section GP Surrogate
* ``eon:gp_surrogate_options_t.linear_path_always`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:383``; section GP Surrogate
* ``eon:gp_surrogate_options_t.potential`` — ``PotType``, default ``PotType::CatLearn``; ``include/eon/ParametersOptions.h:384``; section GP Surrogate
* ``eon:catlearn_options_t.path`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:389``; section CatLearn
* ``eon:catlearn_options_t.model`` — ``std::string``, default ``gp``; ``include/eon/ParametersOptions.h:390``; section CatLearn
* ``eon:catlearn_options_t.prior`` — ``std::string``, default ``median``; ``include/eon/ParametersOptions.h:391``; section CatLearn
* ``eon:catlearn_options_t.use_deriv`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:392``; section CatLearn
* ``eon:catlearn_options_t.use_fingerprint`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:393``; section CatLearn
* ``eon:catlearn_options_t.parallel`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:394``; section CatLearn
* ``eon:ase_orca_options_t.path`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:399``; section ASE_ORCA
* ``eon:ase_orca_options_t.nproc`` — ``std::string``, default ``1``; ``include/eon/ParametersOptions.h:400``; section ASE_ORCA
* ``eon:ase_orca_options_t.simpleinput`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:401``; section ASE_ORCA
* ``eon:ase_orca_options_t.charge`` — ``int``, default ``0``; ``include/eon/ParametersOptions.h:402``; section ASE_ORCA
* ``eon:ase_orca_options_t.multiplicity`` — ``int``, default ``1``; ``include/eon/ParametersOptions.h:403``; section ASE_ORCA
* ``eon:ase_nwchem_options_t.path`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:408``; section ASE_NWCHEM
* ``eon:ase_nwchem_options_t.nproc`` — ``std::string``, default ``1``; ``include/eon/ParametersOptions.h:409``; section ASE_NWCHEM
* ``eon:ase_nwchem_options_t.mpi_launcher`` — ``std::string``, default ``mpirun``; ``include/eon/ParametersOptions.h:411``; section ASE_NWCHEM
* ``eon:ase_nwchem_options_t.multiplicity`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:412``; section ASE_NWCHEM
* ``eon:ase_nwchem_options_t.scf_thresh`` — ``double``, default ``1e-5``; ``include/eon/ParametersOptions.h:413``; section ASE_NWCHEM
* ``eon:ase_nwchem_options_t.scf_maxiter`` — ``long``, default ``200``; ``include/eon/ParametersOptions.h:414``; section ASE_NWCHEM
* ``eon:ase_nwchem_options_t.basis`` — ``std::string``, default ``3-21G``; ``include/eon/ParametersOptions.h:415``; section ASE_NWCHEM
* ``eon:ase_nwchem_options_t.memory`` — ``std::string``, default ``2 gb``; ``include/eon/ParametersOptions.h:416``; section ASE_NWCHEM
* ``eon:metatomic_options_t.model_path`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:421``; section Metatomic
* ``eon:metatomic_options_t.device`` — ``std::string``, default ``cpu``; ``include/eon/ParametersOptions.h:422``; section Metatomic
* ``eon:metatomic_options_t.length_unit`` — ``std::string``, default ``angstrom``; ``include/eon/ParametersOptions.h:423``; section Metatomic
* ``eon:metatomic_options_t.extensions_directory`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:424``; section Metatomic
* ``eon:metatomic_options_t.check_consistency`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:425``; section Metatomic
* ``eon:metatomic_options_t.uncertainty_threshold`` — ``double``, default ``-1.0``; ``include/eon/ParametersOptions.h:426``; section Metatomic
* ``eon:metatomic_options_t.energy_output`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:429``; section Metatomic
* ``eon:metatomic_options_t.energy_uncertainty_output`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:430``; section Metatomic
* ``eon:metatomic_options_t.force_output`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:433``; section Metatomic
* ``eon:metatomic_options_t.non_conservative`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:436``; section Metatomic
* ``eon:metatomic_options_t.random_rotation`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:439``; section Metatomic
* ``eon:metatomic_options_t.n_symmetry_rotations`` — ``long``, default ``0``; ``include/eon/ParametersOptions.h:443``; section Metatomic
* ``eon:metatomic_options_t.deterministic`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:447``; section Metatomic
* ``eon:metatomic_options_t.deterministic_strict`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:450``; section Metatomic
* ``eon:variants_t.base`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:452``; section Metatomic
* ``eon:variants_t.energy`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:453``; section Metatomic
* ``eon:variants_t.energy_uncertainty`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:454``; section Metatomic
* ``eon:variants_t.force`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:455``; section Metatomic
* ``eon:lanczos_options_t.tolerance`` — ``double``, default ``0.01``; ``include/eon/ParametersOptions.h:464``; section Lanczos
* ``eon:lanczos_options_t.max_iterations`` — ``long``, default ``20``; ``include/eon/ParametersOptions.h:465``; section Lanczos
* ``eon:lanczos_options_t.quit_early`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:466``; section Lanczos
* ``eon:lanczos_options_t.phva_atoms`` — ``std::string``, default ``All``; ``include/eon/ParametersOptions.h:467``; section Lanczos
* ``eon:davidson_options_t.tolerance`` — ``double``, default ``0.01``; ``include/eon/ParametersOptions.h:472``; section Davidson
* ``eon:davidson_options_t.max_iterations`` — ``long``, default ``20``; ``include/eon/ParametersOptions.h:473``; section Davidson
* ``eon:davidson_options_t.diagonal_preconditioner`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:476``; section Davidson
* ``eon:davidson_options_t.phva_atoms`` — ``std::string``, default ``All``; ``include/eon/ParametersOptions.h:478``; section Davidson
* ``eon:prefactor_options_t.default_value`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:483``; section Prefactor
* ``eon:prefactor_options_t.max_value`` — ``double``, default ``1e+21``; ``include/eon/ParametersOptions.h:484``; section Prefactor
* ``eon:prefactor_options_t.min_value`` — ``double``, default ``1e+9``; ``include/eon/ParametersOptions.h:485``; section Prefactor
* ``eon:prefactor_options_t.within_radius`` — ``double``, default ``3.3``; ``include/eon/ParametersOptions.h:486``; section Prefactor
* ``eon:prefactor_options_t.min_displacement`` — ``double``, default ``0.25``; ``include/eon/ParametersOptions.h:487``; section Prefactor
* ``eon:prefactor_options_t.rate`` — ``std::string``, default ``htst``; ``include/eon/ParametersOptions.h:488``; section Prefactor
* ``eon:prefactor_options_t.configuration`` — ``std::string``, default ``reactant``; ``include/eon/ParametersOptions.h:489``; section Prefactor
* ``eon:prefactor_options_t.all_free_atoms`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:490``; section Prefactor
* ``eon:prefactor_options_t.filter_scheme`` — ``std::string``, default ``fraction``; ``include/eon/ParametersOptions.h:491``; section Prefactor
* ``eon:prefactor_options_t.filter_fraction`` — ``double``, default ``0.90``; ``include/eon/ParametersOptions.h:492``; section Prefactor
* ``eon:hessian_options_t.phva_atoms`` — ``std::string``, default ``All``; ``include/eon/ParametersOptions.h:500``; section Hessian
* ``eon:hessian_options_t.zero_freq_value`` — ``double``, default ``1e-6``; ``include/eon/ParametersOptions.h:501``; section Hessian
* ``eon:hessian_options_t.fd_scheme`` — ``std::string``, default ``one_sided``; ``include/eon/ParametersOptions.h:506``; section Hessian
* ``eon:hessian_options_t.resume`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:508``; section Hessian
* ``eon:hessian_options_t.checkpoint_path`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:510``; section Hessian
* ``eon:hessian_options_t.write_modes`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:514``; section Hessian
* ``eon:neb_options_t.image_count`` — ``long``, default ``5``; ``include/eon/ParametersOptions.h:519``; section Nudged Elastic Band
* ``eon:neb_options_t.max_iterations`` — ``long``, default ``1000``; ``include/eon/ParametersOptions.h:520``; section Nudged Elastic Band
* ``eon:neb_options_t.opt_method`` — ``OptType``, default ``OptType::LBFGS``; ``include/eon/ParametersOptions.h:521``; section Nudged Elastic Band
* ``eon:neb_options_t.force_tolerance`` — ``double``, default ``0.01``; ``include/eon/ParametersOptions.h:522``; section Nudged Elastic Band
* ``eon:mmf_peak_options_t.enabled`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:524``; section Nudged Elastic Band
* ``eon:mmf_peak_options_t.tolerance`` — ``double``, default ``0.05``; ``include/eon/ParametersOptions.h:525``; section Nudged Elastic Band
* ``eon:neb_options_t.match_endpoints`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:528``; section Nudged Elastic Band
* ``eon:neb_options_t.match_method`` — ``std::string``, default ``ira``; ``include/eon/ParametersOptions.h:530``; section Nudged Elastic Band
* ``eon:spring_options_t.constant`` — ``double``, default ``5.0``; ``include/eon/ParametersOptions.h:533``; section Nudged Elastic Band
* ``eon:spring_options_t.use_elastic_band`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:534``; section Nudged Elastic Band
* ``eon:spring_options_t.doubly_nudged`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:535``; section Nudged Elastic Band
* ``eon:spring_options_t.use_switching`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:536``; section Nudged Elastic Band
* ``eon:energy_weighting_t.enabled`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:539``; section Nudged Elastic Band
* ``eon:energy_weighting_t.trigger`` — ``double``, default ``10.0``; ``include/eon/ParametersOptions.h:540``; section Nudged Elastic Band
* ``eon:energy_weighting_t.k_min`` — ``double``, default ``0.97``; ``include/eon/ParametersOptions.h:541``; section Nudged Elastic Band
* ``eon:energy_weighting_t.k_max`` — ``double``, default ``9.7``; ``include/eon/ParametersOptions.h:542``; section Nudged Elastic Band
* ``eon:onsager_machlup_t.enabled`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:545``; section Nudged Elastic Band
* ``eon:onsager_machlup_t.optimize_k`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:546``; section Nudged Elastic Band
* ``eon:onsager_machlup_t.k_scale`` — ``double``, default ``1.0``; ``include/eon/ParametersOptions.h:547``; section Nudged Elastic Band
* ``eon:onsager_machlup_t.k_min`` — ``double``, default ``0.1``; ``include/eon/ParametersOptions.h:548``; section Nudged Elastic Band
* ``eon:onsager_machlup_t.k_max`` — ``double``, default ``100.0``; ``include/eon/ParametersOptions.h:549``; section Nudged Elastic Band
* ``eon:climbing_image_options_t.enabled`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:554``; section Nudged Elastic Band
* ``eon:climbing_image_options_t.converged_only`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:555``; section Nudged Elastic Band
* ``eon:climbing_image_options_t.band_slack`` — ``double``, default ``10.0``; ``include/eon/ParametersOptions.h:559``; section Nudged Elastic Band
* ``eon:climbing_image_options_t.use_old_tangent`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:560``; section Nudged Elastic Band
* ``eon:climbing_image_options_t.trigger_force`` — ``double``, default ``std::numeric_limits<double>::infinity()``; ``include/eon/ParametersOptions.h:561``; section Nudged Elastic Band
* ``eon:climbing_image_options_t.trigger_factor`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:562``; section Nudged Elastic Band
* ``eon:hybrid_dimer_t.use_mmf`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:565``; section Nudged Elastic Band
* ``eon:hybrid_dimer_t.trigger_force`` — ``double``, default ``0.1``; ``include/eon/ParametersOptions.h:566``; section Nudged Elastic Band
* ``eon:hybrid_dimer_t.max_steps`` — ``long``, default ``1000``; ``include/eon/ParametersOptions.h:567``; section Nudged Elastic Band
* ``eon:hybrid_dimer_t.ci_stability_count`` — ``long``, default ``5``; ``include/eon/ParametersOptions.h:568``; section Nudged Elastic Band
* ``eon:hybrid_dimer_t.angle_tol`` — ``double``, default ``0.7071``; ``include/eon/ParametersOptions.h:569``; section Nudged Elastic Band
* ``eon:hybrid_dimer_t.trigger_factor`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:570``; section Nudged Elastic Band
* ``eon:hybrid_dimer_t.restore_unhelpful`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:573``; section Nudged Elastic Band
* ``eon:zoom_options_t.enabled`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:582``; section Nudged Elastic Band
* ``eon:zoom_options_t.alpha`` — ``double``, default ``0.5``; ``include/eon/ParametersOptions.h:583``; section Nudged Elastic Band
* ``eon:zoom_options_t.offset`` — ``int``, default ``1``; ``include/eon/ParametersOptions.h:584``; section Nudged Elastic Band
* ``eon:zoom_options_t.Mode`` — ``enum class``, default ``Auto, Manual``; ``include/eon/ParametersOptions.h:585``; section Nudged Elastic Band
* ``eon:zoom_options_t.mode`` — ``Mode``, default ``Mode::Auto``; ``include/eon/ParametersOptions.h:586``; section Nudged Elastic Band
* ``eon:zoom_options_t.activation_threshold`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:587``; section Nudged Elastic Band
* ``eon:zoom_options_t.Interpolation`` — ``enum class``, default ``Cubic, Linear``; ``include/eon/ParametersOptions.h:588``; section Nudged Elastic Band
* ``eon:zoom_options_t.interpolation`` — ``Interpolation``, default ``Interpolation::Cubic``; ``include/eon/ParametersOptions.h:589``; section Nudged Elastic Band
* ``eon:zoom_options_t.stability_count`` — ``int``, default ``5``; ``include/eon/ParametersOptions.h:590``; section Nudged Elastic Band
* ``eon:zoom_options_t.max_iterations`` — ``int``, default ``0``; ``include/eon/ParametersOptions.h:591``; section Nudged Elastic Band
* ``eon:path_initialization_t.method`` — ``NEBInit``, default ``NEBInit::LINEAR``; ``include/eon/ParametersOptions.h:595``; section Nudged Elastic Band
* ``eon:path_initialization_t.input_path`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:596``; section Nudged Elastic Band
* ``eon:path_initialization_t.max_iterations`` — ``int``, default ``5000``; ``include/eon/ParametersOptions.h:597``; section Nudged Elastic Band
* ``eon:path_initialization_t.nsteps`` — ``int``, default ``250``; ``include/eon/ParametersOptions.h:598``; section Nudged Elastic Band
* ``eon:path_initialization_t.max_move`` — ``double``, default ``0.1``; ``include/eon/ParametersOptions.h:599``; section Nudged Elastic Band
* ``eon:path_initialization_t.force_tolerance`` — ``double``, default ``0.001``; ``include/eon/ParametersOptions.h:600``; section Nudged Elastic Band
* ``eon:path_initialization_t.sidpp_alpha`` — ``double``, default ``0.33``; ``include/eon/ParametersOptions.h:601``; section Nudged Elastic Band; rgconf ``band.PathInitConfig.sidppAlphaGrow``; eOn ``path_initialization_t.sidpp_alpha``. Fraction of the open gap at which the alternating growth places a new image. eOn defaults to 0.33 and the schema to 0.5.
* ``eon:path_initialization_t.sidpp_frontier_tol`` — ``double``, default ``0.01``; ``include/eon/ParametersOptions.h:602``; section Nudged Elastic Band; rgconf ``band.PathInitConfig.sidppFrontierTolerance``; eOn ``path_initialization_t.sidpp_frontier_tol``. Residual above which a growth stage relaxes again before the next image is added.
* ``eon:path_initialization_t.sidpp_reparam`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:603``; section Nudged Elastic Band; rgconf ``band.PathInitConfig.sidppReparameterize``; eOn ``path_initialization_t.sidpp_reparam``. Places the grown path at equal arc length before the last relaxation.
* ``eon:path_initialization_t.sidpp_ideal_ksp`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:604``; section Nudged Elastic Band
* ``eon:path_initialization_t.opt_method`` — ``OptType``, default ``OptType::LBFGS``; ``include/eon/ParametersOptions.h:605``; section Nudged Elastic Band
* ``eon:path_initialization_t.oversampling`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:606``; section Nudged Elastic Band
* ``eon:path_initialization_t.oversampling_factor`` — ``int``, default ``3``; ``include/eon/ParametersOptions.h:607``; section Nudged Elastic Band
* ``eon:endpoint_options_t.minimize`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:611``; section Nudged Elastic Band
* ``eon:endpoint_options_t.use_path_file`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:612``; section Nudged Elastic Band
* ``eon:solid_state_options_t.enabled`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:618``; section Nudged Elastic Band
* ``eon:solid_state_options_t.weight`` — ``double``, default ``1.0``; ``include/eon/ParametersOptions.h:619``; section Nudged Elastic Band
* ``eon:solid_state_options_t.pressure`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:622``; section Nudged Elastic Band
* ``eon:dynamics_options_t.time_step_input`` — ``double``, default ``1.0``; ``include/eon/ParametersOptions.h:628``; section Molecular Dynamics
* ``eon:dynamics_options_t.time_step`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:629``; section Molecular Dynamics
* ``eon:dynamics_options_t.time_input`` — ``double``, default ``1000.0``; ``include/eon/ParametersOptions.h:630``; section Molecular Dynamics
* ``eon:dynamics_options_t.time`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:631``; section Molecular Dynamics
* ``eon:dynamics_options_t.steps`` — ``long``, default ``0``; ``include/eon/ParametersOptions.h:632``; section Molecular Dynamics
* ``eon:parallel_replica_options_t.refine_transition`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:637``; section Parallel Replica
* ``eon:parallel_replica_options_t.auto_stop`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:638``; section Parallel Replica
* ``eon:parallel_replica_options_t.dephase_loop_stop`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:639``; section Parallel Replica
* ``eon:parallel_replica_options_t.dephase_time_input`` — ``double``, default ``1000.0``; ``include/eon/ParametersOptions.h:640``; section Parallel Replica
* ``eon:parallel_replica_options_t.dephase_time`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:641``; section Parallel Replica
* ``eon:parallel_replica_options_t.dephase_loop_max`` — ``long``, default ``5``; ``include/eon/ParametersOptions.h:642``; section Parallel Replica
* ``eon:parallel_replica_options_t.state_check_interval_input`` — ``double``, default ``1000.0``; ``include/eon/ParametersOptions.h:643``; section Parallel Replica
* ``eon:parallel_replica_options_t.state_check_interval`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:644``; section Parallel Replica
* ``eon:parallel_replica_options_t.record_interval_input`` — ``double``, default ``50.0``; ``include/eon/ParametersOptions.h:645``; section Parallel Replica
* ``eon:parallel_replica_options_t.record_interval`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:646``; section Parallel Replica
* ``eon:parallel_replica_options_t.corr_time_input`` — ``double``, default ``1000.0``; ``include/eon/ParametersOptions.h:647``; section Parallel Replica
* ``eon:parallel_replica_options_t.corr_time`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:648``; section Parallel Replica
* ``eon:tad_options_t.low_temperature`` — ``double``, default ``300.0``; ``include/eon/ParametersOptions.h:653``; section Temperature Accelerated Dynamics
* ``eon:tad_options_t.min_prefactor`` — ``double``, default ``0.001``; ``include/eon/ParametersOptions.h:654``; section Temperature Accelerated Dynamics
* ``eon:tad_options_t.confidence`` — ``double``, default ``0.001``; ``include/eon/ParametersOptions.h:655``; section Temperature Accelerated Dynamics
* ``eon:thermostat_options_t.kind`` — ``std::string``, default ``none``; ``include/eon/ParametersOptions.h:660``; section Thermostat
* ``eon:thermostat_options_t.andersen_alpha`` — ``double``, default ``1.0``; ``include/eon/ParametersOptions.h:661``; section Thermostat
* ``eon:thermostat_options_t.andersen_tcol_input`` — ``double``, default ``100.0``; ``include/eon/ParametersOptions.h:662``; section Thermostat
* ``eon:thermostat_options_t.andersen_tcol`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:663``; section Thermostat
* ``eon:thermostat_options_t.nose_mass`` — ``double``, default ``1.0``; ``include/eon/ParametersOptions.h:664``; section Thermostat
* ``eon:thermostat_options_t.langevin_friction_input`` — ``double``, default ``0.01``; ``include/eon/ParametersOptions.h:665``; section Thermostat
* ``eon:thermostat_options_t.langevin_friction`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:666``; section Thermostat
* ``eon:thermostat_options_t.path_beads`` — ``long``, default ``8``; ``include/eon/ParametersOptions.h:667``; section Thermostat
* ``eon:thermostat_options_t.path_springs`` — ``std::string``, default ``trotter``; ``include/eon/ParametersOptions.h:668``; section Thermostat
* ``eon:thermostat_options_t.path_eco_omega_max`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:669``; section Thermostat
* ``eon:thermostat_options_t.path_gle_file`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:670``; section Thermostat
* ``eon:thermostat_options_t.path_pile_tau_input`` — ``double``, default ``100.0``; ``include/eon/ParametersOptions.h:671``; section Thermostat
* ``eon:thermostat_options_t.path_pile_tau`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:672``; section Thermostat
* ``eon:thermostat_options_t.path_pile_scale`` — ``double``, default ``1.0``; ``include/eon/ParametersOptions.h:673``; section Thermostat
* ``eon:thermostat_options_t.path_seed`` — ``std::uint64_t``, default ``1``; ``include/eon/ParametersOptions.h:674``; section Thermostat
* ``eon:replica_exchange_options_t.temperature_distribution`` — ``std::string``, default ``exponential``; ``include/eon/ParametersOptions.h:679``; section Replica Exchange
* ``eon:replica_exchange_options_t.replicas`` — ``long``, default ``10``; ``include/eon/ParametersOptions.h:680``; section Replica Exchange
* ``eon:replica_exchange_options_t.exchange_trials`` — ``long``, default ``10``; ``include/eon/ParametersOptions.h:681``; section Replica Exchange
* ``eon:replica_exchange_options_t.sampling_time_input`` — ``double``, default ``1000.0``; ``include/eon/ParametersOptions.h:682``; section Replica Exchange
* ``eon:replica_exchange_options_t.sampling_time`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:683``; section Replica Exchange
* ``eon:replica_exchange_options_t.temperature_high`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:684``; section Replica Exchange
* ``eon:replica_exchange_options_t.temperature_low`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:685``; section Replica Exchange
* ``eon:replica_exchange_options_t.exchange_period_input`` — ``double``, default ``100.0``; ``include/eon/ParametersOptions.h:686``; section Replica Exchange
* ``eon:replica_exchange_options_t.exchange_period`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:687``; section Replica Exchange
* ``eon:hyperdynamics_options_t.bias_potential`` — ``std::string``, default ``none``; ``include/eon/ParametersOptions.h:692``; section Bond Boost / Hyperdynamics
* ``eon:hyperdynamics_options_t.boost_atom_list`` — ``std::string``, default ``All``; ``include/eon/ParametersOptions.h:693``; section Bond Boost / Hyperdynamics
* ``eon:hyperdynamics_options_t.rmd_time_input`` — ``double``, default ``100.0``; ``include/eon/ParametersOptions.h:694``; section Bond Boost / Hyperdynamics
* ``eon:hyperdynamics_options_t.rmd_time`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:695``; section Bond Boost / Hyperdynamics
* ``eon:hyperdynamics_options_t.dvmax`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:696``; section Bond Boost / Hyperdynamics
* ``eon:hyperdynamics_options_t.qrr`` — ``double``, default ``0.2``; ``include/eon/ParametersOptions.h:697``; section Bond Boost / Hyperdynamics
* ``eon:hyperdynamics_options_t.prr`` — ``double``, default ``0.95``; ``include/eon/ParametersOptions.h:698``; section Bond Boost / Hyperdynamics
* ``eon:hyperdynamics_options_t.qcut`` — ``double``, default ``3.0``; ``include/eon/ParametersOptions.h:699``; section Bond Boost / Hyperdynamics
* ``eon:basin_hopping_options_t.displacement`` — ``double``, default ``0.5``; ``include/eon/ParametersOptions.h:704``; section Basin Hopping
* ``eon:basin_hopping_options_t.initial_random_structure_probability`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:705``; section Basin Hopping
* ``eon:basin_hopping_options_t.push_apart_distance`` — ``double``, default ``0.4``; ``include/eon/ParametersOptions.h:706``; section Basin Hopping
* ``eon:basin_hopping_options_t.steps`` — ``long``, default ``10000``; ``include/eon/ParametersOptions.h:707``; section Basin Hopping
* ``eon:basin_hopping_options_t.quenching_steps`` — ``long``, default ``0``; ``include/eon/ParametersOptions.h:708``; section Basin Hopping
* ``eon:basin_hopping_options_t.significant_structure`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:709``; section Basin Hopping
* ``eon:basin_hopping_options_t.single_atom_displace`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:710``; section Basin Hopping
* ``eon:basin_hopping_options_t.displacement_algorithm`` — ``std::string``, default ``standard``; ``include/eon/ParametersOptions.h:711``; section Basin Hopping
* ``eon:basin_hopping_options_t.displacement_distribution`` — ``std::string``, default ``uniform``; ``include/eon/ParametersOptions.h:712``; section Basin Hopping
* ``eon:basin_hopping_options_t.swap_probability`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:713``; section Basin Hopping
* ``eon:basin_hopping_options_t.jump_max`` — ``long``, default ``10``; ``include/eon/ParametersOptions.h:714``; section Basin Hopping
* ``eon:basin_hopping_options_t.jump_steps`` — ``long``, default ``0``; ``include/eon/ParametersOptions.h:715``; section Basin Hopping
* ``eon:basin_hopping_options_t.adjust_displacement`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:716``; section Basin Hopping
* ``eon:basin_hopping_options_t.adjust_period`` — ``long``, default ``10``; ``include/eon/ParametersOptions.h:717``; section Basin Hopping
* ``eon:basin_hopping_options_t.adjust_fraction`` — ``double``, default ``0.05``; ``include/eon/ParametersOptions.h:718``; section Basin Hopping
* ``eon:basin_hopping_options_t.target_ratio`` — ``double``, default ``0.5``; ``include/eon/ParametersOptions.h:719``; section Basin Hopping
* ``eon:basin_hopping_options_t.write_unique`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:720``; section Basin Hopping
* ``eon:basin_hopping_options_t.stop_energy`` — ``double``, default ``-std::numeric_limits<double>::max()``; ``include/eon/ParametersOptions.h:721``; section Basin Hopping
* ``eon:global_optimization_options_t.move_method`` — ``std::string``, default ``md``; ``include/eon/ParametersOptions.h:726``; section Global Optimization
* ``eon:global_optimization_options_t.decision_method`` — ``std::string``, default ``npew``; ``include/eon/ParametersOptions.h:727``; section Global Optimization
* ``eon:global_optimization_options_t.steps`` — ``long``, default ``10000``; ``include/eon/ParametersOptions.h:728``; section Global Optimization
* ``eon:global_optimization_options_t.beta`` — ``double``, default ``1.05``; ``include/eon/ParametersOptions.h:729``; section Global Optimization
* ``eon:global_optimization_options_t.alpha`` — ``double``, default ``1.02``; ``include/eon/ParametersOptions.h:730``; section Global Optimization
* ``eon:global_optimization_options_t.mdmin`` — ``long``, default ``3``; ``include/eon/ParametersOptions.h:731``; section Global Optimization
* ``eon:global_optimization_options_t.target_energy`` — ``double``, default ``-1.0e50``; ``include/eon/ParametersOptions.h:732``; section Global Optimization
* ``eon:monte_carlo_options_t.step_size`` — ``double``, default ``0.005``; ``include/eon/ParametersOptions.h:737``; section Monte Carlo
* ``eon:monte_carlo_options_t.steps`` — ``int``, default ``1000``; ``include/eon/ParametersOptions.h:738``; section Monte Carlo
* ``eon:bgsd_options_t.alpha`` — ``double``, default ``10.0``; ``include/eon/ParametersOptions.h:743``; section BGSD
* ``eon:bgsd_options_t.beta`` — ``double``, default ``0.2``; ``include/eon/ParametersOptions.h:744``; section BGSD
* ``eon:bgsd_options_t.gradient_finite_difference`` — ``double``, default ``0.000001``; ``include/eon/ParametersOptions.h:745``; section BGSD
* ``eon:bgsd_options_t.h_force_convergence`` — ``double``, default ``0.01``; ``include/eon/ParametersOptions.h:746``; section BGSD
* ``eon:bgsd_options_t.grad2energy_convergence`` — ``double``, default ``0.000001``; ``include/eon/ParametersOptions.h:747``; section BGSD
* ``eon:bgsd_options_t.grad2force_convergence`` — ``double``, default ``0.0001``; ``include/eon/ParametersOptions.h:748``; section BGSD
* ``eon:serve_options_t.host`` — ``std::string``, default ``localhost``; ``include/eon/ParametersOptions.h:753``; section Serve
* ``eon:serve_options_t.port`` — ``uint16_t``, default ``12345``; ``include/eon/ParametersOptions.h:754``; section Serve
* ``eon:serve_options_t.replicas`` — ``size_t``, default ``1``; ``include/eon/ParametersOptions.h:755``; section Serve
* ``eon:serve_options_t.gateway_port`` — ``uint16_t``, default ``0``; ``include/eon/ParametersOptions.h:756``; section Serve
* ``eon:serve_options_t.endpoints`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:757``; section Serve
* ``eon:artn_options_t.push_step_size`` — ``double``, default ``0.3``; ``include/eon/ParametersOptions.h:762``; section ARTn
* ``eon:artn_options_t.force_threshold`` — ``double``, default ``0.05``; ``include/eon/ParametersOptions.h:763``; section ARTn
* ``eon:artn_options_t.max_iterations`` — ``int``, default ``500``; ``include/eon/ParametersOptions.h:764``; section ARTn
* ``eon:artn_options_t.ninit`` — ``int``, default ``-1``; ``include/eon/ParametersOptions.h:765``; section ARTn
* ``eon:artn_options_t.nperp_limitation`` — ``std::string``, default ``default``; ``include/eon/ParametersOptions.h:770``; section ARTn
* ``eon:artn_options_t.nsmooth`` — ``int``, default ``-1``; ``include/eon/ParametersOptions.h:773``; section ARTn
* ``eon:artn_options_t.filin`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:780``; section ARTn
* ``eon:ira_options_t.distance_threshold`` — ``double``, default ``0.3``; ``include/eon/ParametersOptions.h:790``; section IRA
* ``eon:ira_options_t.symmetry_threshold`` — ``double``, default ``0.1``; ``include/eon/ParametersOptions.h:791``; section IRA
* ``eon:ira_options_t.use_pbc`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:792``; section IRA
* ``eon:debug_options_t.write_movies`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:797``; section Debug
* ``eon:debug_options_t.write_movies_interval`` — ``long``, default ``1``; ``include/eon/ParametersOptions.h:798``; section Debug
* ``eon:debug_options_t.write_deprecated_outs`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:799``; section Debug
* ``eon:debug_options_t.estimate_neb_eigenvalues`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:800``; section Debug
* ``eon:debug_options_t.neb_mmf`` — ``std::string``, default ``dimer``; ``include/eon/ParametersOptions.h:801``; section Debug
* ``eon:instanton_options_t.mode`` — ``std::string``, default ``splitting``; ``include/eon/ParametersOptions.h:810``; section Instanton
* ``eon:instanton_options_t.reactant_filename`` — ``std::string``, default ``reactant.con``; ``include/eon/ParametersOptions.h:811``; section Instanton
* ``eon:instanton_options_t.product_filename`` — ``std::string``, default ``product.con``; ``include/eon/ParametersOptions.h:812``; section Instanton
* ``eon:instanton_options_t.initial_path`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:815``; section Instanton
* ``eon:instanton_options_t.beads`` — ``long``, default ``256``; ``include/eon/ParametersOptions.h:816``; section Instanton
* ``eon:instanton_options_t.beta_hbar_omega`` — ``double``, default ``30.0``; ``include/eon/ParametersOptions.h:817``; section Instanton
* ``eon:instanton_options_t.max_iterations`` — ``long``, default ``5000``; ``include/eon/ParametersOptions.h:818``; section Instanton
* ``eon:instanton_options_t.force_tolerance`` — ``double``, default ``1e-3``; ``include/eon/ParametersOptions.h:819``; section Instanton
* ``eon:instanton_options_t.hessian_stride`` — ``long``, default ``1``; ``include/eon/ParametersOptions.h:821``; section Instanton
* ``eon:instanton_options_t.saddle_filename`` — ``std::string``, default ``saddle.con``; ``include/eon/ParametersOptions.h:823``; section Instanton
* ``eon:instanton_options_t.temperature`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:824``; section Instanton
* ``eon:instanton_options_t.temperatures`` — ``std::vector<double>``, default ``(no source default)``; ``include/eon/ParametersOptions.h:827``; section Instanton
* ``eon:instanton_options_t.half_ring`` — ``bool``, default ``true``; ``include/eon/ParametersOptions.h:830``; section Instanton
* ``eon:instanton_options_t.energy_shift`` — ``double``, default ``0.0``; ``include/eon/ParametersOptions.h:831``; section Instanton
* ``eon:instanton_options_t.initial_hessians`` — ``std::string``, default ``saddle``; ``include/eon/ParametersOptions.h:835``; section Instanton
* ``eon:instanton_options_t.bead_ladder`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:838``; section Instanton
* ``eon:instanton_options_t.hessian_final`` — ``std::string``, default ``recomputed``; ``include/eon/ParametersOptions.h:840``; section Instanton
* ``eon:instanton_options_t.springs`` — ``std::string``, default ``trotter``; ``include/eon/ParametersOptions.h:843``; section Instanton
* ``eon:instanton_options_t.pi_planes`` — ``long``, default ``0``; ``include/eon/ParametersOptions.h:847``; section Instanton
* ``eon:instanton_options_t.pi_beads`` — ``long``, default ``16``; ``include/eon/ParametersOptions.h:848``; section Instanton
* ``eon:instanton_options_t.pi_equilibration_steps`` — ``long``, default ``500``; ``include/eon/ParametersOptions.h:849``; section Instanton
* ``eon:instanton_options_t.pi_sampling_steps`` — ``long``, default ``2000``; ``include/eon/ParametersOptions.h:850``; section Instanton
* ``eon:instanton_options_t.pi_time_step`` — ``double``, default ``0.5``; ``include/eon/ParametersOptions.h:851``; section Instanton
* ``eon:instanton_options_t.pi_thermostat`` — ``std::string``, default ``pile``; ``include/eon/ParametersOptions.h:854``; section Instanton
* ``eon:instanton_options_t.pi_gle_file`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:855``; section Instanton
* ``eon:instanton_options_t.pi_pile_tau`` — ``double``, default ``100.0``; ``include/eon/ParametersOptions.h:856``; section Instanton
* ``eon:instanton_options_t.pi_pile_scale`` — ``double``, default ``1.0``; ``include/eon/ParametersOptions.h:857``; section Instanton
* ``eon:instanton_options_t.pi_seed`` — ``long``, default ``1``; ``include/eon/ParametersOptions.h:858``; section Instanton
* ``eon:instanton_options_t.pi_direction`` — ``std::string``, default ``mode``; ``include/eon/ParametersOptions.h:861``; section Instanton
* ``eon:instanton_options_t.pi_reactant_extent`` — ``double``, default ``0.5``; ``include/eon/ParametersOptions.h:864``; section Instanton
* ``eon:instanton_options_t.pi_recrossing_parents`` — ``long``, default ``0``; ``include/eon/ParametersOptions.h:869``; section Instanton
* ``eon:instanton_options_t.pi_recrossing_children`` — ``long``, default ``20``; ``include/eon/ParametersOptions.h:870``; section Instanton
* ``eon:instanton_options_t.pi_recrossing_time`` — ``double``, default ``100.0``; ``include/eon/ParametersOptions.h:871``; section Instanton
* ``eon:instanton_options_t.pi_recrossing_spacing`` — ``long``, default ``50``; ``include/eon/ParametersOptions.h:872``; section Instanton
* ``eon:oh_tst_options_t.reactant_filename`` — ``std::string``, default ``pos.con``; ``include/eon/ParametersOptions.h:880``; section OH_TST
* ``eon:oh_tst_options_t.product_filename`` — ``std::string``, default ``product.con``; ``include/eon/ParametersOptions.h:881``; section OH_TST
* ``eon:oh_tst_options_t.time_step`` — ``double``, default ``1.0``; ``include/eon/ParametersOptions.h:882``; section OH_TST
* ``eon:oh_tst_options_t.equil_steps`` — ``long``, default ``200``; ``include/eon/ParametersOptions.h:883``; section OH_TST
* ``eon:oh_tst_options_t.sample_steps`` — ``long``, default ``800``; ``include/eon/ParametersOptions.h:884``; section OH_TST
* ``eon:oh_tst_options_t.max_planes`` — ``long``, default ``200``; ``include/eon/ParametersOptions.h:885``; section OH_TST
* ``eon:oh_tst_options_t.plane_mass`` — ``double``, default ``50.0``; ``include/eon/ParametersOptions.h:886``; section OH_TST
* ``eon:oh_tst_options_t.alpha_rot`` — ``double``, default ``50.0``; ``include/eon/ParametersOptions.h:887``; section OH_TST
* ``eon:oh_tst_options_t.plane_time_step`` — ``double``, default ``0.1``; ``include/eon/ParametersOptions.h:888``; section OH_TST
* ``eon:oh_tst_options_t.ds_max`` — ``double``, default ``0.1``; ``include/eon/ParametersOptions.h:889``; section OH_TST
* ``eon:oh_tst_options_t.dtheta_max`` — ``double``, default ``0.05``; ``include/eon/ParametersOptions.h:890``; section OH_TST
* ``eon:oh_tst_options_t.force_tol`` — ``double``, default ``0.005``; ``include/eon/ParametersOptions.h:891``; section OH_TST
* ``eon:oh_tst_options_t.s_init`` — ``double``, default ``0.05``; ``include/eon/ParametersOptions.h:892``; section OH_TST
* ``eon:oh_tst_options_t.reactant_md_steps`` — ``long``, default ``20000``; ``include/eon/ParametersOptions.h:893``; section OH_TST
* ``eon:oh_tst_options_t.max_delta_a`` — ``double``, default ``10.0``; ``include/eon/ParametersOptions.h:894``; section OH_TST
* ``eon:oh_tst_options_t.thermostat`` — ``std::string``, default ``andersen``; ``include/eon/ParametersOptions.h:900``; section OH_TST
* ``eon:oh_tst_options_t.gle_a_file`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:901``; section OH_TST
* ``eon:oh_tst_options_t.symmetry_products`` — ``std::string``, default ``(no source default)``; ``include/eon/ParametersOptions.h:906``; section OH_TST
* ``eon:oh_tst_options_t.pmf_scan`` — ``bool``, default ``false``; ``include/eon/ParametersOptions.h:912``; section OH_TST
* ``eon:oh_tst_options_t.scan_planes`` — ``long``, default ``40``; ``include/eon/ParametersOptions.h:913``; section OH_TST
* ``rgmin:rgmin_abi_stamp_t.abi_major`` — ``uint16_t``, default ``(no source default)``; ``include/rgmin.h:70``
* ``rgmin:rgmin_abi_stamp_t.abi_minor`` — ``uint16_t``, default ``(no source default)``; ``include/rgmin.h:71``
* ``rgmin:rgmin_abi_stamp_t.layout_revision`` — ``uint16_t``, default ``(no source default)``; ``include/rgmin.h:72``
* ``rgmin:rgmin_control_t.maxiter`` — ``size_t``, default ``(no source default)``; ``include/rgmin.h:194``
* ``rgmin:rgmin_control_t.gtol`` — ``double``, default ``(no source default)``; ``include/rgmin.h:195``
* ``rgmin:rgmin_control_t.istep`` — ``double``, default ``(no source default)``; ``include/rgmin.h:196``
* ``rgmin:rgmin_control_t.memory`` — ``size_t``, default ``(no source default)``; ``include/rgmin.h:197``
* ``rgmin:rgmin_control_t.maxmove`` — ``double``, default ``(no source default)``; ``include/rgmin.h:200``
* ``rgmin:rgmin_control_t.noise_eps`` — ``double``, default ``(no source default)``; ``include/rgmin.h:202``
* ``rgmin:rgmin_control_t.noise_c`` — ``double``, default ``(no source default)``; ``include/rgmin.h:204``
* ``rgmin:rgmin_report_t.value`` — ``double``, default ``(no source default)``; ``include/rgmin.h:218``
* ``rgmin:rgmin_report_t.steps`` — ``size_t``, default ``(no source default)``; ``include/rgmin.h:219``
* ``rgmin:rgmin_report_t.grad_norm`` — ``double``, default ``(no source default)``; ``include/rgmin.h:220``
* ``rgmin:rgmin_report_t.stop`` — ``rgmin_stop_t``, default ``(no source default)``; ``include/rgmin.h:222``
* ``rgmin:rgmin_exp_t.a`` — ``double``, default ``(no source default)``; ``include/rgmin.h:363``
* ``rgmin:rgmin_exp_t.c_stab`` — ``double``, default ``(no source default)``; ``include/rgmin.h:364``
* ``rgmin:rgmin_exp_t.mu`` — ``double``, default ``(no source default)``; ``include/rgmin.h:365``
* ``rgmin:rgmin_exp_t.r_nn`` — ``double``, default ``(no source default)``; ``include/rgmin.h:366``
* ``rgmin:rgmin_exp_t.r_cut`` — ``double``, default ``(no source default)``; ``include/rgmin.h:367``
* ``rgmin:rgmin_exp_t.length_x`` — ``double``, default ``(no source default)``; ``include/rgmin.h:368``
* ``rgmin:rgmin_exp_t.length_y`` — ``double``, default ``(no source default)``; ``include/rgmin.h:369``
* ``rgmin:rgmin_exp_t.length_z`` — ``double``, default ``(no source default)``; ``include/rgmin.h:370``
* ``rgmin:rgmin_exp_t.recalc_mu`` — ``int32_t``, default ``(no source default)``; ``include/rgmin.h:371``
* ``rgmin:rgmin_bond_t.k`` — ``double``, default ``(no source default)``; ``include/rgmin.h:411``
* ``rgmin:rgmin_angle_t.k_theta`` — ``double``, default ``(no source default)``; ``include/rgmin.h:416``
* ``rgmin:rgmin_precon_t.struct_size`` — ``uint32_t``, default ``(no source default)``; ``include/rgmin.h:441``
* ``rgmin:rgmin_precon_t.kind`` — ``uint32_t``, default ``(no source default)``; ``include/rgmin.h:442``
* ``rgmin:rgmin_precon_t.natoms`` — ``size_t``, default ``(no source default)``; ``include/rgmin.h:446``
* ``rgmin:rgmin_precon_t.nghost`` — ``size_t``, default ``(no source default)``; ``include/rgmin.h:449``
* ``rgmin:rgmin_precon_t.npairs`` — ``size_t``, default ``(no source default)``; ``include/rgmin.h:453``
* ``rgmin:rgmin_precon_t.nanchor`` — ``size_t``, default ``(no source default)``; ``include/rgmin.h:456``
* ``rgmin:rgmin_precon_t.exp_a`` — ``double``, default ``(no source default)``; ``include/rgmin.h:457``
* ``rgmin:rgmin_precon_t.nbonds`` — ``size_t``, default ``(no source default)``; ``include/rgmin.h:459``
* ``rgmin:rgmin_precon_t.nangles`` — ``size_t``, default ``(no source default)``; ``include/rgmin.h:461``
* ``rgmin:rgmin_precon_t.dihedrals`` — ``int32_t``, default ``(no source default)``; ``include/rgmin.h:462``
* ``rgmin:rgmin_precon_t.geometry_kind`` — ``int32_t``, default ``(no source default)``; ``include/rgmin.h:463``
* ``rgmin:rgmin_scg_params_t.sigma0`` — ``double``, default ``(no source default)``; ``include/rgmin.h:1120``
* ``rgmin:rgmin_scg_params_t.lambda`` — ``double``, default ``(no source default)``; ``include/rgmin.h:1121``
* ``rgmin:rgmin_scg_params_t.lambda_limit`` — ``double``, default ``(no source default)``; ``include/rgmin.h:1122``
* ``rgmin:rgmin_scg_params_t.tol_sol`` — ``double``, default ``(no source default)``; ``include/rgmin.h:1123``
* ``rgmin:rgmin_scg_params_t.tol_func`` — ``double``, default ``(no source default)``; ``include/rgmin.h:1124``
* ``rgmin:rgmin_scg_params_t.conjugacy`` — ``rgmin_conjugacy_t``, default ``(no source default)``; ``include/rgmin.h:1128``
* ``rgmin:rgmin_scg_options_t.step_tolerance`` — ``int32_t``, default ``(no source default)``; ``include/rgmin.h:1171``
* ``rgmin:rgmin_rlbfgs_t.memory`` — ``size_t``, default ``(no source default)``; ``include/rgmin.h:1187``; rgconf ``optimizer.LbfgsConfig.memory``; eOn ``lbfgs_t.memory``. History length in both. The rgmin record defaults to 4 pairs where rgconf defaults to 20.
* ``rgmin:rgmin_rlbfgs_t.cautious_slope`` — ``double``, default ``(no source default)``; ``include/rgmin.h:1188``; rgconf ``optimizer.LbfgsConfig.cautiousEpsilon``; eOn ``lbfgs_t.cautious_eps``. Factor on the gradient norm in the cautious curvature test. The defaults differ: rgmin uses 1e-4 while the schema and eOn use 1e-6. The rgmin test fixes the exponent at one, while rgconf also exposes it as cautiousAlpha.
* ``rgmin:rgmin_rlbfgs_t.armijo`` — ``double``, default ``(no source default)``; ``include/rgmin.h:1189``; rgconf ``unmapped``; eOn ``none``. Sufficient-decrease constant of the Riemannian line search. rgconf has no line search constant for this method.
* ``rgmin:rgmin_rlbfgs_t.backtrack`` — ``double``, default ``(no source default)``; ``include/rgmin.h:1190``; rgconf ``unmapped``; eOn ``none``. Factor applied to a rejected trial length. rgconf has no matching field.
* ``rgmin:rgmin_rlbfgs_t.linesearch_max`` — ``size_t``, default ``(no source default)``; ``include/rgmin.h:1191``; rgconf ``unmapped``; eOn ``none``. Rejected trials allowed per iteration. The rgconf maxIterLineSearch field applies to the cg method only.
* ``rgmin:rgmin_rlbfgs_t.grad_ratio`` — ``double``, default ``(no source default)``; ``include/rgmin.h:1192``; rgconf ``unmapped``; eOn ``none``. Stop when the gradient norm falls below this fraction of the initial norm. rgconf convergence has no ratio criterion.
* ``rgmin:rgmin_eigen_params_t.kind`` — ``int32_t``, default ``(no source default)``; ``include/rgmin.h:1234``
* ``rgmin:rgmin_eigen_params_t.nev`` — ``uint32_t``, default ``(no source default)``; ``include/rgmin.h:1235``
* ``rgmin:rgmin_eigen_params_t.krylov`` — ``uint32_t``, default ``(no source default)``; ``include/rgmin.h:1236``
* ``rgmin:rgmin_eigen_params_t.max_iter`` — ``uint32_t``, default ``(no source default)``; ``include/rgmin.h:1237``
* ``rgmin:rgmin_eigen_params_t.tol`` — ``double``, default ``(no source default)``; ``include/rgmin.h:1238``
* ``rgmin:rgmin_lowest_mode_t.value`` — ``double``, default ``(no source default)``; ``include/rgmin.h:1246``
* ``rgmin:rgmin_lowest_mode_t.actions`` — ``size_t``, default ``(no source default)``; ``include/rgmin.h:1247``
* ``rgmin:rgmin_eigen_options_t.degree`` — ``uint32_t``, default ``(no source default)``; ``include/rgmin.h:1303``
* ``rgmin:rgmin_eigen_options_t.extra`` — ``uint32_t``, default ``(no source default)``; ``include/rgmin.h:1305``
* ``rgmin:rgmin_eigenpair_t.value`` — ``double``, default ``(no source default)``; ``include/rgmin.h:1314``
* ``rgmin:rgmin_eigenpair_t.actions`` — ``size_t``, default ``(no source default)``; ``include/rgmin.h:1315``
* ``rgmin:rgmin_eigenpair_ex_t.value`` — ``double``, default ``(no source default)``; ``include/rgmin.h:1341``
* ``rgmin:rgmin_eigenpair_ex_t.actions`` — ``size_t``, default ``(no source default)``; ``include/rgmin.h:1342``
* ``rgmin:rgmin_eigenpair_ex_t.residual`` — ``double``, default ``(no source default)``; ``include/rgmin.h:1343``
* ``rgmin:rgmin_eigenpair_ex_t.converged`` — ``int32_t``, default ``(no source default)``; ``include/rgmin.h:1344``
* ``rgmin:rgmin_control_t.maxiter`` — ``comment``, default ``100``; ``include/rgmin.h:206``; section control
* ``rgmin:rgmin_control_t.gtol`` — ``comment``, default ``1e-5``; ``include/rgmin.h:206``; section control
* ``rgmin:rgmin_control_t.istep`` — ``comment``, default ``1``; ``include/rgmin.h:206``; section control
* ``rgmin:rgmin_control_t.memory`` — ``comment``, default ``0``; ``include/rgmin.h:206``; section control
* ``rgmin:rgmin_control_t.maxmove`` — ``comment``, default ``0``; ``include/rgmin.h:207``; section control
* ``rgmin:rgmin_control_t.noise_eps`` — ``comment``, default ``0``; ``include/rgmin.h:208``; section control
* ``rgmin:rgmin_control_t.noise_c`` — ``comment``, default ``3``; ``include/rgmin.h:208``; section control
* ``rgsaddle:rgsaddle_version_t.major`` — ``uint32_t``, default ``(no source default)``; ``include/rgsaddle.h:124``
* ``rgsaddle:rgsaddle_version_t.minor`` — ``uint32_t``, default ``(no source default)``; ``include/rgsaddle.h:125``
* ``rgsaddle:rgsaddle_hyperplane_budget_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:232``
* ``rgsaddle:rgsaddle_hyperplane_budget_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:233``
* ``rgsaddle:rgsaddle_hyperplane_budget_t.thermal`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:234``
* ``rgsaddle:rgsaddle_hyperplane_budget_t.seed`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:235``
* ``rgsaddle:rgsaddle_hyperplane_budget_t.thermalize`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:236``
* ``rgsaddle:rgsaddle_hyperplane_budget_t.stride`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:237``
* ``rgsaddle:rgsaddle_hyperplane_budget_t.samples`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:238``
* ``rgsaddle:rgsaddle_hyperplane_budget_t.lambda`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:239``
* ``rgsaddle:rgsaddle_hyperplane_budget_t.positions`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:240``
* ``rgsaddle:rgsaddle_hyperplane_observation_t.constraint_force`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:245``
* ``rgsaddle:rgsaddle_hyperplane_observation_t.thermodynamic_derivative`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:248``
* ``rgsaddle:rgsaddle_hyperplane_ti_config_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:259``
* ``rgsaddle:rgsaddle_hyperplane_ti_config_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:260``
* ``rgsaddle:rgsaddle_hyperplane_ti_config_t.thermal`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:261``
* ``rgsaddle:rgsaddle_hyperplane_ti_config_t.nodes`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:262``
* ``rgsaddle:rgsaddle_hyperplane_ti_config_t.thermalize`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:263``
* ``rgsaddle:rgsaddle_hyperplane_ti_config_t.stride`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:264``
* ``rgsaddle:rgsaddle_hyperplane_ti_config_t.samples`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:265``
* ``rgsaddle:rgsaddle_hyperplane_ti_config_t.blocks`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:266``
* ``rgsaddle:rgsaddle_hyperplane_ti_config_t.seed`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:267``
* ``rgsaddle:rgsaddle_hyperplane_ti_result_t.free_energy`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:271``
* ``rgsaddle:rgsaddle_hyperplane_ti_result_t.error`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:272``
* ``rgsaddle:rgsaddle_hyperplane_ti_result_t.evaluations`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:273``
* ``rgsaddle:rgsaddle_surface_request_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:320``
* ``rgsaddle:rgsaddle_surface_request_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:321``
* ``rgsaddle:rgsaddle_surface_request_t.n_images`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:322``
* ``rgsaddle:rgsaddle_surface_request_t.n_atoms`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:323``
* ``rgsaddle:rgsaddle_surface_request_t.image`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:327``
* ``rgsaddle:rgsaddle_band_config_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:335``
* ``rgsaddle:rgsaddle_band_config_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:336``
* ``rgsaddle:rgsaddle_band_config_t.tangent`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:337``
* ``rgsaddle:rgsaddle_band_config_t.spring`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:338``
* ``rgsaddle:rgsaddle_band_config_t.projection`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:339``
* ``rgsaddle:rgsaddle_band_config_t.method`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:340``
* ``rgsaddle:rgsaddle_band_config_t.spring_k`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:341``
* ``rgsaddle:rgsaddle_band_config_t.ci_trigger_factor`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:345``
* ``rgsaddle:rgsaddle_band_config_t.ci_trigger_force`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:346``
* ``rgsaddle:rgsaddle_band_config_t.force_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:349``
* ``rgsaddle:rgsaddle_band_config_t.max_move`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:350``
* ``rgsaddle:rgsaddle_band_config_t.memory`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:351``
* ``rgsaddle:rgsaddle_report_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:356``
* ``rgsaddle:rgsaddle_report_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:357``
* ``rgsaddle:rgsaddle_report_t.status`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:358``
* ``rgsaddle:rgsaddle_report_t.evaluations`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:364``
* ``rgsaddle:rgsaddle_report_t.max_force`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:365``
* ``rgsaddle:rgsaddle_report_t.ci_index`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:367``
* ``rgsaddle:rgsaddle_report_t.iteration`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:368``
* ``rgsaddle:rgsaddle_report_t.curvature`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:373``
* ``rgsaddle:rgsaddle_report_t.rotations`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:374``
* ``rgsaddle:rgsaddle_solid_surface_request_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:416``
* ``rgsaddle:rgsaddle_solid_surface_request_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:417``
* ``rgsaddle:rgsaddle_solid_surface_request_t.n_images`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:418``
* ``rgsaddle:rgsaddle_solid_surface_request_t.n_atoms`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:419``
* ``rgsaddle:rgsaddle_solid_surface_request_t.image`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:425``
* ``rgsaddle:rgsaddle_minmode_config_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:571``
* ``rgsaddle:rgsaddle_minmode_config_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:572``
* ``rgsaddle:rgsaddle_minmode_config_t.kind`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:573``
* ``rgsaddle:rgsaddle_minmode_config_t.method`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:574``
* ``rgsaddle:rgsaddle_minmode_config_t.dr`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:575``
* ``rgsaddle:rgsaddle_minmode_config_t.rotation_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:576``
* ``rgsaddle:rgsaddle_minmode_config_t.max_rotations`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:577``
* ``rgsaddle:rgsaddle_minmode_config_t.krylov_dim`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:578``
* ``rgsaddle:rgsaddle_minmode_config_t.force_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:579``
* ``rgsaddle:rgsaddle_minmode_config_t.max_move`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:580``
* ``rgsaddle:rgsaddle_minmode_config_t.rotation_angle_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:588``
* ``rgsaddle:rgsaddle_minmode_config_t.difference`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:590``
* ``rgsaddle:rgsaddle_minmode_config_t.reserved`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:591``
* ``rgsaddle:rgsaddle_artn_config_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:600``; rgconf ``abi``; eOn ``none``. ABI record header, not a run parameter.
* ``rgsaddle:rgsaddle_artn_config_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:601``; rgconf ``abi``; eOn ``none``. Reserved ABI word that callers set to zero.
* ``rgsaddle:rgsaddle_artn_config_t.lanczos_start`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:602``; rgconf ``minmode.ArtnConfig.ninitPushSteps``; eOn ``artn_options_t.ninit``. Random-direction pushes before the first Lanczos solve. eOn uses -1 to defer to the pARTn default.
* ``rgsaddle:rgsaddle_artn_config_t.eigenvalue_threshold`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:603``; rgconf ``minmode.ArtnConfig.eigenvalueThreshold``; eOn ``none``. Eigenvalue below which activation starts, in eV per square Angstrom. The rgsaddle default is -0.01 and the schema default is -0.5. The eOn force_threshold is a force criterion, not this value.
* ``rgsaddle:rgsaddle_artn_config_t.basin_push`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:604``; rgconf ``minmode.ArtnConfig.pushStepSizeAngstrom``; eOn ``artn_options_t.push_step_size``. Length of each random-direction push in Angstrom. rgsaddle and rgconf default to 0.2 and eOn to 0.3.
* ``rgsaddle:rgsaddle_artn_config_t.basin_relaxations`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:605``; rgconf ``unmapped``; eOn ``none``. Perpendicular relaxation steps after a basin push. rgconf maxPerpRelaxIters is a single limit with no basin and activation split, and the eOn nperp_limitation string is only a loose relative.
* ``rgsaddle:rgsaddle_artn_config_t.activation_relaxations`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:606``; rgconf ``unmapped``; eOn ``none``. Perpendicular relaxation steps after an activation push. See basin_relaxations for why no field is named.
* ``rgsaddle:rgsaddle_artn_config_t.relaxation_step`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:607``; rgconf ``unmapped``; eOn ``none``. Scale on the projected gradient during relaxation. Neither rgconf nor eOn has this field.
* ``rgsaddle:rgsaddle_artn_config_t.lanczos_actions`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:608``; rgconf ``minmode.ArtnConfig.nlanczosSteps``; eOn ``none``. Most Lanczos Hessian actions per geometry. Both rgsaddle and rgconf default to 16. eOn has no matching option.
* ``rgsaddle:rgsaddle_artn_config_t.curvature_floor`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:609``; rgconf ``unmapped``; eOn ``none``. Lower bound on the absolute eigenvalue in the activation Newton step. Neither rgconf nor eOn has this field.
* ``rgsaddle:rgsaddle_index1_config_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:780``
* ``rgsaddle:rgsaddle_index1_config_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:781``
* ``rgsaddle:rgsaddle_index1_config_t.update`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:782``
* ``rgsaddle:rgsaddle_index1_config_t.mode`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:783``
* ``rgsaddle:rgsaddle_index1_config_t.trust_radius`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:785``
* ``rgsaddle:rgsaddle_index1_config_t.force_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:786``
* ``rgsaddle:rgsaddle_index1_config_t.fd_dr`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:788``
* ``rgsaddle:rgsaddle_kappa_config_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:829``
* ``rgsaddle:rgsaddle_kappa_config_t.beta`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:830``
* ``rgsaddle:rgsaddle_kappa_config_t.tolerance`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:831``
* ``rgsaddle:rgsaddle_kappa_config_t.max_iterations`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:832``
* ``rgsaddle:rgsaddle_kappa_config_t.krylov_dimension`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:833``
* ``rgsaddle:rgsaddle_kappa_config_t.eigen_kind`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:834``
* ``rgsaddle:rgsaddle_kappa_report_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:838``
* ``rgsaddle:rgsaddle_kappa_report_t.kappa`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:839``
* ``rgsaddle:rgsaddle_kappa_report_t.tangent_curvature`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:840``
* ``rgsaddle:rgsaddle_kappa_report_t.gamma_parallel`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:841``
* ``rgsaddle:rgsaddle_kappa_report_t.gamma_perpendicular`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:842``
* ``rgsaddle:rgsaddle_kappa_report_t.residual`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:843``
* ``rgsaddle:rgsaddle_kappa_report_t.hessian_actions`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:844``
* ``rgsaddle:rgsaddle_kappa_report_t.tangent_dimension`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:845``
* ``rgsaddle:rgsaddle_irc_config_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:885``
* ``rgsaddle:rgsaddle_irc_config_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:886``
* ``rgsaddle:rgsaddle_irc_config_t.dx`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:887``
* ``rgsaddle:rgsaddle_irc_config_t.force_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:888``
* ``rgsaddle:rgsaddle_irc_config_t.force_gate`` — ``rgsaddle_force_gate_t``, default ``(no source default)``; ``include/rgsaddle.h:889``
* ``rgsaddle:rgsaddle_irc_config_t.max_move`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:890``
* ``rgsaddle:rgsaddle_irc_config_t.max_inner`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:891``
* ``rgsaddle:rgsaddle_irc_config_t.kind`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:892``
* ``rgsaddle:rgsaddle_irc_config_t.direction`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:893``
* ``rgsaddle:rgsaddle_sella_min_config_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:924``
* ``rgsaddle:rgsaddle_sella_min_config_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:925``
* ``rgsaddle:rgsaddle_sella_min_config_t.delta`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:926``
* ``rgsaddle:rgsaddle_sella_min_config_t.force_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:927``
* ``rgsaddle:rgsaddle_sella_min_config_t.force_gate`` — ``rgsaddle_force_gate_t``, default ``(no source default)``; ``include/rgsaddle.h:928``
* ``rgsaddle:rgsaddle_sella_saddle_config_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:982``
* ``rgsaddle:rgsaddle_sella_saddle_config_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:983``
* ``rgsaddle:rgsaddle_sella_saddle_config_t.delta`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:984``
* ``rgsaddle:rgsaddle_sella_saddle_config_t.force_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:985``
* ``rgsaddle:rgsaddle_sella_saddle_config_t.force_gate`` — ``rgsaddle_force_gate_t``, default ``(no source default)``; ``include/rgsaddle.h:986``
* ``rgsaddle:rgsaddle_sella_saddle_config_t.order`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:987``
* ``rgsaddle:rgsaddle_constraints_config_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:1055``
* ``rgsaddle:rgsaddle_constraints_config_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:1056``
* ``rgsaddle:rgsaddle_samd_config_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:1176``
* ``rgsaddle:rgsaddle_samd_config_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:1177``
* ``rgsaddle:rgsaddle_samd_config_t.dt`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1178``
* ``rgsaddle:rgsaddle_samd_config_t.tau`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1179``
* ``rgsaddle:rgsaddle_samd_config_t.t0`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1180``
* ``rgsaddle:rgsaddle_samd_config_t.tf`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1181``
* ``rgsaddle:rgsaddle_samd_config_t.ngen`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:1182``
* ``rgsaddle:rgsaddle_samd_config_t.exponential`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1183``
* ``rgsaddle:rgsaddle_ocineb_config_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:1248``
* ``rgsaddle:rgsaddle_ocineb_config_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:1249``
* ``rgsaddle:rgsaddle_ocineb_config_t.trigger_factor`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1250``
* ``rgsaddle:rgsaddle_ocineb_config_t.trigger_force`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1251``
* ``rgsaddle:rgsaddle_ocineb_config_t.angle_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1252``
* ``rgsaddle:rgsaddle_ocineb_config_t.stability_count`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:1253``
* ``rgsaddle:rgsaddle_ocineb_config_t.max_mmf_steps`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:1254``
* ``rgsaddle:rgsaddle_ocineb_config_t.restore_unhelpful`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1255``
* ``rgsaddle:rgsaddle_ocineb_config_t.reserved`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1256``
* ``rgsaddle:rgsaddle_ocineb_report_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:1261``
* ``rgsaddle:rgsaddle_ocineb_report_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:1262``
* ``rgsaddle:rgsaddle_ocineb_report_t.status`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1263``
* ``rgsaddle:rgsaddle_ocineb_report_t.phase`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1264``
* ``rgsaddle:rgsaddle_ocineb_report_t.fell_back`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1265``
* ``rgsaddle:rgsaddle_ocineb_report_t.evaluations`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1266``
* ``rgsaddle:rgsaddle_ocineb_report_t.max_force`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1267``
* ``rgsaddle:rgsaddle_ocineb_report_t.curvature`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1268``
* ``rgsaddle:rgsaddle_ocineb_report_t.alignment`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1269``
* ``rgsaddle:rgsaddle_ocineb_report_t.threshold`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1270``
* ``rgsaddle:rgsaddle_ocineb_report_t.ci_index`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:1271``
* ``rgsaddle:rgsaddle_ocineb_report_t.iteration`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:1272``
* ``rgsaddle:rgsaddle_ocineb_report_t.rotations`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:1273``
* ``rgsaddle:rgsaddle_stencil_report_t.struct_size`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:1508``
* ``rgsaddle:rgsaddle_stencil_report_t.status`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1509``
* ``rgsaddle:rgsaddle_stencil_report_t.reserved`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1510``
* ``rgsaddle:rgsaddle_stencil_report_t.max_force`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1511``
* ``rgsaddle:rgsaddle_stencil_report_t.curvature`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1512``
* ``rgsaddle:rgsaddle_stencil_report_t.iteration`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:1513``
* ``rgsaddle:rgsaddle_stencil_report_t.evaluations`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:1514``
* ``rgsaddle:rgsaddle_connect_config_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:1552``
* ``rgsaddle:rgsaddle_connect_config_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:1553``
* ``rgsaddle:rgsaddle_connect_config_t.force_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1554``
* ``rgsaddle:rgsaddle_connect_config_t.max_calls`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:1555``
* ``rgsaddle:rgsaddle_connect_config_t.quench_nodes`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1556``
* ``rgsaddle:rgsaddle_connect_report_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:1561``
* ``rgsaddle:rgsaddle_connect_report_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:1562``
* ``rgsaddle:rgsaddle_connect_report_t.energy`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1563``
* ``rgsaddle:rgsaddle_connect_report_t.force_norm`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1564``
* ``rgsaddle:rgsaddle_connect_report_t.kind`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1565``
* ``rgsaddle:rgsaddle_connect_report_t.hessian_index`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1566``
* ``rgsaddle:rgsaddle_connect_report_t.evaluations`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1567``
* ``rgsaddle:rgsaddle_indexk_config_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:1586``
* ``rgsaddle:rgsaddle_indexk_config_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:1587``
* ``rgsaddle:rgsaddle_indexk_config_t.index`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1588``
* ``rgsaddle:rgsaddle_indexk_config_t.method`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1589``
* ``rgsaddle:rgsaddle_indexk_config_t.dr`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1590``
* ``rgsaddle:rgsaddle_indexk_config_t.rotation_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1591``
* ``rgsaddle:rgsaddle_indexk_config_t.max_rotations`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:1592``
* ``rgsaddle:rgsaddle_indexk_config_t.force_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1593``
* ``rgsaddle:rgsaddle_indexk_config_t.max_move`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1594``
* ``rgsaddle:rgsaddle_indexk_config_t.residual_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1595``
* ``rgsaddle:rgsaddle_indexk_config_t.max_block_iterations`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1596``
* ``rgsaddle:rgsaddle_saddle_check_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:1680``
* ``rgsaddle:rgsaddle_saddle_check_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:1681``
* ``rgsaddle:rgsaddle_saddle_check_t.block`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1682``
* ``rgsaddle:rgsaddle_saddle_check_t.lowest`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1683``
* ``rgsaddle:rgsaddle_saddle_check_t.max_actions`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1684``
* ``rgsaddle:rgsaddle_saddle_check_t.difference`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1685``
* ``rgsaddle:rgsaddle_saddle_check_t.dr`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1686``
* ``rgsaddle:rgsaddle_saddle_check_t.residual_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1687``
* ``rgsaddle:rgsaddle_saddle_check_t.negative_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1688``
* ``rgsaddle:rgsaddle_saddle_check_t.seed`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:1689``
* ``rgsaddle:rgsaddle_saddle_check_t.escape_attempts`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1690``
* ``rgsaddle:rgsaddle_saddle_check_t.reserved`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1691``
* ``rgsaddle:rgsaddle_saddle_check_t.escape_push`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1692``
* ``rgsaddle:rgsaddle_saddle_index_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:1697``
* ``rgsaddle:rgsaddle_saddle_index_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:1698``
* ``rgsaddle:rgsaddle_saddle_index_t.certified`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1701``
* ``rgsaddle:rgsaddle_saddle_index_t.index`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1703``
* ``rgsaddle:rgsaddle_saddle_index_t.converged`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1706``
* ``rgsaddle:rgsaddle_saddle_index_t.n_curvatures`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1709``
* ``rgsaddle:rgsaddle_saddle_index_t.evaluations`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:1711``
* ``rgsaddle:rgsaddle_saddle_index_t.total_evaluations`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:1714``
* ``rgsaddle:rgsaddle_saddle_index_t.escapes`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:1716``
* ``rgsaddle:rgsaddle_growing_string_config_t.version`` — ``rgsaddle_version_t``, default ``(no source default)``; ``include/rgsaddle.h:1802``; rgconf ``abi``; eOn ``none``. ABI record header, not a run parameter.
* ``rgsaddle:rgsaddle_growing_string_config_t.flags`` — ``uint64_t``, default ``(no source default)``; ``include/rgsaddle.h:1803``; rgconf ``abi``; eOn ``none``. Reserved ABI word.
* ``rgsaddle:rgsaddle_growing_string_config_t.band`` — ``rgsaddle_band_config_t``, default ``(no source default)``; ``include/rgsaddle.h:1804``; rgconf ``band.BandConfig``; eOn ``none``. Nested band configuration. rgconf carries the same group as BandConfig, and eOn spreads it over several fields.
* ``rgsaddle:rgsaddle_growing_string_config_t.n_images`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:1805``; rgconf ``band.BandConfig.imageCount``; eOn ``neb_options_t.image_count``. Total image count including both endpoints in rgsaddle and rgconf. eOn counts intermediate images only.
* ``rgsaddle:rgsaddle_growing_string_config_t.frontier_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1806``; rgconf ``band.BandConfig.growingStringFrontierTol``; eOn ``none``. Perpendicular-force threshold for adding a node. The rgsaddle default is 0.2 and the schema default is 0.1. The eOn sidpp_frontier_tol belongs to the IDPP path growth, which is a different method.
* ``rgsaddle:rgsaddle_growing_string_config_t.min_steps_per_node`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:1807``; rgconf ``unmapped``; eOn ``none``. Least relaxation steps at the current fragment size before a frontier may add a node. No rgconf or eOn field.
* ``rgsaddle:rgsaddle_growing_string_config_t.max_steps_per_node`` — ``int64_t``, default ``(no source default)``; ``include/rgsaddle.h:1808``; rgconf ``unmapped``; eOn ``none``. Cap on relaxation steps at the current fragment size, where zero selects the default cap. No rgconf or eOn field.
* ``rgsaddle:rgsaddle_growing_string_config_t.join_tol`` — ``double``, default ``(no source default)``; ``include/rgsaddle.h:1809``; rgconf ``unmapped``; eOn ``none``. Frontier distance at which the two fragments join early, where -1 disables the early join. No rgconf or eOn field.
* ``rgsaddle:rgsaddle_growing_string_config_t.reparametrization`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1810``; rgconf ``unmapped``; eOn ``none``. Spline kind used to redistribute nodes. The eOn sidpp_reparam is a boolean for IDPP growth and does not choose a spline.
* ``rgsaddle:rgsaddle_growing_string_config_t.force_gate`` — ``int32_t``, default ``(no source default)``; ``include/rgsaddle.h:1811``; rgconf ``convergence.ConvergenceConfig.norm``; eOn ``optimizer_options_t.convergence_metric``. Norm used for the frontier test and the reported force. rgsaddle offers L2, Linf and max-atom, defaulting to Linf, while the rgconf ForceNorm also has the rms forms.
* ``rgpot:LJConfig.u0`` — ``double``, default ``1.0``; ``CppCore/rgpot/LennardJones/LJPot.hpp:37``
* ``rgpot:LJConfig.cutoff`` — ``double``, default ``15.0``; ``CppCore/rgpot/LennardJones/LJPot.hpp:38``
* ``rgpot:LJConfig.psi`` — ``double``, default ``1.0``; ``CppCore/rgpot/LennardJones/LJPot.hpp:39``
* ``rgpot:LJConfig.switch_width`` — ``double``, default ``0.0``; ``CppCore/rgpot/LennardJones/LJPot.hpp:43``
* ``rgpot:MetatomicConfig.model_path`` — ``std::string``, default ``(no source default)``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:13``
* ``rgpot:MetatomicConfig.device`` — ``std::string``, default ``(no source default)``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:14``
* ``rgpot:MetatomicConfig.length_unit`` — ``std::string``, default ``angstrom``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:15``
* ``rgpot:MetatomicConfig.extensions_directory`` — ``std::string``, default ``(no source default)``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:16``
* ``rgpot:MetatomicConfig.check_consistency`` — ``bool``, default ``false``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:17``
* ``rgpot:MetatomicConfig.uncertainty_threshold`` — ``double``, default ``-1.0``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:18``
* ``rgpot:MetatomicConfig.dtype_override`` — ``std::string``, default ``(no source default)``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:19``
* ``rgpot:MetatomicConfig.random_rotation`` — ``bool``, default ``false``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:20``
* ``rgpot:MetatomicConfig.n_symmetry_rotations`` — ``long``, default ``0``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:21``
* ``rgpot:MetatomicConfig.so3_probe_scatter`` — ``bool``, default ``false``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:22``
* ``rgpot:MetatomicConfig.torch_determinism`` — ``TorchDeterminismPolicy``, default ``TorchDeterminismPolicy::Fast``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:23``
* ``rgpot:MetatomicConfig.engine_path`` — ``std::string``, default ``(no source default)``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:25``
* ``rgpot:MetatomicConfig.task_name`` — ``std::string``, default ``(no source default)``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:27``
* ``rgpot:MetatomicConfig.charge`` — ``int``, default ``0``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:29``
* ``rgpot:MetatomicConfig.spin`` — ``int``, default ``1``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:32``
* ``rgpot:MetatomicConfig.requested_inputs`` — ``*``, default ``(no source default)``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:34``
* ``rgpot:MetatomicConfig.attach_system_extras`` — ``bool``, default ``false``; ``CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp:36``
* ``cpmdc:CPMDCResult.ok`` — ``int``, default ``(no source default)``; ``include/cpmdc.h:42``
* ``cpmdc:CPMDCResult.energy_h`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:44``
* ``cpmdc:CPMDCEnergyComponents.valid`` — ``int``, default ``(no source default)``; ``include/cpmdc.h:59``
* ``cpmdc:CPMDCEnergyComponents.etot`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:61``
* ``cpmdc:CPMDCEnergyComponents.ekin`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:63``
* ``cpmdc:CPMDCEnergyComponents.epseu`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:65``
* ``cpmdc:CPMDCEnergyComponents.enl`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:67``
* ``cpmdc:CPMDCEnergyComponents.eht`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:69``
* ``cpmdc:CPMDCEnergyComponents.ehep`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:71``
* ``cpmdc:CPMDCEnergyComponents.ehee`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:73``
* ``cpmdc:CPMDCEnergyComponents.ehii`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:75``
* ``cpmdc:CPMDCEnergyComponents.exc`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:77``
* ``cpmdc:CPMDCEnergyComponents.vxc`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:79``
* ``cpmdc:CPMDCEnergyComponents.egc`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:81``
* ``cpmdc:CPMDCEnergyComponents.esr`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:83``
* ``cpmdc:CPMDCEnergyComponents.eeig`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:85``
* ``cpmdc:CPMDCEnergyComponents.eband`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:87``
* ``cpmdc:CPMDCEnergyComponents.entropy`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:89``
* ``cpmdc:CPMDCEnergyComponents.eself`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:91``
* ``cpmdc:CPMDCEnergyComponents.ecnstr`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:93``
* ``cpmdc:CPMDCEnergyComponents.amu`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:95``
* ``cpmdc:CPMDCEnergyComponents.ebogo`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:97``
* ``cpmdc:CPMDCEnergyComponents.eext`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:99``
* ``cpmdc:CPMDCEnergyComponents.etddft`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:101``
* ``cpmdc:CPMDCEnergyComponents.ehsic`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:103``
* ``cpmdc:CPMDCEnergyComponents.erestr`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:105``
* ``cpmdc:CPMDCEnergyComponents.eefield`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:107``
* ``cpmdc:CPMDCChargeIntegrals.valid`` — ``int``, default ``(no source default)``; ``include/cpmdc.h:115``
* ``cpmdc:CPMDCChargeIntegrals.csumg`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:117``
* ``cpmdc:CPMDCChargeIntegrals.csumr`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:119``
* ``cpmdc:CPMDCChargeIntegrals.csums`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:121``
* ``cpmdc:CPMDCChargeIntegrals.csumsabs`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:123``
* ``cpmdc:CPMDCMultiStateEnergies.valid`` — ``int``, default ``(no source default)``; ``include/cpmdc.h:134``
* ``cpmdc:CPMDCMultiStateEnergies.count`` — ``size_t``, default ``(no source default)``; ``include/cpmdc.h:136``
* ``cpmdc:CPMDCMDTrajectoryRow.valid`` — ``int``, default ``(no source default)``; ``include/cpmdc.h:151``
* ``cpmdc:CPMDCMDTrajectoryRow.count`` — ``size_t``, default ``(no source default)``; ``include/cpmdc.h:153``
* ``cpmdc:CPMDCPropertySnapshot.valid`` — ``int``, default ``(no source default)``; ``include/cpmdc.h:169``
* ``cpmdc:CPMDCPropertySnapshot.hessian_count`` — ``size_t``, default ``(no source default)``; ``include/cpmdc.h:171``
* ``cpmdc:CPMDCPropertySnapshot.dipole_count`` — ``size_t``, default ``(no source default)``; ``include/cpmdc.h:175``
* ``cpmdc:CPMDCPropertySnapshot.polarizability_count`` — ``size_t``, default ``(no source default)``; ``include/cpmdc.h:179``
* ``cpmdc:CPMDCStressTensor.valid`` — ``int``, default ``(no source default)``; ``include/cpmdc.h:195``
* ``cpmdc:CPMDCRankSpread.valid`` — ``int``, default ``(no source default)``; ``include/cpmdc.h:210``
* ``cpmdc:CPMDCRankSpread.ranks`` — ``int``, default ``(no source default)``; ``include/cpmdc.h:212``
* ``cpmdc:CPMDCRankSpread.energy_h`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:214``
* ``cpmdc:CPMDCRankSpread.gradient_h_bohr`` — ``double``, default ``(no source default)``; ``include/cpmdc.h:216``
* ``cpmdc:ForceInput.pos`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:19``
* ``cpmdc:ForceInput.atmnrs`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:20``
* ``cpmdc:ForceInput.box`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:21``
* ``cpmdc:ForceInput.lengthUnit`` — ``Text``, default ``angstrom``; ``schema/Potentials.capnp:22``
* ``cpmdc:ForceInput.energyUnit`` — ``Text``, default ``eV``; ``schema/Potentials.capnp:23``
* ``cpmdc:ForceInput.hasCharge`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:24``
* ``cpmdc:ForceInput.charge`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:25``
* ``cpmdc:ForceInput.hasMultiplicity`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:26``
* ``cpmdc:ForceInput.multiplicity`` — ``Int32``, default ``1``; ``schema/Potentials.capnp:27``
* ``cpmdc:PotentialResult.energy`` — ``Float64``, default ``(no source default)``; ``schema/Potentials.capnp:33``
* ``cpmdc:PotentialResult.forces`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:34``
* ``cpmdc:PotentialResult.hessian`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:35``
* ``cpmdc:PotentialResult.dipole`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:36``
* ``cpmdc:PotentialResult.quadrupole`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:37``
* ``cpmdc:PotentialResult.optimizedPos`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:38``
* ``cpmdc:PotentialResult.frequencies`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:39``
* ``cpmdc:PotentialResult.intensities`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:40``
* ``cpmdc:PotentialResult.stress`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:41``
* ``cpmdc:PotentialResult.polarizability`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:42``
* ``cpmdc:PotentialResult.gradient`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:43``
* ``cpmdc:PotentialResult.normalModes`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:44``
* ``cpmdc:PotentialResult.zeroPointEnergy`` — ``Float64``, default ``(no source default)``; ``schema/Potentials.capnp:45``
* ``cpmdc:PotentialResult.thermalEnergy`` — ``Float64``, default ``(no source default)``; ``schema/Potentials.capnp:46``
* ``cpmdc:PotentialResult.thermalEnthalpy`` — ``Float64``, default ``(no source default)``; ``schema/Potentials.capnp:47``
* ``cpmdc:PotentialResult.entropy`` — ``Float64``, default ``(no source default)``; ``schema/Potentials.capnp:48``
* ``cpmdc:PotentialResult.heatCapacityCv`` — ``Float64``, default ``(no source default)``; ``schema/Potentials.capnp:49``
* ``cpmdc:PotentialResult.projectedFrequencies`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:50``
* ``cpmdc:PotentialResult.projectedIntensities`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:51``
* ``cpmdc:PotentialResult.energyComponents`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:53``
* ``cpmdc:PotentialResult.componentsValid`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:54``
* ``cpmdc:PotentialResult.chargeIntegrals`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:55``
* ``cpmdc:PotentialResult.chargeValid`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:56``
* ``cpmdc:PotentialResult.multiStateEnergies`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:57``
* ``cpmdc:PotentialResult.multiStateValid`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:58``
* ``cpmdc:PotentialResult.mdTrajectoryRow`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:59``
* ``cpmdc:PotentialResult.mdTrajectoryValid`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:60``
* ``cpmdc:PotentialResult.embedMdPropsSkipped`` — ``Bool``, default ``true``; ``schema/Potentials.capnp:61``
* ``cpmdc:Capabilities.backendName`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:73``
* ``cpmdc:Capabilities.backendVersion`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:74``
* ``cpmdc:Capabilities.abiVersion`` — ``Int32``, default ``(no source default)``; ``schema/Potentials.capnp:75``
* ``cpmdc:Capabilities.available`` — ``Bool``, default ``(no source default)``; ``schema/Potentials.capnp:76``
* ``cpmdc:Capabilities.operations`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:77``
* ``cpmdc:Capabilities.loweredCommonFields`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:78``
* ``cpmdc:Capabilities.configKinds`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:79``
* ``cpmdc:Capabilities.schemaVersion`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:80``
* ``cpmdc:Capabilities.protocolFamily`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:81``
* ``cpmdc:Capabilities.protocolMajor`` — ``UInt16``, default ``(no source default)``; ``schema/Potentials.capnp:82``
* ``cpmdc:Capabilities.protocolMinor`` — ``UInt16``, default ``(no source default)``; ``schema/Potentials.capnp:83``
* ``cpmdc:Capabilities.schemaId`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:84``
* ``cpmdc:Capabilities.bridgeAbiMajor`` — ``UInt16``, default ``(no source default)``; ``schema/Potentials.capnp:85``
* ``cpmdc:Capabilities.bridgeAbiMinor`` — ``UInt16``, default ``(no source default)``; ``schema/Potentials.capnp:86``
* ``cpmdc:Capabilities.bridgeLayout`` — ``UInt32``, default ``(no source default)``; ``schema/Potentials.capnp:87``
* ``cpmdc:Capabilities.dlpackMajor`` — ``UInt16``, default ``(no source default)``; ``schema/Potentials.capnp:88``
* ``cpmdc:Capabilities.dlpackMinor`` — ``UInt16``, default ``(no source default)``; ``schema/Potentials.capnp:89``
* ``cpmdc:Capabilities.bridgeFeatures`` — ``UInt64``, default ``(no source default)``; ``schema/Potentials.capnp:90``
* ``cpmdc:NWChemDirective.keyword`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:112``
* ``cpmdc:NWChemDirective.args`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:113``
* ``cpmdc:NWChemGenericStanza.name`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:117``
* ``cpmdc:NWChemGenericStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:118``
* ``cpmdc:NWChemSetDirective.key`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:122``
* ``cpmdc:NWChemSetDirective.value`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:123``
* ``cpmdc:NWChemSetDirective.valueType`` — ``ValueType``, default ``auto``; ``schema/Potentials.capnp:124``
* ``cpmdc:NWChemSetDirective.values`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:125``
* ``cpmdc:NWChemDftSmearing.sigmaHartree`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:137``
* ``cpmdc:NWChemDftSmearing.mode`` — ``Mode``, default ``fixsz``; ``schema/Potentials.capnp:138``
* ``cpmdc:NWChemDftGridSpec.quality`` — ``Quality``, default ``unspecified``; ``schema/Potentials.capnp:149``
* ``cpmdc:NWChemDftGridSpec.radial`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:150``
* ``cpmdc:NWChemDftGridSpec.angular`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:151``
* ``cpmdc:NWChemDftStanza.xc`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:165``
* ``cpmdc:NWChemDftStanza.direct`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:166``
* ``cpmdc:NWChemDftStanza.smearing`` — ``NWChemDftSmearing``, default ``(no source default)``; ``schema/Potentials.capnp:167``
* ``cpmdc:NWChemDftStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:169``
* ``cpmdc:NWChemDftStanza.iterations`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:170``
* ``cpmdc:NWChemDftStanza.grid`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:171``
* ``cpmdc:NWChemDftStanza.energyConv`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:173``
* ``cpmdc:NWChemDftStanza.densityConv`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:174``
* ``cpmdc:NWChemDftStanza.gradientConv`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:175``
* ``cpmdc:NWChemDftStanza.odft`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:177``
* ``cpmdc:NWChemDftStanza.diis`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:178``
* ``cpmdc:NWChemDftStanza.nfock`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:179``
* ``cpmdc:NWChemDftStanza.levelShift`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:180``
* ``cpmdc:NWChemDftStanza.vectorsInput`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:181``
* ``cpmdc:NWChemDftStanza.vectorsOutput`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:182``
* ``cpmdc:NWChemDftStanza.gridSpec`` — ``NWChemDftGridSpec``, default ``(no source default)``; ``schema/Potentials.capnp:183``
* ``cpmdc:NWChemDftStanza.disp`` — ``NWChemDftDisp``, default ``(no source default)``; ``schema/Potentials.capnp:184``
* ``cpmdc:NWChemDftStanza.cdft`` — ``NWChemCdftSpec``, default ``(no source default)``; ``schema/Potentials.capnp:185``
* ``cpmdc:NWChemDftDisp.enabled`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:191``
* ``cpmdc:NWChemDftDisp.vdw`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:192``
* ``cpmdc:NWChemDftDisp.s6`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:193``
* ``cpmdc:NWChemDftDisp.s8`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:194``
* ``cpmdc:NWChemDftDisp.sr6`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:195``
* ``cpmdc:NWChemDftDisp.alpha`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:196``
* ``cpmdc:NWChemRelativisticStanza.method`` — ``Method``, default ``none``; ``schema/Potentials.capnp:202``
* ``cpmdc:NWChemRelativisticStanza.spinOrbit`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:203``
* ``cpmdc:NWChemRelativisticStanza.atomicScf`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:204``
* ``cpmdc:NWChemRelativisticStanza.cutoff`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:205``
* ``cpmdc:NWChemRelativisticStanza.dkOrder`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:206``
* ``cpmdc:NWChemRelativisticStanza.dkModifier`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:207``
* ``cpmdc:NWChemRelativisticStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:208``
* ``cpmdc:NWChemCosmoStanza.dielec`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:220``
* ``cpmdc:NWChemCosmoStanza.solvent`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:221``
* ``cpmdc:NWChemCosmoStanza.rsolv`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:222``
* ``cpmdc:NWChemCosmoStanza.radiusScheme`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:223``
* ``cpmdc:NWChemCosmoStanza.radii`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:224``
* ``cpmdc:NWChemCosmoStanza.iscren`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:225``
* ``cpmdc:NWChemCosmoStanza.minbem`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:226``
* ``cpmdc:NWChemCosmoStanza.lineq`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:227``
* ``cpmdc:NWChemCosmoStanza.doCosmoSmd`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:228``
* ``cpmdc:NWChemCosmoStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:229``
* ``cpmdc:NWChemSmdStanza.solvent`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:235``
* ``cpmdc:NWChemSmdStanza.sola`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:236``
* ``cpmdc:NWChemSmdStanza.solb`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:237``
* ``cpmdc:NWChemSmdStanza.soln`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:238``
* ``cpmdc:NWChemSmdStanza.solg`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:239``
* ``cpmdc:NWChemSmdStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:240``
* ``cpmdc:NWChemConstraint.kind`` — ``Kind``, default ``(no source default)``; ``schema/Potentials.capnp:246``
* ``cpmdc:NWChemConstraint.atoms`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:247``
* ``cpmdc:NWChemConstraint.value`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:248``
* ``cpmdc:NWChemConstraintsStanza.clear`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:259``
* ``cpmdc:NWChemConstraintsStanza.constraints`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:260``
* ``cpmdc:NWChemConstraintsStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:261``
* ``cpmdc:NWChemIsotope.atom`` — ``UInt32``, default ``(no source default)``; ``schema/Potentials.capnp:267``
* ``cpmdc:NWChemIsotope.mass`` — ``Float64``, default ``(no source default)``; ``schema/Potentials.capnp:268``
* ``cpmdc:NWChemVibStanza.temperatures`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:272``
* ``cpmdc:NWChemVibStanza.pressure`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:273``
* ``cpmdc:NWChemVibStanza.reuse`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:274``
* ``cpmdc:NWChemVibStanza.animate`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:275``
* ``cpmdc:NWChemVibStanza.masses`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:276``
* ``cpmdc:NWChemVibStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:277``
* ``cpmdc:NWChemBqCharge.x`` — ``Float64``, default ``(no source default)``; ``schema/Potentials.capnp:283``
* ``cpmdc:NWChemBqCharge.y`` — ``Float64``, default ``(no source default)``; ``schema/Potentials.capnp:284``
* ``cpmdc:NWChemBqCharge.z`` — ``Float64``, default ``(no source default)``; ``schema/Potentials.capnp:285``
* ``cpmdc:NWChemBqCharge.q`` — ``Float64``, default ``(no source default)``; ``schema/Potentials.capnp:286``
* ``cpmdc:NWChemBqStanza.units`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:290``
* ``cpmdc:NWChemBqStanza.charges`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:291``
* ``cpmdc:NWChemBqStanza.loadFile`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:292``
* ``cpmdc:NWChemBqStanza.forces`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:293``
* ``cpmdc:NWChemBqStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:294``
* ``cpmdc:NWChemModuleStanza.name`` — ``NWChemModuleName``, default ``custom``; ``schema/Potentials.capnp:389``
* ``cpmdc:NWChemModuleStanza.customName`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:390``
* ``cpmdc:NWChemModuleStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:391``
* ``cpmdc:NWChemPseudopotentialEntry.element`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:395``
* ``cpmdc:NWChemPseudopotentialEntry.libraryType`` — ``LibraryType``, default ``library``; ``schema/Potentials.capnp:396``
* ``cpmdc:NWChemPseudopotentialEntry.libraryName`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:397``
* ``cpmdc:NWChemPseudopotentialEntry.allElements`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:398``
* ``cpmdc:NWChemPseudopotentialSpinRule.channel`` — ``Channel``, default ``up``; ``schema/Potentials.capnp:416``
* ``cpmdc:NWChemPseudopotentialSpinRule.angularMomentum`` — ``AngularMomentum``, default ``s``; ``schema/Potentials.capnp:417``
* ``cpmdc:NWChemPseudopotentialSpinRule.scale`` — ``Float64``, default ``1.0``; ``schema/Potentials.capnp:418``
* ``cpmdc:NWChemPseudopotentialSpinRule.ionIndices`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:419``
* ``cpmdc:NWChemPseudopotentialSpinRule.hasMagneticQuantumNumber`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:420``
* ``cpmdc:NWChemPseudopotentialSpinRule.magneticQuantumNumber`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:421``
* ``cpmdc:NWChemPseudopotentialUtermRule.angularMomentum`` — ``AngularMomentum``, default ``s``; ``schema/Potentials.capnp:437``
* ``cpmdc:NWChemPseudopotentialUtermRule.uScale`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:438``
* ``cpmdc:NWChemPseudopotentialUtermRule.jScale`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:439``
* ``cpmdc:NWChemPseudopotentialUtermRule.ionIndices`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:440``
* ``cpmdc:NWChemPseudopotentialStanza.entries`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:456``
* ``cpmdc:NWChemPseudopotentialStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:457``
* ``cpmdc:NWChemPseudopotentialStanza.pspSpin`` — ``NWChemPseudopotentialSpinMode``, default ``unspecified``; ``schema/Potentials.capnp:458``
* ``cpmdc:NWChemPseudopotentialStanza.spinRules`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:459``
* ``cpmdc:NWChemPseudopotentialStanza.semicoreSmall`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:460``
* ``cpmdc:NWChemPseudopotentialStanza.uterm`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:461``
* ``cpmdc:NWChemPseudopotentialStanza.utermRules`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:462``
* ``cpmdc:NWChemPseudopotentialStanza.blockName`` — ``NWChemPseudopotentialBlockName``, default ``pseudopotentials``; ``schema/Potentials.capnp:463``
* ``cpmdc:NWChemKVector.x`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:467``
* ``cpmdc:NWChemKVector.y`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:468``
* ``cpmdc:NWChemKVector.z`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:469``
* ``cpmdc:NWChemKVector.weight`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:470``
* ``cpmdc:NWChemBrillouinZoneStanza.zoneName`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:474``
* ``cpmdc:NWChemBrillouinZoneStanza.kVectors`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:475``
* ``cpmdc:NWChemBrillouinZoneStanza.maxKpointsPrint`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:476``
* ``cpmdc:NWChemBrillouinZoneStanza.monkhorstPackX`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:477``
* ``cpmdc:NWChemBrillouinZoneStanza.monkhorstPackY`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:478``
* ``cpmdc:NWChemBrillouinZoneStanza.monkhorstPackZ`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:479``
* ``cpmdc:NWChemBrillouinZoneStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:480``
* ``cpmdc:NWChemBrillouinZoneStanza.zoneStructureName`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:481``
* ``cpmdc:NWChemBrillouinZoneStanza.zoneFftName`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:482``
* ``cpmdc:NWChemBrillouinZoneStanza.tetrahedronGridX`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:483``
* ``cpmdc:NWChemBrillouinZoneStanza.tetrahedronGridY`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:484``
* ``cpmdc:NWChemBrillouinZoneStanza.tetrahedronGridZ`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:485``
* ``cpmdc:NWChemBrillouinZoneStanza.tetrahedronZoneName`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:486``
* ``cpmdc:NWChemBrillouinZoneStanza.dosGridX`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:487``
* ``cpmdc:NWChemBrillouinZoneStanza.dosGridY`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:488``
* ``cpmdc:NWChemBrillouinZoneStanza.dosGridZ`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:489``
* ``cpmdc:NWChemBrillouinZoneStanza.dosGridZoneName`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:490``
* ``cpmdc:NWChemBrillouinZoneStanza.dosFftGridX`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:491``
* ``cpmdc:NWChemBrillouinZoneStanza.dosFftGridY`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:492``
* ``cpmdc:NWChemBrillouinZoneStanza.dosFftGridZ`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:493``
* ``cpmdc:NWChemBrillouinZoneStanza.dosFftGridZoneName`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:494``
* ``cpmdc:NWChemSimulationCellStanza.cellName`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:505``
* ``cpmdc:NWChemSimulationCellStanza.boundaryConditions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:506``
* ``cpmdc:NWChemSimulationCellStanza.latticeVectorsBohr`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:507``
* ``cpmdc:NWChemSimulationCellStanza.ngridX`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:508``
* ``cpmdc:NWChemSimulationCellStanza.ngridY`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:509``
* ``cpmdc:NWChemSimulationCellStanza.ngridZ`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:510``
* ``cpmdc:NWChemSimulationCellStanza.ngridSmallX`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:511``
* ``cpmdc:NWChemSimulationCellStanza.ngridSmallY`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:512``
* ``cpmdc:NWChemSimulationCellStanza.ngridSmallZ`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:513``
* ``cpmdc:NWChemSimulationCellStanza.boxDeltaBohr`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:514``
* ``cpmdc:NWChemSimulationCellStanza.boxOrient`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:515``
* ``cpmdc:NWChemSimulationCellStanza.boxDifferentLengths`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:516``
* ``cpmdc:NWChemSimulationCellStanza.latticeKind`` — ``NWChemSimulationCellLatticeKind``, default ``unspecified``; ``schema/Potentials.capnp:517``
* ``cpmdc:NWChemSimulationCellStanza.latticeLengthBohr`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:518``
* ``cpmdc:NWChemSimulationCellStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:519``
* ``cpmdc:NWChemNwpwStanza.energyCutoff`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:671``
* ``cpmdc:NWChemNwpwStanza.wavefunctionCutoff`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:672``
* ``cpmdc:NWChemNwpwStanza.ewaldRcut`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:673``
* ``cpmdc:NWChemNwpwStanza.ewaldNcut`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:674``
* ``cpmdc:NWChemNwpwStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:675``
* ``cpmdc:NWChemNwpwStanza.cellName`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:676``
* ``cpmdc:NWChemNwpwStanza.inputWavefunctionFilename`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:677``
* ``cpmdc:NWChemNwpwStanza.outputWavefunctionFilename`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:678``
* ``cpmdc:NWChemNwpwStanza.fakeMass`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:679``
* ``cpmdc:NWChemNwpwStanza.timeStep`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:680``
* ``cpmdc:NWChemNwpwStanza.loopStart`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:681``
* ``cpmdc:NWChemNwpwStanza.loopEnd`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:682``
* ``cpmdc:NWChemNwpwStanza.toleranceEnergy`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:683``
* ``cpmdc:NWChemNwpwStanza.toleranceDensity`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:684``
* ``cpmdc:NWChemNwpwStanza.toleranceGradient`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:685``
* ``cpmdc:NWChemNwpwStanza.exchangeCorrelation`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:686``
* ``cpmdc:NWChemNwpwStanza.balanceMode`` — ``NWChemNwpwBalanceMode``, default ``unspecified``; ``schema/Potentials.capnp:687``
* ``cpmdc:NWChemNwpwStanza.boStepStart`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:688``
* ``cpmdc:NWChemNwpwStanza.boStepEnd`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:689``
* ``cpmdc:NWChemNwpwStanza.boTimeStep`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:690``
* ``cpmdc:NWChemNwpwStanza.boAlgorithm`` — ``NWChemNwpwBoAlgorithm``, default ``unspecified``; ``schema/Potentials.capnp:691``
* ``cpmdc:NWChemNwpwStanza.boFakeMass`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:692``
* ``cpmdc:NWChemNwpwStanza.scalingFirst`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:693``
* ``cpmdc:NWChemNwpwStanza.scalingSecond`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:694``
* ``cpmdc:NWChemNwpwStanza.npFftProcesses`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:695``
* ``cpmdc:NWChemNwpwStanza.npOrbitalProcesses`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:696``
* ``cpmdc:NWChemNwpwStanza.npKspaceProcesses`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:697``
* ``cpmdc:NWChemNwpwStanza.spinOrbit`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:698``
* ``cpmdc:NWChemNwpwStanza.parallelIo`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:699``
* ``cpmdc:NWChemNwpwStanza.xyzFilename`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:700``
* ``cpmdc:NWChemNwpwStanza.ionMotionFilename`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:701``
* ``cpmdc:NWChemNwpwStanza.electronMotionFilename`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:702``
* ``cpmdc:NWChemNwpwStanza.hamiltonianMotionFilename`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:703``
* ``cpmdc:NWChemNwpwStanza.orbitalMotionFilename`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:704``
* ``cpmdc:NWChemNwpwStanza.eigenvalueMotionFilename`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:705``
* ``cpmdc:NWChemNwpwStanza.fractionalOrbitalsStart`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:706``
* ``cpmdc:NWChemNwpwStanza.fractionalOrbitalsEnd`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:707``
* ``cpmdc:NWChemNwpwStanza.smearTemperature`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:708``
* ``cpmdc:NWChemNwpwStanza.smearAlpha`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:709``
* ``cpmdc:NWChemNwpwStanza.smearType`` — ``NWChemNwpwSmearType``, default ``unspecified``; ``schema/Potentials.capnp:710``
* ``cpmdc:NWChemNwpwStanza.virtualOrbitalsStart`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:711``
* ``cpmdc:NWChemNwpwStanza.virtualOrbitalsEnd`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:712``
* ``cpmdc:NWChemNwpwStanza.lcaoMode`` — ``NWChemNwpwLcaoMode``, default ``unspecified``; ``schema/Potentials.capnp:713``
* ``cpmdc:NWChemNwpwStanza.ewaldGridX`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:714``
* ``cpmdc:NWChemNwpwStanza.ewaldGridY`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:715``
* ``cpmdc:NWChemNwpwStanza.ewaldGridZ`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:716``
* ``cpmdc:NWChemNwpwStanza.noseHoover`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:717``
* ``cpmdc:NWChemNwpwStanza.noseRestart`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:718``
* ``cpmdc:NWChemNwpwStanza.noseElectronPeriod`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:719``
* ``cpmdc:NWChemNwpwStanza.noseElectronTemperature`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:720``
* ``cpmdc:NWChemNwpwStanza.noseIonPeriod`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:721``
* ``cpmdc:NWChemNwpwStanza.noseIonTemperature`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:722``
* ``cpmdc:NWChemNwpwStanza.noseElectronChainLength`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:723``
* ``cpmdc:NWChemNwpwStanza.noseIonChainLength`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:724``
* ``cpmdc:NWChemNwpwStanza.atomEfield`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:725``
* ``cpmdc:NWChemNwpwStanza.atomEfieldGradient`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:726``
* ``cpmdc:NWChemNwpwStanza.mulliken`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:727``
* ``cpmdc:NWChemNwpwStanza.mullikenKawai`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:728``
* ``cpmdc:NWChemNwpwStanza.periodicDipole`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:729``
* ``cpmdc:NWChemNwpwStanza.electricField`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:730``
* ``cpmdc:NWChemNwpwStanza.electricFieldX`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:731``
* ``cpmdc:NWChemNwpwStanza.electricFieldY`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:732``
* ``cpmdc:NWChemNwpwStanza.electricFieldZ`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:733``
* ``cpmdc:NWChemNwpwStanza.electricFieldCenterSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:734``
* ``cpmdc:NWChemNwpwStanza.electricFieldCenterX`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:735``
* ``cpmdc:NWChemNwpwStanza.electricFieldCenterY`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:736``
* ``cpmdc:NWChemNwpwStanza.electricFieldCenterZ`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:737``
* ``cpmdc:NWChemNwpwStanza.electricFieldType`` — ``NWChemNwpwEfieldType``, default ``unspecified``; ``schema/Potentials.capnp:738``
* ``cpmdc:NWChemNwpwStanza.smoothCutoff`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:739``
* ``cpmdc:NWChemNwpwStanza.smoothCutoffAfac`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:740``
* ``cpmdc:NWChemNwpwStanza.smoothCutoffSigma`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:741``
* ``cpmdc:NWChemNwpwStanza.cutoffBootWavefunction`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:742``
* ``cpmdc:NWChemNwpwStanza.fastErf`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:743``
* ``cpmdc:NWChemNwpwStanza.dipoleMotion`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:744``
* ``cpmdc:NWChemNwpwStanza.dipoleMotionFilename`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:745``
* ``cpmdc:NWChemNwpwStanza.rhoUseSymmetry`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:746``
* ``cpmdc:NWChemNwpwStanza.fmm`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:747``
* ``cpmdc:NWChemNwpwStanza.fmmLmax`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:748``
* ``cpmdc:NWChemNwpwStanza.fmmLongRange`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:749``
* ``cpmdc:NWChemNwpwStanza.born`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:750``
* ``cpmdc:NWChemNwpwStanza.bornDielectric`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:751``
* ``cpmdc:NWChemNwpwStanza.bornRelax`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:752``
* ``cpmdc:NWChemNwpwStanza.bornVRadiiAngstrom`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:753``
* ``cpmdc:NWChemNwpwStanza.cpmdProperties`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:754``
* ``cpmdc:NWChemNwpwStanza.useGridComparison`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:755``
* ``cpmdc:NWChemNwpwStanza.director`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:756``
* ``cpmdc:NWChemNwpwStanza.directorFilename`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:757``
* ``cpmdc:NWChemNwpwStanza.cellExpandX`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:758``
* ``cpmdc:NWChemNwpwStanza.cellExpandY`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:759``
* ``cpmdc:NWChemNwpwStanza.cellExpandZ`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:760``
* ``cpmdc:NWChemNwpwStanza.mapping`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:761``
* ``cpmdc:NWChemNwpwStanza.rotation`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:762``
* ``cpmdc:NWChemNwpwStanza.lmaxMultipole`` — ``Int32``, default ``-1``; ``schema/Potentials.capnp:763``
* ``cpmdc:NWChemNwpwStanza.fei`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:764``
* ``cpmdc:NWChemNwpwStanza.feiFilename`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:765``
* ``cpmdc:NWChemNwpwStanza.initialVelocitiesTemperature`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:766``
* ``cpmdc:NWChemNwpwStanza.initialVelocitiesSeed`` — ``Int32``, default ``-1``; ``schema/Potentials.capnp:767``
* ``cpmdc:NWChemNwpwStanza.makeHmass2`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:768``
* ``cpmdc:NWChemNwpwStanza.translateVectorSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:769``
* ``cpmdc:NWChemNwpwStanza.translateVectorX`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:770``
* ``cpmdc:NWChemNwpwStanza.translateVectorY`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:771``
* ``cpmdc:NWChemNwpwStanza.translateVectorZ`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:772``
* ``cpmdc:NWChemNwpwStanza.translateGeometryName`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:773``
* ``cpmdc:NWChemNwpwStanza.translateReorder`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:774``
* ``cpmdc:NWChemNwpwStanza.socketType`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:775``
* ``cpmdc:NWChemNwpwStanza.socketIp`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:776``
* ``cpmdc:NWChemNwpwStanza.apcSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:777``
* ``cpmdc:NWChemNwpwStanza.apcGc`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:778``
* ``cpmdc:NWChemNwpwStanza.apcGamma`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:779``
* ``cpmdc:NWChemNwpwStanza.translation`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:780``
* ``cpmdc:NWChemNwpwStanza.minimizer`` — ``NWChemNwpwMinimizer``, default ``unspecified``; ``schema/Potentials.capnp:781``
* ``cpmdc:NWChemNwpwStanza.ksAlgorithm`` — ``NWChemNwpwKsAlgorithm``, default ``unspecified``; ``schema/Potentials.capnp:782``
* ``cpmdc:NWChemNwpwStanza.scfAlgorithm`` — ``NWChemNwpwScfAlgorithm``, default ``unspecified``; ``schema/Potentials.capnp:783``
* ``cpmdc:NWChemNwpwStanza.precondition`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:784``
* ``cpmdc:NWChemNwpwStanza.kerkerG0Set`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:785``
* ``cpmdc:NWChemNwpwStanza.kerkerG0`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:786``
* ``cpmdc:NWChemNwpwStanza.ksAlphaSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:787``
* ``cpmdc:NWChemNwpwStanza.ksAlpha`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:788``
* ``cpmdc:NWChemNwpwStanza.ksMaxitOrbSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:789``
* ``cpmdc:NWChemNwpwStanza.ksMaxitOrb`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:790``
* ``cpmdc:NWChemNwpwStanza.ksMaxitOrbsSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:791``
* ``cpmdc:NWChemNwpwStanza.ksMaxitOrbs`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:792``
* ``cpmdc:NWChemNwpwStanza.diisHistoriesSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:793``
* ``cpmdc:NWChemNwpwStanza.diisHistories`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:794``
* ``cpmdc:NWChemNwpwStanza.oneElectronGuessSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:795``
* ``cpmdc:NWChemNwpwStanza.oneElectronGuessItIn`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:796``
* ``cpmdc:NWChemNwpwStanza.oneElectronGuessItOut`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:797``
* ``cpmdc:NWChemNwpwStanza.oneElectronGuessItOrtho`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:798``
* ``cpmdc:NWChemNwpwStanza.vfieldFilenames`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:799``
* ``cpmdc:NWChemNwpwStanza.singlePrecisionHfx`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:800``
* ``cpmdc:NWChemNwpwStanza.dosAlphaSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:801``
* ``cpmdc:NWChemNwpwStanza.dosAlpha`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:802``
* ``cpmdc:NWChemNwpwStanza.dosNpointsSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:803``
* ``cpmdc:NWChemNwpwStanza.dosNpoints`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:804``
* ``cpmdc:NWChemNwpwStanza.dosEminSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:805``
* ``cpmdc:NWChemNwpwStanza.dosEmin`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:806``
* ``cpmdc:NWChemNwpwStanza.dosEmaxSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:807``
* ``cpmdc:NWChemNwpwStanza.dosEmax`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:808``
* ``cpmdc:NWChemNwpwStanza.dosFilename`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:809``
* ``cpmdc:NWChemNwpwStanza.geometryOptimize`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:810``
* ``cpmdc:NWChemNwpwStanza.auxiliaryPotentials`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:811``
* ``cpmdc:NWChemNwpwStanza.multiplicity`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:812``
* ``cpmdc:NWChemNwpwStanza.spinMode`` — ``NWChemNwpwSpinMode``, default ``unspecified``; ``schema/Potentials.capnp:813``
* ``cpmdc:NWChemNwpwStanza.allowTranslation`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:814``
* ``cpmdc:NWChemNwpwStanza.cutoffWavefunction`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:815``
* ``cpmdc:NWChemNwpwStanza.cutoffEnergy`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:816``
* ``cpmdc:NWChemNwpwStanza.mcStepStart`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:817``
* ``cpmdc:NWChemNwpwStanza.mcStepEnd`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:818``
* ``cpmdc:NWChemNwpwStanza.etMovecsA`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:819``
* ``cpmdc:NWChemNwpwStanza.etMovecsB`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:820``
* ``cpmdc:NWChemNwpwStanza.etIonA`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:821``
* ``cpmdc:NWChemNwpwStanza.etIonB`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:822``
* ``cpmdc:NWChemNwpwStanza.temperatureIon`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:823``
* ``cpmdc:NWChemNwpwStanza.temperatureIonPeriod`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:824``
* ``cpmdc:NWChemNwpwStanza.temperatureElectron`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:825``
* ``cpmdc:NWChemNwpwStanza.temperatureElectronPeriod`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:826``
* ``cpmdc:NWChemNwpwStanza.temperatureRestart`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:827``
* ``cpmdc:NWChemNwpwStanza.temperatureIonChainLength`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:828``
* ``cpmdc:NWChemNwpwStanza.temperatureElectronChainLength`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:829``
* ``cpmdc:NWChemNwpwStanza.mappingAlias`` — ``NWChemNwpwMappingAlias``, default ``unspecified``; ``schema/Potentials.capnp:830``
* ``cpmdc:NWChemNwpwStanza.virtualAliasSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:831``
* ``cpmdc:NWChemNwpwStanza.virtualAliasStart`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:832``
* ``cpmdc:NWChemNwpwStanza.virtualAliasEnd`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:833``
* ``cpmdc:NWChemNwpwStanza.boTimeStepSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:834``
* ``cpmdc:NWChemNwpwStanza.boFakeMassSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:835``
* ``cpmdc:NWChemNwpwStanza.scalingSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:836``
* ``cpmdc:NWChemNwpwStanza.npDimensionsSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:837``
* ``cpmdc:NWChemNwpwStanza.tolerancesSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:838``
* ``cpmdc:NWChemNwpwStanza.mappingSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:839``
* ``cpmdc:NWChemNwpwStanza.dosSet`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:840``
* ``cpmdc:NWChemNwpwStanza.lcaoMask`` — ``NWChemNwpwToggle``, default ``unspecified``; ``schema/Potentials.capnp:841``
* ``cpmdc:NWChemNwpwStanza.lcaoMaskUpOrbitals`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:842``
* ``cpmdc:NWChemNwpwStanza.lcaoMaskDownOrbitals`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:843``
* ``cpmdc:NWChemNwpwStanza.occupations`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:844``
* ``cpmdc:NWChemNwpwStanza.occupationStates`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:845``
* ``cpmdc:NWChemNwpwStanza.extraOrbitals`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:846``
* ``cpmdc:NWChemNwpwStanza.scalingAtomIndices`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:847``
* ``cpmdc:NWChemScfConvergence.mode`` — ``Mode``, default ``unspecified``; ``schema/Potentials.capnp:852``
* ``cpmdc:NWChemScfSemidirect.enabled`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:870``
* ``cpmdc:NWChemScfSemidirect.filesize`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:871``
* ``cpmdc:NWChemScfSemidirect.memsize`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:872``
* ``cpmdc:NWChemScfStanza.vectorsInput`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:879``
* ``cpmdc:NWChemScfStanza.vectorsOutput`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:880``
* ``cpmdc:NWChemScfStanza.maxiter`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:881``
* ``cpmdc:NWChemScfStanza.thresh`` — ``Float64``, default ``0``; ``schema/Potentials.capnp:882``
* ``cpmdc:NWChemScfStanza.tol2e`` — ``Float64``, default ``0``; ``schema/Potentials.capnp:883``
* ``cpmdc:NWChemScfStanza.noprint`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:884``
* ``cpmdc:NWChemScfStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:885``
* ``cpmdc:NWChemScfStanza.wavefunctionType`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:889``
* ``cpmdc:NWChemScfStanza.nopen`` — ``Int32``, default ``-1``; ``schema/Potentials.capnp:891``
* ``cpmdc:NWChemScfStanza.diis`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:892``
* ``cpmdc:NWChemScfStanza.diisBas`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:893``
* ``cpmdc:NWChemScfStanza.maxsub`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:894``
* ``cpmdc:NWChemScfStanza.lock`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:895``
* ``cpmdc:NWChemScfStanza.adapt`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:896``
* ``cpmdc:NWChemScfStanza.noscf`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:897``
* ``cpmdc:NWChemScfStanza.convergence`` — ``NWChemScfConvergence``, default ``(no source default)``; ``schema/Potentials.capnp:898``
* ``cpmdc:NWChemScfStanza.semidirect`` — ``NWChemScfSemidirect``, default ``(no source default)``; ``schema/Potentials.capnp:899``
* ``cpmdc:NWChemCcsdStanza.maxiter`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:905``
* ``cpmdc:NWChemCcsdStanza.thresh`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:906``
* ``cpmdc:NWChemCcsdStanza.tol2e`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:907``
* ``cpmdc:NWChemCcsdStanza.iprt`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:908``
* ``cpmdc:NWChemCcsdStanza.maxDiis`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:909``
* ``cpmdc:NWChemCcsdStanza.frozenCore`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:910``
* ``cpmdc:NWChemCcsdStanza.frozenVirtual`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:911``
* ``cpmdc:NWChemCcsdStanza.useDisk`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:912``
* ``cpmdc:NWChemCcsdStanza.sameSpinScale`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:913``
* ``cpmdc:NWChemCcsdStanza.oppositeSpinScale`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:914``
* ``cpmdc:NWChemCcsdStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:915``
* ``cpmdc:NWChemCcsdStanza.printLevel`` — ``NWChemPrintLevel``, default ``unspecified``; ``schema/Potentials.capnp:916``
* ``cpmdc:NWChemCcsdStanza.printItems`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:917``
* ``cpmdc:NWChemCcsdStanza.noPrintItems`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:918``
* ``cpmdc:NWChemCcsdStanza.doa`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:919``
* ``cpmdc:NWChemCcsdStanza.dob`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:920``
* ``cpmdc:NWChemCcsdStanza.dog`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:921``
* ``cpmdc:NWChemCcsdStanza.doh`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:922``
* ``cpmdc:NWChemCcsdStanza.dojk`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:923``
* ``cpmdc:NWChemCcsdStanza.dos`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:924``
* ``cpmdc:NWChemCcsdStanza.dod`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:925``
* ``cpmdc:NWChemCcsdStanza.useTriplesDriverNonblocking`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:926``
* ``cpmdc:NWChemCcsdStanza.useCcsdOpenmp`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:927``
* ``cpmdc:NWChemCcsdStanza.useTriplesDriverOpenmp`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:928``
* ``cpmdc:NWChemCcsdStanza.useTriplesDriverOffload`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:929``
* ``cpmdc:NWChemTceStanza.method`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:935``
* ``cpmdc:NWChemTceStanza.model`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:936``
* ``cpmdc:NWChemTceStanza.model2e`` — ``NWChemTceTwoElectronStorage``, default ``unspecified``; ``schema/Potentials.capnp:937``
* ``cpmdc:NWChemTceStanza.perturbative`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:938``
* ``cpmdc:NWChemTceStanza.ccsdVariant`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:939``
* ``cpmdc:NWChemTceStanza.noTriplesSingles`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:940``
* ``cpmdc:NWChemTceStanza.reference`` — ``NWChemTceReference``, default ``unspecified``; ``schema/Potentials.capnp:941``
* ``cpmdc:NWChemTceStanza.frozenCore`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:942``
* ``cpmdc:NWChemTceStanza.frozenVirtual`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:943``
* ``cpmdc:NWChemTceStanza.thresh`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:944``
* ``cpmdc:NWChemTceStanza.levelShift`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:945``
* ``cpmdc:NWChemTceStanza.leftLevelShift`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:946``
* ``cpmdc:NWChemTceStanza.levelShift2Alpha`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:947``
* ``cpmdc:NWChemTceStanza.levelShift2Beta`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:948``
* ``cpmdc:NWChemTceStanza.levelShift3Alpha`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:949``
* ``cpmdc:NWChemTceStanza.levelShift3Beta`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:950``
* ``cpmdc:NWChemTceStanza.maxiter`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:951``
* ``cpmdc:NWChemTceStanza.ioAlgorithm`` — ``NWChemTceIoAlgorithm``, default ``unspecified``; ``schema/Potentials.capnp:952``
* ``cpmdc:NWChemTceStanza.diis`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:953``
* ``cpmdc:NWChemTceStanza.diis2`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:954``
* ``cpmdc:NWChemTceStanza.diis3`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:955``
* ``cpmdc:NWChemTceStanza.eomSolver`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:956``
* ``cpmdc:NWChemTceStanza.hbarDimension`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:957``
* ``cpmdc:NWChemTceStanza.nroots`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:958``
* ``cpmdc:NWChemTceStanza.target`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:959``
* ``cpmdc:NWChemTceStanza.targetSymmetry`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:960``
* ``cpmdc:NWChemTceStanza.symmetry`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:961``
* ``cpmdc:NWChemTceStanza.densityMatrix`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:962``
* ``cpmdc:NWChemTceStanza.densityMatrixFile`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:963``
* ``cpmdc:NWChemTceStanza.left`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:964``
* ``cpmdc:NWChemTceStanza.multipole`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:965``
* ``cpmdc:NWChemTceStanza.fragment`` — ``Int32``, default ``-1``; ``schema/Potentials.capnp:966``
* ``cpmdc:NWChemTceStanza.recomputeFock`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:967``
* ``cpmdc:NWChemTceStanza.activeOccupiedAlpha`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:968``
* ``cpmdc:NWChemTceStanza.activeOccupiedBeta`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:969``
* ``cpmdc:NWChemTceStanza.activeVirtualAlpha`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:970``
* ``cpmdc:NWChemTceStanza.activeVirtualBeta`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:971``
* ``cpmdc:NWChemTceStanza.activeOccupied`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:972``
* ``cpmdc:NWChemTceStanza.activeUnoccupied`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:973``
* ``cpmdc:NWChemTceStanza.activeEnergyMin`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:974``
* ``cpmdc:NWChemTceStanza.activeEnergyMax`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:975``
* ``cpmdc:NWChemTceStanza.activeExcitationLevel`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:976``
* ``cpmdc:NWChemTceStanza.maxDiff`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:977``
* ``cpmdc:NWChemTceStanza.atomicTileSize`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:978``
* ``cpmdc:NWChemTceStanza.split`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:979``
* ``cpmdc:NWChemTceStanza.twoElectronMethod`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:980``
* ``cpmdc:NWChemTceStanza.diskBackend`` — ``Int32``, default ``-1``; ``schema/Potentials.capnp:981``
* ``cpmdc:NWChemTceStanza.tileSize`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:982``
* ``cpmdc:NWChemTceStanza.cudaDevices`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:983``
* ``cpmdc:NWChemTceStanza.mrcc`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:984``
* ``cpmdc:NWChemTceStanza.tccSpaces`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:985``
* ``cpmdc:NWChemTceStanza.eaCcsd`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:986``
* ``cpmdc:NWChemTceStanza.ipCcsd`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:987``
* ``cpmdc:NWChemTceStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:988``
* ``cpmdc:NWChemTceStanza.freezeMode`` — ``NWChemTceFreezeMode``, default ``unspecified``; ``schema/Potentials.capnp:989``
* ``cpmdc:NWChemTceStanza.dipole`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:990``
* ``cpmdc:NWChemTceStanza.printLevel`` — ``NWChemPrintLevel``, default ``unspecified``; ``schema/Potentials.capnp:991``
* ``cpmdc:NWChemTceStanza.printItems`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:992``
* ``cpmdc:NWChemTceStanza.quadrupole`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:993``
* ``cpmdc:NWChemTceStanza.octupole`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:994``
* ``cpmdc:NWChemMrccDataStanza.root`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1000``
* ``cpmdc:NWChemMrccDataStanza.casElectrons`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1001``
* ``cpmdc:NWChemMrccDataStanza.casOrbitals`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1002``
* ``cpmdc:NWChemMrccDataStanza.nref`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1003``
* ``cpmdc:NWChemMrccDataStanza.references`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1004``
* ``cpmdc:NWChemMrccDataStanza.se4t`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1005``
* ``cpmdc:NWChemMrccDataStanza.noAposteriori`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1006``
* ``cpmdc:NWChemMrccDataStanza.subgroupSize`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1007``
* ``cpmdc:NWChemMrccDataStanza.improveTiling`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1008``
* ``cpmdc:NWChemMrccDataStanza.usspt`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1009``
* ``cpmdc:NWChemMrccDataStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1010``
* ``cpmdc:NWChemTaskStanza.theory`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1018``
* ``cpmdc:NWChemTaskStanza.operation`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1019``
* ``cpmdc:NWChemTaskStanza.ignore`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1020``
* ``cpmdc:NWChemDriverStanza.maxiter`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1026``
* ``cpmdc:NWChemDriverStanza.tight`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1027``
* ``cpmdc:NWChemDriverStanza.loose`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1028``
* ``cpmdc:NWChemDriverStanza.xyz`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1029``
* ``cpmdc:NWChemDriverStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1030``
* ``cpmdc:NWChemDriverStanza.gmaxTol`` — ``Float64``, default ``0``; ``schema/Potentials.capnp:1031``
* ``cpmdc:NWChemDriverStanza.grmsTol`` — ``Float64``, default ``0``; ``schema/Potentials.capnp:1032``
* ``cpmdc:NWChemDriverStanza.xmaxTol`` — ``Float64``, default ``0``; ``schema/Potentials.capnp:1033``
* ``cpmdc:NWChemDriverStanza.xrmsTol`` — ``Float64``, default ``0``; ``schema/Potentials.capnp:1034``
* ``cpmdc:NWChemPropertyStanza.dipole`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1041``
* ``cpmdc:NWChemPropertyStanza.mulliken`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1042``
* ``cpmdc:NWChemPropertyStanza.quadrupol`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1043``
* ``cpmdc:NWChemPropertyStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1044``
* ``cpmdc:NWChemPropertyStanza.octupole`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1045``
* ``cpmdc:NWChemPropertyStanza.esp`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1046``
* ``cpmdc:NWChemPropertyStanza.efield`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1047``
* ``cpmdc:NWChemPropertyStanza.efieldGrad`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1048``
* ``cpmdc:NWChemPropertyStanza.electronDensity`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1049``
* ``cpmdc:NWChemPropertyStanza.spinDensity`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1050``
* ``cpmdc:NWChemPropertyStanza.spinPopulation`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1051``
* ``cpmdc:NWChemPropertyStanza.shielding`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1052``
* ``cpmdc:NWChemPropertyStanza.hyperfine`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1053``
* ``cpmdc:NWChemPropertyStanza.polarizability`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1054``
* ``cpmdc:NWChemMp2Stanza.freezeCore`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1060``
* ``cpmdc:NWChemMp2Stanza.freezeVirtual`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1061``
* ``cpmdc:NWChemMp2Stanza.tight`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1062``
* ``cpmdc:NWChemMp2Stanza.aotol2e`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1063``
* ``cpmdc:NWChemMp2Stanza.aotol2eFock`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1064``
* ``cpmdc:NWChemMp2Stanza.backtol`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1065``
* ``cpmdc:NWChemMp2Stanza.sameSpinScale`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1066``
* ``cpmdc:NWChemMp2Stanza.oppositeSpinScale`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1067``
* ``cpmdc:NWChemMp2Stanza.scs`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:1068``
* ``cpmdc:NWChemMp2Stanza.scratchDisk`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1069``
* ``cpmdc:NWChemMp2Stanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1070``
* ``cpmdc:NWChemTddftStanza.nroots`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1076``
* ``cpmdc:NWChemTddftStanza.tda`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:1077``
* ``cpmdc:NWChemTddftStanza.maxiter`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1078``
* ``cpmdc:NWChemTddftStanza.thresh`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1079``
* ``cpmdc:NWChemTddftStanza.maxvecs`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1080``
* ``cpmdc:NWChemTddftStanza.singlet`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:1081``
* ``cpmdc:NWChemTddftStanza.triplet`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:1082``
* ``cpmdc:NWChemTddftStanza.target`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1083``
* ``cpmdc:NWChemTddftStanza.targetSym`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1084``
* ``cpmdc:NWChemTddftStanza.symmetry`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:1085``
* ``cpmdc:NWChemTddftStanza.algorithm`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1086``
* ``cpmdc:NWChemTddftStanza.energyCutoff`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1087``
* ``cpmdc:NWChemTddftStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1088``
* ``cpmdc:NWChemTddftStanza.cdspectrum`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1090``
* ``cpmdc:NWChemTddftStanza.giao`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1091``
* ``cpmdc:NWChemTddftStanza.velocity`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1092``
* ``cpmdc:NWChemTddftStanza.civecs`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1093``
* ``cpmdc:NWChemTddftStanza.transden`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1094``
* ``cpmdc:NWChemTddftStanza.ewinLow`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1095``
* ``cpmdc:NWChemTddftStanza.ewinHigh`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1096``
* ``cpmdc:NWChemTddftStanza.gradRoots`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1097``
* ``cpmdc:NWChemBasisStanza.spherical`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1105``
* ``cpmdc:NWChemBasisStanza.segment`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1106``
* ``cpmdc:NWChemBasisStanza.ecp`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1107``
* ``cpmdc:NWChemBasisStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1108``
* ``cpmdc:NWChemBasisStanza.libraryRoot`` — ``NWChemBasisLibraryRoot``, default ``unspecified``; ``schema/Potentials.capnp:1109``
* ``cpmdc:NWChemBasisStanza.angularKind`` — ``NWChemBasisAngularKind``, default ``unspecified``; ``schema/Potentials.capnp:1110``
* ``cpmdc:NWChemBasisStanza.segmentMode`` — ``NWChemBasisSegmentMode``, default ``unspecified``; ``schema/Potentials.capnp:1111``
* ``cpmdc:NWChemGeometryStanza.units`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1118``
* ``cpmdc:NWChemGeometryStanza.symmetry`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1119``
* ``cpmdc:NWChemGeometryStanza.noautosym`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1120``
* ``cpmdc:NWChemGeometryStanza.noautoz`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1121``
* ``cpmdc:NWChemGeometryStanza.center`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1122``
* ``cpmdc:NWChemGeometryStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1123``
* ``cpmdc:NWChemInputStanza.kind`` — ``Kind``, default ``generic``; ``schema/Potentials.capnp:1127``
* ``cpmdc:NWChemInputStanza.generic`` — ``NWChemGenericStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1128``
* ``cpmdc:NWChemInputStanza.dft`` — ``NWChemDftStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1129``
* ``cpmdc:NWChemInputStanza.set`` — ``NWChemSetDirective``, default ``(no source default)``; ``schema/Potentials.capnp:1130``
* ``cpmdc:NWChemInputStanza.raw`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1131``
* ``cpmdc:NWChemInputStanza.module`` — ``NWChemModuleStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1132``
* ``cpmdc:NWChemInputStanza.pseudopotential`` — ``NWChemPseudopotentialStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1133``
* ``cpmdc:NWChemInputStanza.scf`` — ``NWChemScfStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1134``
* ``cpmdc:NWChemInputStanza.taskStanza`` — ``NWChemTaskStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1135``
* ``cpmdc:NWChemInputStanza.driver`` — ``NWChemDriverStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1136``
* ``cpmdc:NWChemInputStanza.property`` — ``NWChemPropertyStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1137``
* ``cpmdc:NWChemInputStanza.basisStanza`` — ``NWChemBasisStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1138``
* ``cpmdc:NWChemInputStanza.geometry`` — ``NWChemGeometryStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1139``
* ``cpmdc:NWChemInputStanza.nwpw`` — ``NWChemNwpwStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1140``
* ``cpmdc:NWChemInputStanza.ccsd`` — ``NWChemCcsdStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1141``
* ``cpmdc:NWChemInputStanza.tce`` — ``NWChemTceStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1142``
* ``cpmdc:NWChemInputStanza.mrccData`` — ``NWChemMrccDataStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1143``
* ``cpmdc:NWChemInputStanza.brillouinZone`` — ``NWChemBrillouinZoneStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1144``
* ``cpmdc:NWChemInputStanza.simulationCell`` — ``NWChemSimulationCellStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1145``
* ``cpmdc:NWChemInputStanza.mp2`` — ``NWChemMp2Stanza``, default ``(no source default)``; ``schema/Potentials.capnp:1146``
* ``cpmdc:NWChemInputStanza.tddft`` — ``NWChemTddftStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1147``
* ``cpmdc:NWChemInputStanza.relativistic`` — ``NWChemRelativisticStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1148``
* ``cpmdc:NWChemInputStanza.cosmo`` — ``NWChemCosmoStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1149``
* ``cpmdc:NWChemInputStanza.smd`` — ``NWChemSmdStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1150``
* ``cpmdc:NWChemInputStanza.constraints`` — ``NWChemConstraintsStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1151``
* ``cpmdc:NWChemInputStanza.vib`` — ``NWChemVibStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1152``
* ``cpmdc:NWChemInputStanza.bq`` — ``NWChemBqStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1153``
* ``cpmdc:NWChemInputStanza.dplot`` — ``NWChemDplotStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1154``
* ``cpmdc:NWChemInputStanza.esp`` — ``NWChemEspStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1155``
* ``cpmdc:NWChemInputStanza.qmd`` — ``NWChemQmdStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1156``
* ``cpmdc:NWChemInputStanza.raman`` — ``NWChemRamanStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1157``
* ``cpmdc:NWChemInputStanza.fon`` — ``NWChemFonStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1158``
* ``cpmdc:NWChemInputStanza.neb`` — ``NWChemNebStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1159``
* ``cpmdc:NWChemInputStanza.stringMethod`` — ``NWChemStringStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1160``
* ``cpmdc:NWChemInputStanza.gw`` — ``NWChemGwStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1161``
* ``cpmdc:NWChemInputStanza.etrans`` — ``NWChemEtransStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1162``
* ``cpmdc:NWChemInputStanza.rism`` — ``NWChemRismStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1163``
* ``cpmdc:NWChemInputStanza.dimQm`` — ``NWChemDimQmStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1164``
* ``cpmdc:NWChemInputStanza.metadynamics`` — ``NWChemMetadynamicsStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1165``
* ``cpmdc:NWChemInputStanza.cellOptimize`` — ``NWChemCellOptimizeStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1166``
* ``cpmdc:NWChemInputStanza.mepgs`` — ``NWChemMepgsStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1167``
* ``cpmdc:NWChemInputStanza.tropt`` — ``NWChemTroptStanza``, default ``(no source default)``; ``schema/Potentials.capnp:1168``
* ``cpmdc:NWChemParams.basis`` — ``Text``, default ``sto-3g``; ``schema/Potentials.capnp:1216``
* ``cpmdc:NWChemParams.theory`` — ``Text``, default ``scf``; ``schema/Potentials.capnp:1217``
* ``cpmdc:NWChemParams.scfType`` — ``Text``, default ``rhf``; ``schema/Potentials.capnp:1218``
* ``cpmdc:NWChemParams.charge`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1219``
* ``cpmdc:NWChemParams.multiplicity`` — ``Int32``, default ``1``; ``schema/Potentials.capnp:1220``
* ``cpmdc:NWChemParams.enginePath`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1221``
* ``cpmdc:NWChemParams.nwchemRoot`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1222``
* ``cpmdc:NWChemParams.task`` — ``Text``, default ``gradient``; ``schema/Potentials.capnp:1223``
* ``cpmdc:NWChemParams.title`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1224``
* ``cpmdc:NWChemParams.memoryMb`` — ``UInt32``, default ``0``; ``schema/Potentials.capnp:1225``
* ``cpmdc:NWChemParams.scratchDir`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1226``
* ``cpmdc:NWChemParams.permanentDir`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1227``
* ``cpmdc:NWChemParams.inputBlocks`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1228``
* ``cpmdc:NWChemParams.inputStanzas`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1229``
* ``cpmdc:CPMDDirective.keyword`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1242``
* ``cpmdc:CPMDDirective.args`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1243``
* ``cpmdc:CPMDGenericSection.name`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1247``
* ``cpmdc:CPMDGenericSection.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1248``
* ``cpmdc:CPMDSetDirective.key`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1252``
* ``cpmdc:CPMDSetDirective.value`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1253``
* ``cpmdc:CPMDKPoint.coordinates`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1257``
* ``cpmdc:CPMDKPoint.weight`` — ``Float64``, default ``1.0``; ``schema/Potentials.capnp:1258``
* ``cpmdc:CPMDKPointBand.points`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1262``
* ``cpmdc:CPMDKPointBand.start`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1263``
* ``cpmdc:CPMDKPointBand.end`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1264``
* ``cpmdc:CPMDCouplingSurface.stateI`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1268``
* ``cpmdc:CPMDCouplingSurface.stateJ`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1269``
* ``cpmdc:CPMDCouplingSurface.coefficient`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1270``
* ``cpmdc:CPMDSystemSection.symmetry`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1274``
* ``cpmdc:CPMDSystemSection.angstrom`` — ``Bool``, default ``true``; ``schema/Potentials.capnp:1275``
* ``cpmdc:CPMDSystemSection.cell`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1276``
* ``cpmdc:CPMDSystemSection.cutOffRy`` — ``Float64``, default ``70.0``; ``schema/Potentials.capnp:1277``
* ``cpmdc:CPMDSystemSection.scale`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1278``
* ``cpmdc:CPMDSystemSection.charge`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1279``
* ``cpmdc:CPMDSystemSection.multiplicity`` — ``Int32``, default ``1``; ``schema/Potentials.capnp:1280``
* ``cpmdc:CPMDSystemSection.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1281``
* ``cpmdc:CPMDSystemSection.densityCutOffRy`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1282``
* ``cpmdc:CPMDSystemSection.poissonSolver`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1283``
* ``cpmdc:CPMDSystemSection.poissonParameter`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1284``
* ``cpmdc:CPMDSystemSection.surface`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1285``
* ``cpmdc:CPMDSystemSection.referenceCell`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1286``
* ``cpmdc:CPMDSystemSection.classicalCell`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1287``
* ``cpmdc:CPMDSystemSection.isotropicCell`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1288``
* ``cpmdc:CPMDSystemSection.zFlexibleCell`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1289``
* ``cpmdc:CPMDSystemSection.densityCutoffNumber`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1290``
* ``cpmdc:CPMDSystemSection.dual`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1291``
* ``cpmdc:CPMDSystemSection.constantCutoff`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1292``
* ``cpmdc:CPMDSystemSection.mesh`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1293``
* ``cpmdc:CPMDSystemSection.scaleCartesian`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1294``
* ``cpmdc:CPMDSystemSection.doubleGrid`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1295``
* ``cpmdc:CPMDSystemSection.symmetrizeCoordinates`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1296``
* ``cpmdc:CPMDSystemSection.tesr`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1297``
* ``cpmdc:CPMDSystemSection.polymer`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1298``
* ``cpmdc:CPMDSystemSection.cluster`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1299``
* ``cpmdc:CPMDSystemSection.cutoffShape`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1300``
* ``cpmdc:CPMDSystemSection.hfxCutoff`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1301``
* ``cpmdc:CPMDSystemSection.boxWalls`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1302``
* ``cpmdc:CPMDSystemSection.nSup`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1303``
* ``cpmdc:CPMDSystemSection.states`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1304``
* ``cpmdc:CPMDSystemSection.occupation`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1305``
* ``cpmdc:CPMDSystemSection.occupationFixed`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1306``
* ``cpmdc:CPMDSystemSection.externalField`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1307``
* ``cpmdc:CPMDSystemSection.pressure`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1308``
* ``cpmdc:CPMDSystemSection.stressTensor`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1309``
* ``cpmdc:CPMDSystemSection.shockVelocity`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1310``
* ``cpmdc:CPMDSystemSection.checkSymmetryPrecision`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1311``
* ``cpmdc:CPMDSystemSection.checkSymmetryOff`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1312``
* ``cpmdc:CPMDSystemSection.wCut`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1313``
* ``cpmdc:CPMDSystemSection.wGauss`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1314``
* ``cpmdc:CPMDSystemSection.lowSpinExcitation`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1315``
* ``cpmdc:CPMDSystemSection.lowSpinExcitationLsets`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1316``
* ``cpmdc:CPMDSystemSection.lseParameters`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1317``
* ``cpmdc:CPMDSystemSection.modifiedGoedecker`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1318``
* ``cpmdc:CPMDSystemSection.modifiedGoedeckerParameters`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1319``
* ``cpmdc:CPMDSystemSection.energyProfile`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1320``
* ``cpmdc:CPMDSystemSection.pointGroup`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1321``
* ``cpmdc:CPMDSystemSection.pointGroupDelta`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1322``
* ``cpmdc:CPMDSystemSection.pointGroupMolecule`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1323``
* ``cpmdc:CPMDSystemSection.scaleX`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1324``
* ``cpmdc:CPMDSystemSection.scaleY`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1325``
* ``cpmdc:CPMDSystemSection.scaleZ`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1326``
* ``cpmdc:CPMDSystemSection.kpoints`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1327``
* ``cpmdc:CPMDSystemSection.kpointsScaled`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1328``
* ``cpmdc:CPMDSystemSection.kpointsOnlyDiagonal`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1329``
* ``cpmdc:CPMDSystemSection.kpointsMonkhorstPack`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1330``
* ``cpmdc:CPMDSystemSection.kpointsMonkhorstSymmetrized`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1331``
* ``cpmdc:CPMDSystemSection.kpointsMonkhorstFull`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1332``
* ``cpmdc:CPMDSystemSection.kpointsMonkhorstKdp`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1333``
* ``cpmdc:CPMDSystemSection.kpointsMonkhorstShift`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1334``
* ``cpmdc:CPMDSystemSection.kpointBands`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1335``
* ``cpmdc:CPMDSystemSection.kpointsBlock`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1336``
* ``cpmdc:CPMDSystemSection.kpointsBlockAll`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1337``
* ``cpmdc:CPMDSystemSection.kpointsBlockCalculated`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1338``
* ``cpmdc:CPMDSystemSection.kpointsBlockNoSwap`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1339``
* ``cpmdc:CPMDSystemSection.lowSpinExcitationPenalty`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1340``
* ``cpmdc:CPMDSystemSection.cdftDonorAtoms`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1341``
* ``cpmdc:CPMDSystemSection.cdftDonorWeights`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1342``
* ``cpmdc:CPMDSystemSection.cdftAcceptorAtoms`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1343``
* ``cpmdc:CPMDSystemSection.cdftAcceptorHdasDonors`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1344``
* ``cpmdc:CPMDSystemSection.cdftAcceptorWeights`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1345``
* ``cpmdc:CPMDSystemSection.couplingsFiniteDifference`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1346``
* ``cpmdc:CPMDSystemSection.couplingsFiniteDifferenceDisplacement`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1347``
* ``cpmdc:CPMDSystemSection.couplingsProductDisplacement`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1348``
* ``cpmdc:CPMDSystemSection.couplingsLinres`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1349``
* ``cpmdc:CPMDSystemSection.couplingsLinresTolerance`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1350``
* ``cpmdc:CPMDSystemSection.couplingsLinresNvects`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1351``
* ``cpmdc:CPMDSystemSection.couplingsLinresSpecify`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1352``
* ``cpmdc:CPMDSystemSection.couplingsLinresBruteForce`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1353``
* ``cpmdc:CPMDSystemSection.couplingsLinresThresholds`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1354``
* ``cpmdc:CPMDSystemSection.couplingsSurfaces`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1355``
* ``cpmdc:CPMDSystemSection.couplingsFiniteDifferenceAtoms`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1356``
* ``cpmdc:CPMDSystemSection.cellAbsolute`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1357``
* ``cpmdc:CPMDSystemSection.cellDegree`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1358``
* ``cpmdc:CPMDSystemSection.referenceCellAbsolute`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1359``
* ``cpmdc:CPMDSystemSection.referenceCellDegree`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1360``
* ``cpmdc:CPMDSystemSection.classicalCellAbsolute`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1361``
* ``cpmdc:CPMDSystemSection.classicalCellDegree`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1362``
* ``cpmdc:CPMDSystemSection.cellVectors`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1363``
* ``cpmdc:CPMDSystemSection.referenceCellVectors`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1364``
* ``cpmdc:CPMDCpmdSection.optimizeWavefunction`` — ``Bool``, default ``true``; ``schema/Potentials.capnp:1368``
* ``cpmdc:CPMDCpmdSection.molecularDynamics`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1369``
* ``cpmdc:CPMDCpmdSection.convergenceOrbitals`` — ``Float64``, default ``1.0e-6``; ``schema/Potentials.capnp:1370``
* ``cpmdc:CPMDCpmdSection.maxStep`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1371``
* ``cpmdc:CPMDCpmdSection.timestep`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1372``
* ``cpmdc:CPMDCpmdSection.restartWavefunction`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1373``
* ``cpmdc:CPMDCpmdSection.trajectory`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1374``
* ``cpmdc:CPMDCpmdSection.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1375``
* ``cpmdc:CPMDCpmdSection.optimizeGeometry`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1376``
* ``cpmdc:CPMDCpmdSection.maxIter`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1377``
* ``cpmdc:CPMDCpmdSection.convergenceGeometry`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1378``
* ``cpmdc:CPMDCpmdSection.electronMass`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1379``
* ``cpmdc:CPMDCpmdSection.molecularDynamicsCp`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1380``
* ``cpmdc:CPMDCpmdSection.molecularDynamicsBo`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1381``
* ``cpmdc:CPMDCpmdSection.molecularDynamicsEh`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1382``
* ``cpmdc:CPMDCpmdSection.molecularDynamicsPt`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1383``
* ``cpmdc:CPMDCpmdSection.molecularDynamicsClassical`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1384``
* ``cpmdc:CPMDCpmdSection.molecularDynamicsFile`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1385``
* ``cpmdc:CPMDCpmdSection.nose`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1386``
* ``cpmdc:CPMDCpmdSection.noseIons`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1387``
* ``cpmdc:CPMDCpmdSection.noseElectrons`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1388``
* ``cpmdc:CPMDCpmdSection.berendsen`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1389``
* ``cpmdc:CPMDCpmdSection.langevin`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1390``
* ``cpmdc:CPMDCpmdSection.annealing`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1391``
* ``cpmdc:CPMDCpmdSection.quench`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1392``
* ``cpmdc:CPMDCpmdSection.rattle`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1393``
* ``cpmdc:CPMDCpmdSection.shake`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1394``
* ``cpmdc:CPMDCpmdSection.constraint`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1395``
* ``cpmdc:CPMDCpmdSection.trotter`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1396``
* ``cpmdc:CPMDCpmdSection.restart`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1397``
* ``cpmdc:CPMDCpmdSection.printOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1398``
* ``cpmdc:CPMDCpmdSection.storeOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1399``
* ``cpmdc:CPMDCpmdSection.centerMoleculeOff`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1400``
* ``cpmdc:CPMDCpmdSection.centerMoleculeOn`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1401``
* ``cpmdc:CPMDCpmdSection.diis`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1402``
* ``cpmdc:CPMDCpmdSection.odiis`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1403``
* ``cpmdc:CPMDCpmdSection.pcg`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1404``
* ``cpmdc:CPMDCpmdSection.diagonalization`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1405``
* ``cpmdc:CPMDCpmdSection.freeEnergy`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1406``
* ``cpmdc:CPMDCpmdSection.interface`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1407``
* ``cpmdc:CPMDCpmdSection.qmmm`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1408``
* ``cpmdc:CPMDCpmdSection.bicanonicalEnsemble`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1409``
* ``cpmdc:CPMDCpmdSection.cdft`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1410``
* ``cpmdc:CPMDCpmdSection.properties`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1411``
* ``cpmdc:CPMDCpmdSection.vdwCorrection`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1412``
* ``cpmdc:CPMDCpmdSection.vdwWannier`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1413``
* ``cpmdc:CPMDCpmdSection.dcacp`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1414``
* ``cpmdc:CPMDCpmdSection.isolatedMolecule`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1415``
* ``cpmdc:CPMDCpmdSection.maxRuntime`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1416``
* ``cpmdc:CPMDCpmdSection.timestepElectrons`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1417``
* ``cpmdc:CPMDCpmdSection.timestepIons`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1418``
* ``cpmdc:CPMDCpmdSection.cellMass`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1419``
* ``cpmdc:CPMDCpmdSection.temperatureElectron`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1420``
* ``cpmdc:CPMDCpmdSection.temperature`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1421``
* ``cpmdc:CPMDCpmdSection.temperatureRamp`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1422``
* ``cpmdc:CPMDCpmdSection.temperatureRampTime`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1423``
* ``cpmdc:CPMDCpmdSection.temperatureRampRate`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1424``
* ``cpmdc:CPMDCpmdSection.rescaleOldVelocities`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1425``
* ``cpmdc:CPMDCpmdSection.reverseVelocities`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1426``
* ``cpmdc:CPMDCpmdSection.subtractComVelocity`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1427``
* ``cpmdc:CPMDCpmdSection.subtractRotVelocity`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1428``
* ``cpmdc:CPMDCpmdSection.prngSeed`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1429``
* ``cpmdc:CPMDCpmdSection.tempControlIons`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1430``
* ``cpmdc:CPMDCpmdSection.tempControlElectrons`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1431``
* ``cpmdc:CPMDCpmdSection.tempControlCell`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1432``
* ``cpmdc:CPMDCpmdSection.berendsenIons`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1433``
* ``cpmdc:CPMDCpmdSection.berendsenElectrons`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1434``
* ``cpmdc:CPMDCpmdSection.berendsenCell`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1435``
* ``cpmdc:CPMDCpmdSection.noseIonsThermostat`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1436``
* ``cpmdc:CPMDCpmdSection.noseElectronsThermostat`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1437``
* ``cpmdc:CPMDCpmdSection.noseCellThermostat`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1438``
* ``cpmdc:CPMDCpmdSection.noseParameters`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1439``
* ``cpmdc:CPMDCpmdSection.convergenceCell`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1440``
* ``cpmdc:CPMDCpmdSection.convergenceAdapt`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1441``
* ``cpmdc:CPMDCpmdSection.convergenceEnergy`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1442``
* ``cpmdc:CPMDCpmdSection.convergenceCalfor`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1443``
* ``cpmdc:CPMDCpmdSection.convergenceRelax`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1444``
* ``cpmdc:CPMDCpmdSection.convergenceRhofix`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1445``
* ``cpmdc:CPMDCpmdSection.convergenceInitial`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1446``
* ``cpmdc:CPMDCpmdSection.convergenceConstraint`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1447``
* ``cpmdc:CPMDCpmdSection.annealingIons`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1448``
* ``cpmdc:CPMDCpmdSection.annealingElectrons`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1449``
* ``cpmdc:CPMDCpmdSection.annealingCell`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1450``
* ``cpmdc:CPMDCpmdSection.dampingIons`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1451``
* ``cpmdc:CPMDCpmdSection.dampingElectrons`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1452``
* ``cpmdc:CPMDCpmdSection.dampingCell`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1453``
* ``cpmdc:CPMDCpmdSection.hessian`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1454``
* ``cpmdc:CPMDCpmdSection.project`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1455``
* ``cpmdc:CPMDCpmdSection.stressTensorSample`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1456``
* ``cpmdc:CPMDCpmdSection.stressTensorVirial`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1457``
* ``cpmdc:CPMDCpmdSection.classStressSample`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1458``
* ``cpmdc:CPMDCpmdSection.storeSelection`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1459``
* ``cpmdc:CPMDCpmdSection.storeInterval`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1460``
* ``cpmdc:CPMDCpmdSection.storeSelfConsistentInterval`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1461``
* ``cpmdc:CPMDCpmdSection.storeOffSelection`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1462``
* ``cpmdc:CPMDCpmdSection.restFileCount`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1463``
* ``cpmdc:CPMDCpmdSection.restFileSample`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1464``
* ``cpmdc:CPMDCpmdSection.trajectoryOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1465``
* ``cpmdc:CPMDCpmdSection.trajectorySample`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1466``
* ``cpmdc:CPMDCpmdSection.trajectoryRange`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1467``
* ``cpmdc:CPMDCpmdSection.movieSample`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1468``
* ``cpmdc:CPMDCpmdSection.movieOff`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1469``
* ``cpmdc:CPMDCpmdSection.energyBands`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1470``
* ``cpmdc:CPMDCpmdSection.externalPotential`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1471``
* ``cpmdc:CPMDCpmdSection.externalPotentialAdd`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1472``
* ``cpmdc:CPMDCpmdSection.electrostaticPotential`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1473``
* ``cpmdc:CPMDCpmdSection.electrostaticPotentialSample`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1474``
* ``cpmdc:CPMDCpmdSection.dipoleDynamicsSample`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1475``
* ``cpmdc:CPMDCpmdSection.dipoleDynamicsWannier`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1476``
* ``cpmdc:CPMDCpmdSection.rhoOut`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1477``
* ``cpmdc:CPMDCpmdSection.rhoOutSample`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1478``
* ``cpmdc:CPMDCpmdSection.rhoOutBandsCount`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1479``
* ``cpmdc:CPMDCpmdSection.rhoOutBands`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1480``
* ``cpmdc:CPMDCpmdSection.elf`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1481``
* ``cpmdc:CPMDCpmdSection.elfParameters`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1482``
* ``cpmdc:CPMDCpmdSection.wannierParameters`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1483``
* ``cpmdc:CPMDCpmdSection.wannierOptimization`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1484``
* ``cpmdc:CPMDCpmdSection.wannierType`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1485``
* ``cpmdc:CPMDCpmdSection.wannierReference`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1486``
* ``cpmdc:CPMDCpmdSection.wannierSerial`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1487``
* ``cpmdc:CPMDCpmdSection.wannierDos`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1488``
* ``cpmdc:CPMDCpmdSection.wannierMolecular`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1489``
* ``cpmdc:CPMDCpmdSection.wannierWfnOutOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1490``
* ``cpmdc:CPMDCpmdSection.wannierWfnOutPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1491``
* ``cpmdc:CPMDCpmdSection.compress`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1492``
* ``cpmdc:CPMDCpmdSection.memory`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1493``
* ``cpmdc:CPMDCpmdSection.realSpaceWfnKeep`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1494``
* ``cpmdc:CPMDCpmdSection.realSpaceWfnSize`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1495``
* ``cpmdc:CPMDCpmdSection.splineOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1496``
* ``cpmdc:CPMDCpmdSection.splinePoints`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1497``
* ``cpmdc:CPMDCpmdSection.splineRange`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1498``
* ``cpmdc:CPMDCpmdSection.finiteDifferences`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1499``
* ``cpmdc:CPMDCpmdSection.taskGroups`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1500``
* ``cpmdc:CPMDCpmdSection.taskGroupsCount`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1501``
* ``cpmdc:CPMDCpmdSection.distributeFnl`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1502``
* ``cpmdc:CPMDCpmdSection.filePath`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1503``
* ``cpmdc:CPMDCpmdSection.benchmark`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1504``
* ``cpmdc:CPMDCpmdSection.mirror`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1505``
* ``cpmdc:CPMDCpmdSection.shiftPotential`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1506``
* ``cpmdc:CPMDCpmdSection.glocalizationParameters`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1507``
* ``cpmdc:CPMDCpmdSection.glocalizationOptimization`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1508``
* ``cpmdc:CPMDCpmdSection.gfunctionalType`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1509``
* ``cpmdc:CPMDCpmdSection.spreadRspace`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1510``
* ``cpmdc:CPMDCpmdSection.gUnitarityOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1511``
* ``cpmdc:CPMDCpmdSection.stepTuning`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1512``
* ``cpmdc:CPMDCpmdSection.gAntisym`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1513``
* ``cpmdc:CPMDCpmdSection.gAntisymPenalty`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1514``
* ``cpmdc:CPMDCpmdSection.gKick`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1515``
* ``cpmdc:CPMDCpmdSection.gComplex`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1516``
* ``cpmdc:CPMDCpmdSection.gReal`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1517``
* ``cpmdc:CPMDCpmdSection.readMatrix`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1518``
* ``cpmdc:CPMDCpmdSection.gStepTune`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1519``
* ``cpmdc:CPMDCpmdSection.glocWfnOutOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1520``
* ``cpmdc:CPMDCpmdSection.glocWfnOutPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1521``
* ``cpmdc:CPMDCpmdSection.noGeoCheck`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1522``
* ``cpmdc:CPMDCpmdSection.brokenSymmetry`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1523``
* ``cpmdc:CPMDCpmdSection.distributedLinalg`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1524``
* ``cpmdc:CPMDCpmdSection.linalgNewOrtho`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1525``
* ``cpmdc:CPMDCpmdSection.disorthoBlockSize`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1526``
* ``cpmdc:CPMDCpmdSection.statesBlockSize`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1527``
* ``cpmdc:CPMDCpmdSection.allToAllPrecision`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1528``
* ``cpmdc:CPMDCpmdSection.gshell`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1529``
* ``cpmdc:CPMDCpmdSection.localPotential`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1530``
* ``cpmdc:CPMDCpmdSection.cdftOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1531``
* ``cpmdc:CPMDCpmdSection.cdftPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1532``
* ``cpmdc:CPMDCpmdSection.cdftHdaPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1533``
* ``cpmdc:CPMDCpmdSection.vgfactor`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1534``
* ``cpmdc:CPMDCpmdSection.vMirror`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1535``
* ``cpmdc:CPMDCpmdSection.combineSystemsOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1536``
* ``cpmdc:CPMDCpmdSection.combineSystemsPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1537``
* ``cpmdc:CPMDCpmdSection.combineSystemsSabPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1538``
* ``cpmdc:CPMDCpmdSection.kshamOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1539``
* ``cpmdc:CPMDCpmdSection.kshamPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1540``
* ``cpmdc:CPMDCpmdSection.czonesSet`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1541``
* ``cpmdc:CPMDCpmdSection.woutOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1542``
* ``cpmdc:CPMDCpmdSection.woutPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1543``
* ``cpmdc:CPMDCpmdSection.xfmqcTrajectories`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1544``
* ``cpmdc:CPMDCpmdSection.molecularDynamicsFileOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1545``
* ``cpmdc:CPMDCpmdSection.molecularDynamicsBdTrajectories`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1546``
* ``cpmdc:CPMDCpmdSection.parrinelloRahmanOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1547``
* ``cpmdc:CPMDCpmdSection.optimizeGeometryOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1548``
* ``cpmdc:CPMDCpmdSection.optimizeGeometrySample`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1549``
* ``cpmdc:CPMDCpmdSection.optimizeCombinedOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1550``
* ``cpmdc:CPMDCpmdSection.optimizeCombinedSample`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1551``
* ``cpmdc:CPMDCpmdSection.cheby`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1552``
* ``cpmdc:CPMDCpmdSection.cayley`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1553``
* ``cpmdc:CPMDCpmdSection.rungeKutta`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1554``
* ``cpmdc:CPMDCpmdSection.forceMatch`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1555``
* ``cpmdc:CPMDCpmdSection.debugOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1556``
* ``cpmdc:CPMDCpmdSection.kohnShamEnergiesOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1557``
* ``cpmdc:CPMDCpmdSection.kohnShamEnergiesCount`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1558``
* ``cpmdc:CPMDCpmdSection.surfaceHoppingOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1559``
* ``cpmdc:CPMDCpmdSection.roksOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1560``
* ``cpmdc:CPMDCpmdSection.roksExpertPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1561``
* ``cpmdc:CPMDCpmdSection.pathSampling`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1562``
* ``cpmdc:CPMDCpmdSection.fixrhoUpwfnOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1563``
* ``cpmdc:CPMDCpmdSection.fixrhoVectors`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1564``
* ``cpmdc:CPMDCpmdSection.fixrhoLoop`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1565``
* ``cpmdc:CPMDCpmdSection.fixrhoWftol`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1566``
* ``cpmdc:CPMDCpmdSection.bogoliubovCorrection`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1567``
* ``cpmdc:CPMDCpmdSection.vibrationalAnalysisOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1568``
* ``cpmdc:CPMDCpmdSection.vibrationalAnalysisSample`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1569``
* ``cpmdc:CPMDCpmdSection.vibrationalAnalysisMode`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1570``
* ``cpmdc:CPMDCpmdSection.electronicSpectra`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1571``
* ``cpmdc:CPMDCpmdSection.spinOrbitCouplingStates`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1572``
* ``cpmdc:CPMDCpmdSection.propagationSpectra`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1573``
* ``cpmdc:CPMDCpmdSection.propagationDistrub`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1574``
* ``cpmdc:CPMDCpmdSection.gaugePulse`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1575``
* ``cpmdc:CPMDCpmdSection.gaugeFieldFrequency`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1576``
* ``cpmdc:CPMDCpmdSection.nacv`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1577``
* ``cpmdc:CPMDCpmdSection.orbitalHardnessOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1578``
* ``cpmdc:CPMDCpmdSection.pathIntegral`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1579``
* ``cpmdc:CPMDCpmdSection.pathMinimization`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1580``
* ``cpmdc:CPMDCpmdSection.langevinOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1581``
* ``cpmdc:CPMDCpmdSection.langevinParameter`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1582``
* ``cpmdc:CPMDCpmdSection.qmmmEasy`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1583``
* ``cpmdc:CPMDCpmdSection.interfaceOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1584``
* ``cpmdc:CPMDCpmdSection.trotterFactorCount`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1585``
* ``cpmdc:CPMDCpmdSection.trotterFactorPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1586``
* ``cpmdc:CPMDCpmdSection.linearResponse`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1587``
* ``cpmdc:CPMDCpmdSection.harmonicReference`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1588``
* ``cpmdc:CPMDCpmdSection.scaledMasses`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1589``
* ``cpmdc:CPMDCpmdSection.tddft`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1590``
* ``cpmdc:CPMDCpmdSection.ssic`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1591``
* ``cpmdc:CPMDCpmdSection.nonorthogonalOrbitalsOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1592``
* ``cpmdc:CPMDCpmdSection.nonorthogonalOrbitalsLimit`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1593``
* ``cpmdc:CPMDCpmdSection.lanczosDiagonalizationOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1594``
* ``cpmdc:CPMDCpmdSection.lanczosParametersCount`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1595``
* ``cpmdc:CPMDCpmdSection.lanczosParametersPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1596``
* ``cpmdc:CPMDCpmdSection.davidsonDiagonalization`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1597``
* ``cpmdc:CPMDCpmdSection.davidsonParameters`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1598``
* ``cpmdc:CPMDCpmdSection.alexanderMixing`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1599``
* ``cpmdc:CPMDCpmdSection.andersonMixingGspace`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1600``
* ``cpmdc:CPMDCpmdSection.andersonMixingCount`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1601``
* ``cpmdc:CPMDCpmdSection.andersonMixingPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1602``
* ``cpmdc:CPMDCpmdSection.broydenMixingOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1603``
* ``cpmdc:CPMDCpmdSection.broydenMixingPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1604``
* ``cpmdc:CPMDCpmdSection.diisMixingCount`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1605``
* ``cpmdc:CPMDCpmdSection.diisMixingPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1606``
* ``cpmdc:CPMDCpmdSection.moverhoMixing`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1607``
* ``cpmdc:CPMDCpmdSection.extrapolateWfnOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1608``
* ``cpmdc:CPMDCpmdSection.extrapolateWfnOrder`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1609``
* ``cpmdc:CPMDCpmdSection.extrapolateConstraintOrder`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1610``
* ``cpmdc:CPMDCpmdSection.tsdeOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1611``
* ``cpmdc:CPMDCpmdSection.tsdpOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1612``
* ``cpmdc:CPMDCpmdSection.tcgp`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1613``
* ``cpmdc:CPMDCpmdSection.tsdc`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1614``
* ``cpmdc:CPMDCpmdSection.steepestDescentOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1615``
* ``cpmdc:CPMDCpmdSection.conjugateGradientOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1616``
* ``cpmdc:CPMDCpmdSection.odiisOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1617``
* ``cpmdc:CPMDCpmdSection.odiisVectors`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1618``
* ``cpmdc:CPMDCpmdSection.hamiltonianCutoff`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1619``
* ``cpmdc:CPMDCpmdSection.gdiisVectors`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1620``
* ``cpmdc:CPMDCpmdSection.lbfgsOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1621``
* ``cpmdc:CPMDCpmdSection.lbfgsPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1622``
* ``cpmdc:CPMDCpmdSection.prfoOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1623``
* ``cpmdc:CPMDCpmdSection.prfoPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1624``
* ``cpmdc:CPMDCpmdSection.hesscore`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1625``
* ``cpmdc:CPMDCpmdSection.bfgs`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1626``
* ``cpmdc:CPMDCpmdSection.rfoOrder`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1627``
* ``cpmdc:CPMDCpmdSection.inrParametersCount`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1628``
* ``cpmdc:CPMDCpmdSection.inrParametersPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1629``
* ``cpmdc:CPMDCpmdSection.implicitNewtonOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1630``
* ``cpmdc:CPMDCpmdSection.implicitNewtonMaxIter`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1631``
* ``cpmdc:CPMDCpmdSection.mixsd`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1632``
* ``cpmdc:CPMDCpmdSection.mixdiis`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1633``
* ``cpmdc:CPMDCpmdSection.restartOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1634``
* ``cpmdc:CPMDCpmdSection.intFileOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1635``
* ``cpmdc:CPMDCpmdSection.intFileName`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1636``
* ``cpmdc:CPMDCpmdSection.initializeWavefunctionOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1637``
* ``cpmdc:CPMDCpmdSection.rattleParameters`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1638``
* ``cpmdc:CPMDCpmdSection.orthogonalizationOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1639``
* ``cpmdc:CPMDCpmdSection.quenchOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1640``
* ``cpmdc:CPMDCpmdSection.randomizeOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1641``
* ``cpmdc:CPMDCpmdSection.randomizeAmplitude`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1642``
* ``cpmdc:CPMDCpmdSection.useMts`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1643``
* ``cpmdc:CPMDCpmdSection.nabdyZmax`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1644``
* ``cpmdc:CPMDCpmdSection.nabdySoft`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1645``
* ``cpmdc:CPMDCpmdSection.nabdyRedistributeAmplitude`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1646``
* ``cpmdc:CPMDCpmdSection.nabdyScaleP`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1647``
* ``cpmdc:CPMDCpmdSection.nabdyThermo`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1648``
* ``cpmdc:CPMDCpmdSection.noseIonsOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1649``
* ``cpmdc:CPMDCpmdSection.useInStream`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1650``
* ``cpmdc:CPMDCpmdSection.useOutStream`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1651``
* ``cpmdc:CPMDCpmdSection.useCublas`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1652``
* ``cpmdc:CPMDCpmdSection.useCufft`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1653``
* ``cpmdc:CPMDCpmdSection.blasNStreamsPerDevice`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1654``
* ``cpmdc:CPMDCpmdSection.blasNDevicesPerTask`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1655``
* ``cpmdc:CPMDCpmdSection.fftNStreamsPerDevice`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1656``
* ``cpmdc:CPMDCpmdSection.fftNDevicesPerTask`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1657``
* ``cpmdc:CPMDCpmdSection.useMpiIo`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1658``
* ``cpmdc:CPMDCpmdSection.traceOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1659``
* ``cpmdc:CPMDCpmdSection.traceProcedure`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1660``
* ``cpmdc:CPMDCpmdSection.traceMaxDepth`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1661``
* ``cpmdc:CPMDCpmdSection.traceMaxCalls`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1662``
* ``cpmdc:CPMDCpmdSection.structureOptions`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1663``
* ``cpmdc:CPMDCpmdSection.structureSelection`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1664``
* ``cpmdc:CPMDCpmdSection.wannierNproc`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1665``
* ``cpmdc:CPMDCpmdSection.wannierRelocalizeInScf`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1666``
* ``cpmdc:CPMDCpmdSection.wannierRecomputeDipoleMatricesEvery`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1667``
* ``cpmdc:CPMDCpmdSection.wannierRelocalizeEvery`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1668``
* ``cpmdc:CPMDCpmdSection.paraUseMpiInPlace`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1669``
* ``cpmdc:CPMDCpmdSection.paraBuffSize`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1670``
* ``cpmdc:CPMDCpmdSection.paraStackBuffSize`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1671``
* ``cpmdc:CPMDCpmdSection.cpGroups`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1672``
* ``cpmdc:CPMDCpmdSection.fileFusionPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1673``
* ``cpmdc:CPMDCpmdSection.fileMergePayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1674``
* ``cpmdc:CPMDCpmdSection.fileSeparationPayload`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1675``
* ``cpmdc:CPMDCpmdSection.noReset`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1676``
* ``cpmdc:CPMDCpmdSection.lbfgsNtrust`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1677``
* ``cpmdc:CPMDCpmdSection.lbfgsNrestt`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1678``
* ``cpmdc:CPMDCpmdSection.lbfgsNtrstr`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1679``
* ``cpmdc:CPMDCpmdSection.lbfgsTrustr`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1680``
* ``cpmdc:CPMDCpmdSection.prfoMode`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1681``
* ``cpmdc:CPMDCpmdSection.prfoMdlock`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1682``
* ``cpmdc:CPMDCpmdSection.prfoTolenv`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1683``
* ``cpmdc:CPMDCpmdSection.prfoTrustp`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1684``
* ``cpmdc:CPMDCpmdSection.prfoOmin`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1685``
* ``cpmdc:CPMDCpmdSection.prfoNsvib`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1686``
* ``cpmdc:CPMDCpmdSection.prfoCoreAtoms`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1687``
* ``cpmdc:CPMDCpmdSection.prfoNsmaxp`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1688``
* ``cpmdc:CPMDCpmdSection.prfoProjectedHessian`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1689``
* ``cpmdc:CPMDCpmdSection.prfoDisplacement`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1690``
* ``cpmdc:CPMDCpmdSection.prfoHessianType`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1691``
* ``cpmdc:CPMDCpmdSection.lsd`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1692``
* ``cpmdc:CPMDCpmdSection.checkMemory`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1693``
* ``cpmdc:CPMDCpmdSection.trotterFactorizationOff`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1694``
* ``cpmdc:CPMDCpmdSection.damping`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1695``
* ``cpmdc:CPMDCpmdSection.dipoleDynamics`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1696``
* ``cpmdc:CPMDCpmdSection.realSpaceWfn`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1697``
* ``cpmdc:CPMDCpmdSection.noseIonsLocalT0`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1698``
* ``cpmdc:CPMDCpmdSection.noseIonsLocalThermostatCount`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1699``
* ``cpmdc:CPMDCpmdSection.noseIonsLocalThermostats`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1700``
* ``cpmdc:CPMDCpmdSection.noseIonsLocalRangeCount`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1701``
* ``cpmdc:CPMDCpmdSection.noseIonsLocalRanges`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1702``
* ``cpmdc:CPMDCpmdSection.noseIonsCafesGroupCount`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1703``
* ``cpmdc:CPMDCpmdSection.noseIonsCafesGroups`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1704``
* ``cpmdc:CPMDCpmdSection.trotterFactorization`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1705``
* ``cpmdc:CPMDDftSection.functional`` — ``Text``, default ``BLYP``; ``schema/Potentials.capnp:1709``
* ``cpmdc:CPMDDftSection.lsd`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1710``
* ``cpmdc:CPMDDftSection.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1711``
* ``cpmdc:CPMDDftSection.gcCutoff`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1712``
* ``cpmdc:CPMDDftSection.xcDriver`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1713``
* ``cpmdc:CPMDDftSection.libxc`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1714``
* ``cpmdc:CPMDDftSection.lrKernel`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1715``
* ``cpmdc:CPMDDftSection.refunct`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1716``
* ``cpmdc:CPMDDftSection.mtsHighFunc`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1717``
* ``cpmdc:CPMDDftSection.mtsLowFunc`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1718``
* ``cpmdc:CPMDDftSection.hfx`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1719``
* ``cpmdc:CPMDDftSection.hfxScreening`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1720``
* ``cpmdc:CPMDDftSection.hubbard`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1721``
* ``cpmdc:CPMDDftSection.alpha`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1722``
* ``cpmdc:CPMDDftSection.beta`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1723``
* ``cpmdc:CPMDDftSection.oldCode`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1724``
* ``cpmdc:CPMDDftSection.newCode`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1725``
* ``cpmdc:CPMDDftSection.correlation`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1726``
* ``cpmdc:CPMDDftSection.exchange`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1727``
* ``cpmdc:CPMDDftSection.becke88`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1728``
* ``cpmdc:CPMDDftSection.hubbardU`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1729``
* ``cpmdc:CPMDDftSection.hfxWfcCutoff`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1730``
* ``cpmdc:CPMDDftSection.hfxBlock`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1731``
* ``cpmdc:CPMDDftSection.hfxDistribution`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1732``
* ``cpmdc:CPMDAtomsPseudopotential.element`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1736``
* ``cpmdc:CPMDAtomsPseudopotential.path`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1737``
* ``cpmdc:CPMDAtomsPseudopotential.lmax`` — ``Int32``, default ``-1``; ``schema/Potentials.capnp:1738``
* ``cpmdc:CPMDAtomsPseudopotential.loc`` — ``Int32``, default ``-1``; ``schema/Potentials.capnp:1739``
* ``cpmdc:CPMDAtomsPseudopotential.skip`` — ``Int32``, default ``-1``; ``schema/Potentials.capnp:1740``
* ``cpmdc:CPMDAtomsPseudopotential.kleinmanBylander`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1741``
* ``cpmdc:CPMDAtomsPseudopotential.raggio`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1742``
* ``cpmdc:CPMDAtomsPseudopotential.nonlinearCore`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1743``
* ``cpmdc:CPMDAtomConstraint.kind`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1749``
* ``cpmdc:CPMDAtomConstraint.atoms`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1750``
* ``cpmdc:CPMDAtomConstraint.target`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1751``
* ``cpmdc:CPMDAtomConstraint.growth`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1752``
* ``cpmdc:CPMDIsotope.element`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1756``
* ``cpmdc:CPMDIsotope.mass`` — ``Float64``, default ``(no source default)``; ``schema/Potentials.capnp:1757``
* ``cpmdc:CPMDAtomVelocity.atom`` — ``Int32``, default ``(no source default)``; ``schema/Potentials.capnp:1761``
* ``cpmdc:CPMDAtomVelocity.velocity`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1762``
* ``cpmdc:CPMDDummyAtom.type`` — ``Int32``, default ``1``; ``schema/Potentials.capnp:1766``
* ``cpmdc:CPMDDummyAtom.atoms`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1767``
* ``cpmdc:CPMDDummyAtom.weights`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1768``
* ``cpmdc:CPMDAtomsSection.pseudopotentials`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1772``
* ``cpmdc:CPMDAtomsSection.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1775``
* ``cpmdc:CPMDAtomsSection.constraints`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1776``
* ``cpmdc:CPMDAtomsSection.isotopes`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1777``
* ``cpmdc:CPMDAtomsSection.velocities`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1778``
* ``cpmdc:CPMDAtomsSection.dummyAtoms`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1779``
* ``cpmdc:CPMDAtomsSection.changeBonds`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1780``
* ``cpmdc:CPMDAtomsSection.generate`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1781``
* ``cpmdc:CPMDVdwSection.empiricalCorrection`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1788``
* ``cpmdc:CPMDVdwSection.grimme`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1789``
* ``cpmdc:CPMDVdwSection.s6`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1790``
* ``cpmdc:CPMDVdwSection.vdwCutoff`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1791``
* ``cpmdc:CPMDVdwSection.vdwCell`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1792``
* ``cpmdc:CPMDVdwSection.radius`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1793``
* ``cpmdc:CPMDVdwSection.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1794``
* ``cpmdc:CPMDHubbardU.element`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1800``
* ``cpmdc:CPMDHubbardU.l`` — ``Int32``, default ``2``; ``schema/Potentials.capnp:1801``
* ``cpmdc:CPMDHubbardU.u`` — ``Float64``, default ``(no source default)``; ``schema/Potentials.capnp:1802``
* ``cpmdc:CPMDDirectiveSection.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1806``
* ``cpmdc:CPMDDirectiveSection.subsections`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1807``
* ``cpmdc:CPMDInputSection.generic`` — ``CPMDGenericSection``, default ``(no source default)``; ``schema/Potentials.capnp:1843``
* ``cpmdc:CPMDInputSection.system`` — ``CPMDSystemSection``, default ``(no source default)``; ``schema/Potentials.capnp:1844``
* ``cpmdc:CPMDInputSection.cpmd`` — ``CPMDCpmdSection``, default ``(no source default)``; ``schema/Potentials.capnp:1845``
* ``cpmdc:CPMDInputSection.dft`` — ``CPMDDftSection``, default ``(no source default)``; ``schema/Potentials.capnp:1846``
* ``cpmdc:CPMDInputSection.atoms`` — ``CPMDAtomsSection``, default ``(no source default)``; ``schema/Potentials.capnp:1847``
* ``cpmdc:CPMDInputSection.set`` — ``CPMDSetDirective``, default ``(no source default)``; ``schema/Potentials.capnp:1848``
* ``cpmdc:CPMDInputSection.raw`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1849``
* ``cpmdc:CPMDInputSection.atom`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1850``
* ``cpmdc:CPMDInputSection.basis`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1851``
* ``cpmdc:CPMDInputSection.clas`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1852``
* ``cpmdc:CPMDInputSection.eam`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1853``
* ``cpmdc:CPMDInputSection.exte`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1854``
* ``cpmdc:CPMDInputSection.hardness`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1855``
* ``cpmdc:CPMDInputSection.info`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1856``
* ``cpmdc:CPMDInputSection.linres`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1857``
* ``cpmdc:CPMDInputSection.molstates`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1858``
* ``cpmdc:CPMDInputSection.mts`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1859``
* ``cpmdc:CPMDInputSection.nlcc`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1860``
* ``cpmdc:CPMDInputSection.path`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1861``
* ``cpmdc:CPMDInputSection.pimd`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1862``
* ``cpmdc:CPMDInputSection.potential`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1863``
* ``cpmdc:CPMDInputSection.prop`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1864``
* ``cpmdc:CPMDInputSection.ptddft`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1865``
* ``cpmdc:CPMDInputSection.resp`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1866``
* ``cpmdc:CPMDInputSection.tddft`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1867``
* ``cpmdc:CPMDInputSection.vdw`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1868``
* ``cpmdc:CPMDInputSection.vectors`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1869``
* ``cpmdc:CPMDInputSection.wavefunction`` — ``CPMDDirectiveSection``, default ``(no source default)``; ``schema/Potentials.capnp:1870``
* ``cpmdc:CPMDInputSection.vdwParams`` — ``CPMDVdwSection``, default ``(no source default)``; ``schema/Potentials.capnp:1871``
* ``cpmdc:CPMDInputSection.propParams`` — ``CPMDPropSection``, default ``(no source default)``; ``schema/Potentials.capnp:1872``
* ``cpmdc:CPMDInputSection.linresParams`` — ``CPMDLinresSection``, default ``(no source default)``; ``schema/Potentials.capnp:1873``
* ``cpmdc:CPMDInputSection.pimdParams`` — ``CPMDPimdSection``, default ``(no source default)``; ``schema/Potentials.capnp:1874``
* ``cpmdc:CPMDInputSection.pathParams`` — ``CPMDPathSection``, default ``(no source default)``; ``schema/Potentials.capnp:1875``
* ``cpmdc:CPMDInputSection.tddftParams`` — ``CPMDTddftSection``, default ``(no source default)``; ``schema/Potentials.capnp:1876``
* ``cpmdc:CPMDInputSection.respParams`` — ``CPMDRespSection``, default ``(no source default)``; ``schema/Potentials.capnp:1877``
* ``cpmdc:CPMDInputSection.exteParams`` — ``CPMDExteSection``, default ``(no source default)``; ``schema/Potentials.capnp:1878``
* ``cpmdc:CPMDInputSection.vectorsParams`` — ``CPMDVectorsSection``, default ``(no source default)``; ``schema/Potentials.capnp:1879``
* ``cpmdc:CPMDParams.functional`` — ``Text``, default ``BLYP``; ``schema/Potentials.capnp:1889``
* ``cpmdc:CPMDParams.cutOffRy`` — ``Float64``, default ``70.0``; ``schema/Potentials.capnp:1890``
* ``cpmdc:CPMDParams.charge`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1891``
* ``cpmdc:CPMDParams.multiplicity`` — ``Int32``, default ``1``; ``schema/Potentials.capnp:1892``
* ``cpmdc:CPMDParams.task`` — ``Text``, default ``gradient``; ``schema/Potentials.capnp:1893``
* ``cpmdc:CPMDParams.title`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1894``
* ``cpmdc:CPMDParams.memoryMb`` — ``UInt32``, default ``0``; ``schema/Potentials.capnp:1895``
* ``cpmdc:CPMDParams.scratchDir`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1896``
* ``cpmdc:CPMDParams.permanentDir`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1897``
* ``cpmdc:CPMDParams.cpmdRoot`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1898``
* ``cpmdc:CPMDParams.enginePath`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1899``
* ``cpmdc:CPMDParams.inputBlocks`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1900``
* ``cpmdc:CPMDParams.inputSections`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1901``
* ``cpmdc:CommonMethodSpec.xcFunctionals`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1914``
* ``cpmdc:CommonMethodSpec.basisSet`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1915``
* ``cpmdc:CommonMethodSpec.planewaveCutoffEv`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1916``
* ``cpmdc:CommonMethodSpec.charge`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1917``
* ``cpmdc:CommonMethodSpec.spinMultiplicity`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1918``
* ``cpmdc:CommonMethodSpec.scfEnergyToleranceEv`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1919``
* ``cpmdc:CommonMethodSpec.scfMaxIterations`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1920``
* ``cpmdc:CommonMethodSpec.kMesh`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1921``
* ``cpmdc:CommonMethodSpec.smearing`` — ``Smearing``, default ``(no source default)``; ``schema/Potentials.capnp:1922``
* ``cpmdc:CommonMethodSpec.vanDerWaalsMethod`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1923``
* ``cpmdc:CommonMethodSpec.relativityMethod`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1924``
* ``cpmdc:CommonMethodSpec.vanDerWaalsS6`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1925``
* ``cpmdc:Smearing.kind`` — ``Kind``, default ``none``; ``schema/Potentials.capnp:1929``
* ``cpmdc:Smearing.widthEv`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1930``
* ``cpmdc:MetatomicParams.modelPath`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1946``
* ``cpmdc:MetatomicParams.device`` — ``Text``, default ``cpu``; ``schema/Potentials.capnp:1947``
* ``cpmdc:MetatomicParams.extensionsDirectory`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1948``
* ``cpmdc:MetatomicParams.checkConsistency`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1949``
* ``cpmdc:MetatomicParams.uncertaintyThreshold`` — ``Float64``, default ``-1.0``; ``schema/Potentials.capnp:1950``
* ``cpmdc:MetatomicParams.dtypeOverride`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1951``
* ``cpmdc:MetatomicParams.outputs`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1952``
* ``cpmdc:Output.name`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1959``
* ``cpmdc:Output.perAtom`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:1960``
* ``cpmdc:Output.explicitGradients`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1961``
* ``cpmdc:UmaParams.modelPath`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1971``
* ``cpmdc:UmaParams.taskName`` — ``Text``, default ``omol``; ``schema/Potentials.capnp:1972``
* ``cpmdc:UmaParams.device`` — ``Text``, default ``cpu``; ``schema/Potentials.capnp:1973``
* ``cpmdc:UmaParams.charge`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1974``
* ``cpmdc:UmaParams.spin`` — ``Int32``, default ``1``; ``schema/Potentials.capnp:1975``
* ``cpmdc:UmaParams.cutoff`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:1976``
* ``cpmdc:UmaParams.maxNeighbors`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:1977``
* ``cpmdc:LammpsParams.unitsStyle`` — ``Text``, default ``metal``; ``schema/Potentials.capnp:1989``
* ``cpmdc:LammpsParams.atomStyle`` — ``Text``, default ``atomic``; ``schema/Potentials.capnp:1990``
* ``cpmdc:LammpsParams.pairStyle`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1991``
* ``cpmdc:LammpsParams.pairCoeffs`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1992``
* ``cpmdc:LammpsParams.typeToAtomicNumber`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1993``
* ``cpmdc:LammpsParams.masses`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1994``
* ``cpmdc:LammpsParams.newtonPair`` — ``Bool``, default ``true``; ``schema/Potentials.capnp:1995``
* ``cpmdc:LammpsParams.boundary`` — ``Text``, default ``p p p``; ``schema/Potentials.capnp:1996``
* ``cpmdc:LammpsParams.extraSetup`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:1997``
* ``cpmdc:LammpsParams.suffix`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:1998``
* ``cpmdc:NWChemDplotStanza.title`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:2004``
* ``cpmdc:NWChemDplotStanza.gaussianCube`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:2005``
* ``cpmdc:NWChemDplotStanza.civecs`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:2006``
* ``cpmdc:NWChemDplotStanza.limitXyz`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2007``
* ``cpmdc:NWChemDplotStanza.spin`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:2008``
* ``cpmdc:NWChemDplotStanza.orbitals`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2009``
* ``cpmdc:NWChemDplotStanza.output`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:2010``
* ``cpmdc:NWChemDplotStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2011``
* ``cpmdc:NWChemEspStanza.recalculate`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2017``
* ``cpmdc:NWChemEspStanza.rangeFactor`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2018``
* ``cpmdc:NWChemEspStanza.probe`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2019``
* ``cpmdc:NWChemEspStanza.spacing`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2020``
* ``cpmdc:NWChemEspStanza.restrain`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2021``
* ``cpmdc:NWChemEspStanza.restrainHfree`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2022``
* ``cpmdc:NWChemEspStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2023``
* ``cpmdc:CPMDPropSection.dipoleMoment`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2031``
* ``cpmdc:CPMDPropSection.localize`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2032``
* ``cpmdc:CPMDPropSection.polarizability`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2033``
* ``cpmdc:CPMDPropSection.cubecenter`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2034``
* ``cpmdc:CPMDPropSection.cubefileDensity`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2035``
* ``cpmdc:CPMDPropSection.cubefileOrbitals`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2036``
* ``cpmdc:CPMDPropSection.chargesGaussian`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2037``
* ``cpmdc:CPMDPropSection.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2038``
* ``cpmdc:CPMDLinresSection.convergence`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2044``
* ``cpmdc:CPMDLinresSection.maxSteps`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2045``
* ``cpmdc:CPMDLinresSection.hthrs`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2046``
* ``cpmdc:CPMDLinresSection.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2047``
* ``cpmdc:CPMDPimdSection.replicas`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2053``
* ``cpmdc:CPMDPimdSection.facmass`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2054``
* ``cpmdc:CPMDPimdSection.centroidDynamics`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2055``
* ``cpmdc:CPMDPimdSection.normalModes`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2056``
* ``cpmdc:CPMDPimdSection.staging`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2057``
* ``cpmdc:CPMDPimdSection.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2058``
* ``cpmdc:CPMDPathSection.replicaNumber`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2064``
* ``cpmdc:CPMDPathSection.nebSpring`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2065``
* ``cpmdc:CPMDPathSection.factor`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2066``
* ``cpmdc:CPMDPathSection.nloop`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2067``
* ``cpmdc:CPMDPathSection.alpha`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2068``
* ``cpmdc:CPMDPathSection.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2069``
* ``cpmdc:CPMDTddftSection.states`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2075``
* ``cpmdc:CPMDTddftSection.tammDancoff`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2076``
* ``cpmdc:CPMDTddftSection.diagonalizer`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:2077``
* ``cpmdc:CPMDTddftSection.convergence`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2078``
* ``cpmdc:CPMDTddftSection.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2079``
* ``cpmdc:NWChemQmdStanza.nstepNucl`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2085``
* ``cpmdc:NWChemQmdStanza.dtNucl`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2086``
* ``cpmdc:NWChemQmdStanza.targTemp`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2087``
* ``cpmdc:NWChemQmdStanza.thermostat`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:2088``
* ``cpmdc:NWChemQmdStanza.thermostatParameter`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2089``
* ``cpmdc:NWChemQmdStanza.randomSeed`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2090``
* ``cpmdc:NWChemQmdStanza.comStep`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2091``
* ``cpmdc:NWChemQmdStanza.printXyz`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2092``
* ``cpmdc:NWChemQmdStanza.linear`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2093``
* ``cpmdc:NWChemQmdStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2094``
* ``cpmdc:NWChemRamanStanza.normalMode`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2100``
* ``cpmdc:NWChemRamanStanza.dynamic`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2101``
* ``cpmdc:NWChemRamanStanza.low`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2102``
* ``cpmdc:NWChemRamanStanza.high`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2103``
* ``cpmdc:NWChemRamanStanza.steps`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2104``
* ``cpmdc:NWChemRamanStanza.fieldFrequency`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2105``
* ``cpmdc:NWChemRamanStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2106``
* ``cpmdc:NWChemFonStanza.partial`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2112``
* ``cpmdc:NWChemFonStanza.electrons`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2113``
* ``cpmdc:NWChemFonStanza.filled`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2114``
* ``cpmdc:NWChemFonStanza.alphaOnly`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2115``
* ``cpmdc:NWChemFonStanza.temperature`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2116``
* ``cpmdc:NWChemFonStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2117``
* ``cpmdc:CPMDRespSection.restraintStrength`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2125``
* ``cpmdc:CPMDRespSection.hyperbolic`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2126``
* ``cpmdc:CPMDRespSection.tightness`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2127``
* ``cpmdc:CPMDRespSection.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2128``
* ``cpmdc:CPMDExteSection.externalField`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2134``
* ``cpmdc:CPMDExteSection.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2135``
* ``cpmdc:CPMDVectorsSection.newOrtho`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2141``
* ``cpmdc:CPMDVectorsSection.overlap`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2142``
* ``cpmdc:CPMDVectorsSection.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2143``
* ``cpmdc:NWChemNebStanza.nbeads`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2149``
* ``cpmdc:NWChemNebStanza.kbeads`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2150``
* ``cpmdc:NWChemNebStanza.maxiter`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2151``
* ``cpmdc:NWChemNebStanza.stepsize`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2152``
* ``cpmdc:NWChemNebStanza.trust`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2153``
* ``cpmdc:NWChemNebStanza.nhist`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2154``
* ``cpmdc:NWChemNebStanza.algorithm`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2155``
* ``cpmdc:NWChemNebStanza.reset`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2156``
* ``cpmdc:NWChemNebStanza.convergence`` — ``Convergence``, default ``unspecified``; ``schema/Potentials.capnp:2157``
* ``cpmdc:NWChemNebStanza.gmax`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2158``
* ``cpmdc:NWChemNebStanza.grms`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2159``
* ``cpmdc:NWChemNebStanza.xmax`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2160``
* ``cpmdc:NWChemNebStanza.xrms`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2161``
* ``cpmdc:NWChemNebStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2162``
* ``cpmdc:NWChemMepgsStanza.stride`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2175``
* ``cpmdc:NWChemMepgsStanza.maxmep`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2176``
* ``cpmdc:NWChemMepgsStanza.maxiter`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2177``
* ``cpmdc:NWChemMepgsStanza.inhess`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2178``
* ``cpmdc:NWChemMepgsStanza.evib`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2179``
* ``cpmdc:NWChemMepgsStanza.direction`` — ``Direction``, default ``both``; ``schema/Potentials.capnp:2180``
* ``cpmdc:NWChemMepgsStanza.opttol`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2181``
* ``cpmdc:NWChemMepgsStanza.eprec`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2182``
* ``cpmdc:NWChemMepgsStanza.xyz`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2183``
* ``cpmdc:NWChemMepgsStanza.mswg`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2184``
* ``cpmdc:NWChemMepgsStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2185``
* ``cpmdc:NWChemTroptStanza.opttol`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2197``
* ``cpmdc:NWChemTroptStanza.eprec`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2198``
* ``cpmdc:NWChemTroptStanza.trust`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2199``
* ``cpmdc:NWChemTroptStanza.maxiter`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2200``
* ``cpmdc:NWChemTroptStanza.inhess`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2201``
* ``cpmdc:NWChemTroptStanza.lbfgs`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2202``
* ``cpmdc:NWChemTroptStanza.mh`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2203``
* ``cpmdc:NWChemTroptStanza.linopt`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2204``
* ``cpmdc:NWChemTroptStanza.qstep`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:2205``
* ``cpmdc:NWChemTroptStanza.moddir`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2206``
* ``cpmdc:NWChemTroptStanza.modsad`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2207``
* ``cpmdc:NWChemTroptStanza.redoautoz`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2208``
* ``cpmdc:NWChemTroptStanza.convergence`` — ``Convergence``, default ``unspecified``; ``schema/Potentials.capnp:2209``
* ``cpmdc:NWChemTroptStanza.gmax`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2210``
* ``cpmdc:NWChemTroptStanza.grms`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2211``
* ``cpmdc:NWChemTroptStanza.xmax`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2212``
* ``cpmdc:NWChemTroptStanza.xrms`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2213``
* ``cpmdc:NWChemTroptStanza.xyz`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2214``
* ``cpmdc:NWChemTroptStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2215``
* ``cpmdc:NWChemMetadynamicsCv.kind`` — ``Kind``, default ``bond``; ``schema/Potentials.capnp:2229``
* ``cpmdc:NWChemMetadynamicsCv.atoms`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2230``
* ``cpmdc:NWChemMetadynamicsCv.index1`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2231``
* ``cpmdc:NWChemMetadynamicsCv.index2`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2232``
* ``cpmdc:NWChemMetadynamicsCv.weight`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2233``
* ``cpmdc:NWChemMetadynamicsCv.sigma`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2234``
* ``cpmdc:NWChemMetadynamicsCv.rangeLow`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2235``
* ``cpmdc:NWChemMetadynamicsCv.rangeHigh`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2236``
* ``cpmdc:NWChemMetadynamicsCv.nrange`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2237``
* ``cpmdc:NWChemMetadynamicsCv.n`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2238``
* ``cpmdc:NWChemMetadynamicsCv.m`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2239``
* ``cpmdc:NWChemMetadynamicsCv.r0`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2240``
* ``cpmdc:NWChemMetadynamicsStanza.cvs`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2252``
* ``cpmdc:NWChemMetadynamicsStanza.tempered`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2253``
* ``cpmdc:NWChemMetadynamicsStanza.boundary`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2254``
* ``cpmdc:NWChemMetadynamicsStanza.potentialFilename`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:2255``
* ``cpmdc:NWChemMetadynamicsStanza.printShift`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2256``
* ``cpmdc:NWChemMetadynamicsStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2257``
* ``cpmdc:NWChemCellOptimizeStanza.cellName`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:2264``
* ``cpmdc:NWChemCellOptimizeStanza.strategy`` — ``Strategy``, default ``unspecified``; ``schema/Potentials.capnp:2265``
* ``cpmdc:NWChemCellOptimizeStanza.optimizeLattice`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2266``
* ``cpmdc:NWChemCellOptimizeStanza.optimizeLatticeVectors`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2267``
* ``cpmdc:NWChemCellOptimizeStanza.latticeTolerances`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2268``
* ``cpmdc:NWChemCellOptimizeStanza.cycles`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2269``
* ``cpmdc:NWChemCellOptimizeStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2270``
* ``cpmdc:NWChemRismStanza.permittivity`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2282``
* ``cpmdc:NWChemRismStanza.tau`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2283``
* ``cpmdc:NWChemRismStanza.temp`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2284``
* ``cpmdc:NWChemRismStanza.tol`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2285``
* ``cpmdc:NWChemRismStanza.lambda`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2286``
* ``cpmdc:NWChemRismStanza.diis`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2287``
* ``cpmdc:NWChemRismStanza.ngrid`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2288``
* ``cpmdc:NWChemRismStanza.closure`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:2289``
* ``cpmdc:NWChemRismStanza.vdwRule`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:2290``
* ``cpmdc:NWChemRismStanza.vdwParameters`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:2291``
* ``cpmdc:NWChemRismStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2292``
* ``cpmdc:NWChemDimQmStanza.frequency`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2298``
* ``cpmdc:NWChemDimQmStanza.algorithm`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2299``
* ``cpmdc:NWChemDimQmStanza.tolerance`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2300``
* ``cpmdc:NWChemDimQmStanza.efield`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2301``
* ``cpmdc:NWChemDimQmStanza.localfield`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2302``
* ``cpmdc:NWChemDimQmStanza.noresp`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2303``
* ``cpmdc:NWChemDimQmStanza.screen`` — ``Screen``, default ``unspecified``; ``schema/Potentials.capnp:2304``
* ``cpmdc:NWChemDimQmStanza.screenFactor`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2305``
* ``cpmdc:NWChemDimQmStanza.noseed`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2306``
* ``cpmdc:NWChemDimQmStanza.debug`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2307``
* ``cpmdc:NWChemDimQmStanza.printAtomicDipoles`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2308``
* ``cpmdc:NWChemDimQmStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2309``
* ``cpmdc:NWChemGwStanza.evgw`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2321``
* ``cpmdc:NWChemGwStanza.evgw0`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2322``
* ``cpmdc:NWChemGwStanza.eviter`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2323``
* ``cpmdc:NWChemGwStanza.method`` — ``Method``, default ``unspecified``; ``schema/Potentials.capnp:2324``
* ``cpmdc:NWChemGwStanza.ngl`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2325``
* ``cpmdc:NWChemGwStanza.solver`` — ``Solver``, default ``unspecified``; ``schema/Potentials.capnp:2326``
* ``cpmdc:NWChemGwStanza.qpiter`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2327``
* ``cpmdc:NWChemGwStanza.thresholdEv`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2328``
* ``cpmdc:NWChemGwStanza.occAlpha`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2329``
* ``cpmdc:NWChemGwStanza.virAlpha`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2330``
* ``cpmdc:NWChemGwStanza.occBeta`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2331``
* ``cpmdc:NWChemGwStanza.virBeta`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2332``
* ``cpmdc:NWChemGwStanza.first`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2333``
* ``cpmdc:NWChemGwStanza.ncap`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2334``
* ``cpmdc:NWChemGwStanza.eta`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2335``
* ``cpmdc:NWChemGwStanza.rpa`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2336``
* ``cpmdc:NWChemGwStanza.diag`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2337``
* ``cpmdc:NWChemGwStanza.core`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2338``
* ``cpmdc:NWChemGwStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2339``
* ``cpmdc:NWChemEtransStanza.tol2e`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2355``
* ``cpmdc:NWChemEtransStanza.fock`` — ``NWChemToggle``, default ``unspecified``; ``schema/Potentials.capnp:2356``
* ``cpmdc:NWChemEtransStanza.fmo`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2357``
* ``cpmdc:NWChemEtransStanza.vectorsReactant`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:2358``
* ``cpmdc:NWChemEtransStanza.vectorsProduct`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:2359``
* ``cpmdc:NWChemEtransStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2360``
* ``cpmdc:NWChemStringStanza.nbeads`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2366``
* ``cpmdc:NWChemStringStanza.maxiter`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2367``
* ``cpmdc:NWChemStringStanza.stepsize`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2368``
* ``cpmdc:NWChemStringStanza.tol`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2369``
* ``cpmdc:NWChemStringStanza.interpol`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2370``
* ``cpmdc:NWChemStringStanza.nhist`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2371``
* ``cpmdc:NWChemStringStanza.algorithm`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2372``
* ``cpmdc:NWChemStringStanza.printShift`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2373``
* ``cpmdc:NWChemStringStanza.freeze1`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2374``
* ``cpmdc:NWChemStringStanza.freezen`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2375``
* ``cpmdc:NWChemStringStanza.hasmiddle`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2376``
* ``cpmdc:NWChemStringStanza.impose`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2377``
* ``cpmdc:NWChemStringStanza.reset`` — ``Bool``, default ``false``; ``schema/Potentials.capnp:2378``
* ``cpmdc:NWChemStringStanza.xyzPath`` — ``Text``, default ``(no source default)``; ``schema/Potentials.capnp:2379``
* ``cpmdc:NWChemStringStanza.mode`` — ``Mode``, default ``unspecified``; ``schema/Potentials.capnp:2380``
* ``cpmdc:NWChemStringStanza.directives`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2381``
* ``cpmdc:NWChemCdftConstraint.firstAtomStart`` — ``Int32``, default ``(no source default)``; ``schema/Potentials.capnp:2392``
* ``cpmdc:NWChemCdftConstraint.firstAtomEnd`` — ``Int32``, default ``(no source default)``; ``schema/Potentials.capnp:2393``
* ``cpmdc:NWChemCdftConstraint.secondAtomStart`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2394``
* ``cpmdc:NWChemCdftConstraint.secondAtomEnd`` — ``Int32``, default ``0``; ``schema/Potentials.capnp:2395``
* ``cpmdc:NWChemCdftConstraint.kind`` — ``Kind``, default ``charge``; ``schema/Potentials.capnp:2396``
* ``cpmdc:NWChemCdftConstraint.value`` — ``Float64``, default ``(no source default)``; ``schema/Potentials.capnp:2397``
* ``cpmdc:NWChemCdftSpec.constraints`` — ``List``, default ``(no source default)``; ``schema/Potentials.capnp:2407``
* ``cpmdc:NWChemCdftSpec.convergence`` — ``Float64``, default ``0.0``; ``schema/Potentials.capnp:2408``
* ``cpmdc:NWChemCdftSpec.population`` — ``Population``, default ``unspecified``; ``schema/Potentials.capnp:2409``
* ``cpmdc:PotentialConfig.none`` — ``Void``, default ``(no source default)``; ``schema/Potentials.capnp:2431``
* ``cpmdc:PotentialConfig.nwchem`` — ``NWChemParams``, default ``(no source default)``; ``schema/Potentials.capnp:2432``
* ``cpmdc:PotentialConfig.cpmd`` — ``CPMDParams``, default ``(no source default)``; ``schema/Potentials.capnp:2433``
* ``cpmdc:PotentialConfig.metatomic`` — ``MetatomicParams``, default ``(no source default)``; ``schema/Potentials.capnp:2434``
* ``cpmdc:PotentialConfig.lammps`` — ``LammpsParams``, default ``(no source default)``; ``schema/Potentials.capnp:2435``
* ``cpmdc:PotentialConfig.common`` — ``CommonMethodSpec``, default ``(no source default)``; ``schema/Potentials.capnp:2440``
