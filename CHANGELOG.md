# Changelog

<!-- towncrier release notes start -->

## [0.1.0] - 2026-11-20

### Added

- The Python package builds with maturin, so `rgconf._core` is installed beside the Python sources and the tests exercise the native extension.
- The eOn `lineb` job accepts a fixed tunneling energy and mass-weighted force tolerances in the `[Lineb]` section. Optional controls select a straight initial path, a constant endpoint restraint, separate endpoint and interior convergence tests, and projected velocity Verlet.
- The first rgconf resolver, Cap'n Proto parameter schema, C ABI, and Python bindings.
- The parameter map lists the rgmin Riemannian L-BFGS fields and the rgsaddle ARTn and growing string fields. Each names the rgconf schema field and the eOn option it corresponds to, or marks that there is none.

### Changed

- Prepare the Rust crates, Python package, bundled schemas, README, and documentation site for the 0.1.0 release.
- The parameter map records the upstream commit each citation was checked against in `generated/sources.tsv`. `tools/fetch_sources.py` fetches those commits, and CI regenerates the map from them and compares it with the committed copy.

### Fixed

- Build the Python extension on CPython 3.14.
- Nested struct and enum declarations are included in the parameter catalog, so each leaf parameter has a spec. A type name shared by a struct and an enum resolves to the declaration named by the reference scope or path. The cautious slope annotation states that rgmin defaults to 1e-4 while the schema and eOn default to 1e-6.
- Resolve dotted schema references in their enclosing scope before considering import-alias tails, and preserve fields whose names match declaration keywords.
- Source line citations in the parameter map point at the declarations they name, and a test resolves every citation against the upstream trees at their pinned commits.
