Upstream gaps and design notes
==============================



rfc9162 v0.11.0
===============


These calls return ``Error::UpstreamPatch`` and are not implemented here.

* ``rfc9162::canonicalize_capnp`` should pack a Cap'n Proto message before ``leaf_hash``.
* ``rfc9162_log::Store::put_blob_and_append`` should store an entry longer than 65535 bytes as a blob and append a short pointer leaf. ``Sidecar::put`` writes a file and does not append that leaf.
* ``rfc9162_tiles::Writer::append_shared`` should append without holding ``treeState.lock`` from open until drop.
* C and Python bindings for the log belong in the ``rfc9162`` repository.


Notes on the design text
========================


The schema ``ConfigLayer`` ordinals stop at ``liveRuntimeEdit`` (@7). ``derived`` and the C ABI fallback are ``ValueStatus`` values, not layers. A map section that calls the fallback a layer is the part that does not match the schema.

``OciNebRefinePatch`` still carries ``hasMaxSteps``, ``hasMaxPhases``, and ``hasMaxInnerIterations`` beside its counts. That is the presence shape the rest of the schema left behind. The ordinals are unchanged.

The zoom trigger literal in the schema is 0.5 and the ``ewKMin`` literal is 0.972. The resolver text says 10 times the band force and one tenth of the spring constant. The formulas win when the user did not set the field.

The gpridge companion did not declare ``ci_stability_count`` or ``use_prune``. Those deprecated aliases are on ``GpridgeDeprecated`` so an old job file still loads.

``RgConf`` had no list for a namespaced key that no companion schema declared. ``extensions`` (@39) is that list.
