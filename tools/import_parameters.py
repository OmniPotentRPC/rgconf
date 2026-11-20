# Copyright 2023--present rgconf developers
"""Read parameter fields from the sibling stacks and write a source map.

The map records only fields the parser saw, with the file and line they came
from. A brace or equals initializer is the default. A later comment that names
the field and a number is recorded as that comment's line, which is how rgmin
publishes Control::default.
"""

from __future__ import annotations

import argparse
import csv
import re
import subprocess
from pathlib import Path

from fetch_sources import PINS, read_pins

ROOT = Path(__file__).resolve().parents[1]
ANNOTATIONS = Path(__file__).resolve().parent / "parameter_annotations.tsv"
HEADER = ["repo", "relpath", "line", "struct", "field", "type", "default", "section", "rgconf", "eon", "note"]

FIELD_INIT = re.compile(
    r"""
    ^\s*
    (?:static\s+|const\s+|constexpr\s+|mutable\s+)*
    (?P<type>(?:[\w:<>]+\s+)*[\w:<>*]+)\s+
    (?P<name>[A-Za-z_][A-Za-z0-9_]*)
    \s*(?:\{(?P<brace>[^{}]*)\}|\=\s*(?P<eq>[^;]+))?
    \s*;
    """,
    re.VERBOSE,
)
STRUCT = re.compile(r"\b(?:typedef\s+)?struct(?:\s+([A-Za-z_][A-Za-z0-9_]*))?")
CLASS = re.compile(r"\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)")
SECTION = re.compile(r"^\s*//\s*\[([^\]]+)\]")
COMMENT_DEFAULT = re.compile(
    r"\b([A-Za-z_][A-Za-z0-9_]*)\s+(-?(?:\d+\.\d+|\d+)(?:e[+-]?\d+)?)\b"
)
CAPNP_FIELD = re.compile(
    r"""
    ^\s*
    (?P<name>[A-Za-z_][A-Za-z0-9_]*)
    \s*@\d+\s*:\s*
    (?P<type>[A-Za-z0-9_.:]+)
    (?:\s*=\s*(?P<default>[^;$]+))?
    """,
    re.VERBOSE,
)

SKIP_NAMES = {"if", "for", "while", "switch", "return", "sizeof"}


def iter_sources(root: Path, repo: str) -> list[Path]:
    patterns = {
        "eon": ["include/eon/ParametersOptions.h"],
        "rgmin": ["include/rgmin.h"],
        "rgsaddle": ["include/rgsaddle.h"],
        "rgpot": [
            "CppCore/rgpot/LennardJones/LJPot.hpp",
            "CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp",
        ],
        "cpmdc": ["include/cpmdc.h", "schema/Potentials.capnp"],
    }
    found = []
    for rel in patterns[repo]:
        path = root / rel
        if path.is_file():
            found.append(path)
    return found


def parse_cxx(text: str, repo: str, rel: str) -> list[tuple]:
    rows = []
    stack: list[tuple[str, int]] = []
    section = ""
    depth = 0
    lines = text.splitlines()
    for index, line in enumerate(lines, 1):
        section_match = SECTION.search(line)
        if section_match:
            section = section_match.group(1)
        struct = stack[-1][0] if stack else ""
        if struct:
            match = FIELD_INIT.match(line)
            if match and match.group("name") not in SKIP_NAMES:
                default = match.group("brace")
                if default is None:
                    default = match.group("eq")
                default = "" if default is None else default.strip()
                rows.append(
                    (repo, rel, str(index), struct, match.group("name"), match.group("type").strip(), default, section)
                )
        opened = "struct " in line or line.strip().startswith("typedef struct")
        depth += line.count("{") - line.count("}")
        if opened and "{" in line:
            match = CLASS.search(line) or STRUCT.search(line)
            name = match.group(1) if match and match.group(1) else ""
            if not name:
                name = f"anonymous{index}"
            stack.append((name, depth))
        close = re.match(r"\s*\}\s*([A-Za-z_][A-Za-z0-9_]*)\s*;", line)
        if close and stack and stack[-1][0].startswith("anonymous"):
            old = stack[-1][0]
            new = close.group(1)
            rows = [
                (row[0], row[1], row[2], new if row[3] == old else row[3], *row[4:])
                for row in rows
            ]
            stack[-1] = (new, stack[-1][1])
        while stack and depth < stack[-1][1]:
            stack.pop()
    rows.extend(comment_defaults(lines, repo, rel, rows))
    return rows


def comment_defaults(lines: list[str], repo: str, rel: str, existing: list[tuple]) -> list[tuple]:
    fields = {(row[3], row[4]): row for row in existing}
    extra = []
    for index, line in enumerate(lines, 1):
        if "Control::default" not in line:
            continue
        struct = "rgmin_control_t"
        for offset, extra_line in enumerate(lines[index - 1 : index + 3]):
            for name, value in COMMENT_DEFAULT.findall(extra_line):
                if (struct, name) not in fields:
                    continue
                if any(row[3] == struct and row[4] == name for row in extra):
                    continue
                extra.append((repo, rel, str(index + offset), struct, name, "comment", value, "control"))
    return extra


def parse_capnp(text: str, repo: str, rel: str) -> list[tuple]:
    rows = []
    struct = ""
    for index, line in enumerate(text.splitlines(), 1):
        stripped = line.strip()
        if stripped.startswith("struct "):
            struct = stripped.split()[1]
            continue
        if not struct or stripped.startswith("#") or stripped.startswith("enum "):
            continue
        match = CAPNP_FIELD.match(stripped)
        if not match:
            continue
        default = match.group("default") or ""
        rows.append(
            (repo, rel, str(index), struct, match.group("name"), match.group("type"), default.strip(), "")
        )
    return rows


def load_annotations(path: Path = ANNOTATIONS) -> dict[tuple[str, str, str], tuple[str, str, str]]:
    notes = {}
    with path.open(newline="") as stream:
        for row in csv.DictReader(stream, delimiter="\t"):
            key = (row["repo"], row["struct"], row["field"])
            if key in notes:
                raise SystemExit(f"duplicate annotation for {key}")
            notes[key] = (row["rgconf"], row["eon"], row["note"])
    return notes


def check_pins(layout: dict[str, Path], pins: dict[str, dict[str, str]]) -> None:
    for root in layout.values():
        head = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=root, capture_output=True, text=True, check=False
        ).stdout.strip()
        pin = pins[root.name]["commit"]
        if head != pin:
            raise SystemExit(
                f"{root} is at {head or 'no commit'}, but {PINS.name} pins {pin}; "
                "run tools/fetch_sources.py or update the pin and re-verify the map"
            )


def build_rows(layout: dict[str, Path], annotations: dict) -> list[tuple]:
    rows: list[tuple] = []
    for repo, root in layout.items():
        if not root.is_dir():
            raise SystemExit(f"missing source tree {root}")
        for path in iter_sources(root, repo):
            text = path.read_text(errors="replace")
            rel = str(path.relative_to(root))
            if path.suffix == ".capnp":
                rows.extend(parse_capnp(text, repo, rel))
            else:
                rows.extend(parse_cxx(text, repo, rel))
    seen = {(row[0], row[3], row[4]) for row in rows}
    orphans = sorted(set(annotations) - seen)
    if orphans:
        raise SystemExit(f"annotations name fields the sources do not declare: {orphans}")
    return [(*row, *annotations.get((row[0], row[3], row[4]), ("", "", ""))) for row in rows]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--sources", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--pins", type=Path, default=PINS)
    parser.add_argument("--annotations", type=Path, default=ANNOTATIONS)
    parser.add_argument(
        "--allow-unpinned", action="store_true", help="skip the check that each tree sits at its pinned commit"
    )
    args = parser.parse_args()
    layout = {
        "eon": args.sources / "eOn",
        "rgmin": args.sources / "rgmin",
        "rgsaddle": args.sources / "rgsaddle",
        "rgpot": args.sources / "rgpot",
        "cpmdc": args.sources / "cpmdc",
    }
    if not args.allow_unpinned:
        check_pins(layout, read_pins(args.pins))
    rows = build_rows(layout, load_annotations(args.annotations))
    args.out.parent.mkdir(parents=True, exist_ok=True)
    lines = ["\t".join(HEADER)]
    for row in rows:
        lines.append("\t".join(row))
    args.out.write_text("\n".join(lines) + "\n")
    print(f"wrote {len(rows)} rows to {args.out}")


if __name__ == "__main__":
    main()
