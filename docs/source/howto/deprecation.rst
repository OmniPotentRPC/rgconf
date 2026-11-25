Deprecation plan
================


``eon-schema`` and the hand written rgpot parameter blocks are inputs to rgconf. They stop being a second source of truth.


eon-schema
==========


The INI sections ``Main``, ``Optimizer``, ``Refine``, ``Nudged Elastic Band``, and ``Saddle Search`` keep working through aliases on the Cap'n Proto fields. A removed key is a deprecated alias: the resolver still loads it and reports ``deprecatedAlias``, as ``ci_stability_count`` and ``use_prune`` already do on the gpridge companion.

The removal order is:

* Ship rgconf in the clients that still read those sections.
* Keep the aliases for one release while jobs print the canonical key beside the old one.
* Delete the alias from the schema only after that release, in a schema file that bumps nothing underneath an existing ordinal.

``eon-schema`` itself is not imported by this crate. The alias table is the migration.


rgpot schema
============


``potential.capnp`` is the evolved rgpot parameter schema. The union arm is the backend. There is no ``kind`` field beside it. New backends are new arms at the next free ordinal, not a parallel struct in the rgpot tree.

Fields that exist only as comments in a C++ header stay in the generated parameter map until the schema field that replaces them is in a release. The map records the header line. It is not a second schema.

``potential.capnp`` is the only copy of the potentials schema (file id ``0xbd1f89fa17369103``). rgpot vendors it byte for byte, with ``common.capnp``, instead of carrying the ``potentials-schema`` file and its local diff series. It holds every member those two carried, at their ordinals: the ``composite`` (18), ``d2`` (19), ``d2cpmd`` (20) and ``mullerBrown`` (21) arms, ``ForceInput.hasOrbitalKey`` (10) and ``orbitalKey`` (11), ``D4Params.periodic``, and the ``hasCharge`` flags of ``CommonMethodSpec``, ``CPMDParams`` and ``CPMDSystemSection``.

rgconf 0.1.0 had used some of those ordinals for its own members, so they moved:

| Member                          | 0.1.0 | Now |
|---------------------------------+-------+-----|
| ``PotentialConfig.gpr``           |    18 |  27 |
| ``PotentialConfig.potlib``        |    19 |  28 |
| ``PotentialConfig.external``      |    20 |  29 |
| ``PotentialConfig.plugin``        |    21 |  30 |
| ``ForceInput.hessianDisplacement`` |   10 |  12 |
| ``ForceInput.wantStress``         |    11 |  13 |
| ``ForceInput.wantHessianVector``  |    12 |  14 |

Keys, INI and TOML aliases and the resolved ``potential.arm`` value are names, so a resolution reads the same before and after. rgconf 0.1.0 neither encoded nor decoded a Cap'n Proto message, so it wrote no binary with the old ordinals. A binary message that another tool encoded against the 0.1.0 file carries those arms or fields at the old ordinals and must be encoded again from its text form.

The ``orca`` arm (``PotentialConfig`` 31, ``PotentialSpec`` 7) with ``ORCAParams`` is the configuration of the ``orcac`` engine shim.


Defaults must agree
===================


The Lennard Jones and Morse ``switchWidth`` literals are 0.0, matching
``LJConfig.switch_width``, ``LJClusterConfig.switch_width``,
``MorseConfig.switch_width``, and the long-range dispersion switch in rgpot.
The importer keeps the source value and its exact line.
