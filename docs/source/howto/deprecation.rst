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


Defaults must agree
===================


The Lennard Jones and Morse ``switchWidth`` literals are 0.0, matching
``LJConfig.switch_width``, ``LJClusterConfig.switch_width``,
``MorseConfig.switch_width``, and the long-range dispersion switch in rgpot.
The importer keeps the source value and its exact line.
