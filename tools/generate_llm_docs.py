# Copyright 2023--present rgconf developers
"""Write llms.txt, llms-full.txt, and AGENTS.md from docs/orgmode."""

import csv
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ORG = ROOT / "docs" / "orgmode"
PAGES = [
    "home.org",
    "tutorials/resolve.org",
    "howto/extend-schema.org",
    "howto/deprecation.org",
    "reference/parameters.org",
    "explanation/precedence.org",
    "explanation/upstream.org",
]
PARAMETER_MAP = ROOT / "generated" / "parameter_map.tsv"
PARAMETER_REFERENCE = ROOT / "docs" / "source" / "reference" / "source-parameter-map.rst"


def body(path: Path) -> str:
    lines = []
    for line in path.read_text().splitlines():
        if line.startswith("#+"):
            continue
        lines.append(line)
    return "\n".join(lines).strip() + "\n"


def parameter_reference() -> str:
    lines = [
        "Imported source parameter map",
        "=============================",
        "",
        "Generated from ``generated/parameter_map.tsv``. Each entry records the",
        "source declaration, type, default, and source section. A blank default",
        "means the source declaration has no default. Units remain those stated",
        "by the named source API when the source map has no unit annotation.",
        "Entries that carry a mapping name the rgconf schema field (``unmapped``",
        "when rgconf has none, ``abi`` for ABI bookkeeping) and the eOn option",
        "(``none`` when eOn has no counterpart).",
        "",
    ]
    with PARAMETER_MAP.open(newline="") as stream:
        for row in csv.DictReader(stream, delimiter="\t"):
            default = row["default"] or "(no source default)"
            section = f"; section {row['section']}" if row["section"] else ""
            mapping = ""
            if row["rgconf"]:
                mapping = f"; rgconf ``{row['rgconf']}``; eOn ``{row['eon']}``. {row['note']}"
            lines.append(
                f"* ``{row['repo']}:{row['struct']}.{row['field']}`` — "
                f"``{row['type']}``, default ``{default}``; "
                f"``{row['relpath']}:{row['line']}``{section}{mapping}"
            )
    return "\n".join(lines) + "\n"


def main() -> None:
    parts = []
    index = ["# rgconf", "", "Generated from docs/orgmode. Do not edit by hand.", ""]
    for rel in PAGES:
        path = ORG / rel
        text = body(path)
        title = text.splitlines()[0].lstrip("* ").strip()
        index.append(f"- docs/orgmode/{rel}")
        parts.append(f"# {rel}\n\n{text}")
    source_parameters = parameter_reference()
    PARAMETER_REFERENCE.write_text(source_parameters)
    index.append("- docs/source/reference/source-parameter-map.rst")
    parts.append(f"# reference/source-parameter-map.rst\n\n{source_parameters}")
    (ROOT / "docs" / "llms.txt").write_text("\n".join(index) + "\n")
    (ROOT / "docs" / "llms-full.txt").write_text("\n".join(parts))
    agents = """# Agents

Generated from docs/orgmode by tools/generate_llm_docs.py.

Build with cargo. The workspace crate is `rgconf`. The C ABI is `rgconf-c-abi` and the header is `include/rgconf.h`. Python lives in `python/` (PyO3, module `_core`) and `bind/ext.cpp` (nanobind).

```
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
emacs --batch -l docs/export.el
python3 tools/generate_llm_docs.py
```

Public config structs are `#[non_exhaustive]` with `Default` and `with_*` setters. Provenance leaves go through the `rfc9162` crate. Production code does not call `unwrap_or` or `clamp`, and it does not introduce a bare `1e-N` literal. New files carry `Copyright 2023--present rgconf developers`.

"""
    agents += "\n".join(parts)
    (ROOT / "AGENTS.md").write_text(agents)
    print("wrote docs/llms.txt docs/llms-full.txt AGENTS.md")


if __name__ == "__main__":
    main()
