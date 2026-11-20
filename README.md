# rgconf

`rgconf` is the parameter-resolution and provenance library for the
OmniPotentRPC stack. It turns layered Cap'n Proto configuration defaults,
files, environment variables, command-line options, and runtime edits into one
validated configuration. Every resolved value retains its origin, and
configuration snapshots can be recorded in an RFC 9162 transparency log.

The project provides a Rust library, the `rgconf` command-line program, a C
ABI, and Python bindings.

## Install

Install the command-line program from crates.io:

```console
cargo install rgconf-cli
```

Add the Rust library to a project:

```console
cargo add rgconf
```

Install the Python package from PyPI:

```console
python -m pip install rgconf
```

## Quick start

Create `job.ini`:

```ini
[Optimizer]
converged_force = 0.03
```

Resolve it using the legacy EON profile:

```console
rgconf resolve --profile eonLegacyIni --ini job.ini
```

The equivalent Python call is:

```python
from rgconf import resolve_ini

resolved = resolve_ini(
    "eonLegacyIni",
    """[Optimizer]
converged_force = 0.03
""",
    "job.ini",
)
print(resolved)
```

## Documentation

The [user guide](https://omnipotentrpc.github.io/rgconf/) covers resolution,
schema extension, deprecation, parameters, precedence, and upstream
compatibility. Rust API documentation is available on
[docs.rs](https://docs.rs/rgconf). See the
[changelog](https://github.com/OmniPotentRPC/rgconf/blob/main/CHANGELOG.md) for
release notes.

The public C interface is declared in [`include/rgconf.h`](include/rgconf.h).

## License

MIT. See [LICENSE](LICENSE).
