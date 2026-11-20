# Copyright 2023--present rgconf developers
"""The committed parameter map matches the sources it names.

The citation tests need the upstream trees at their pinned commits. Run
``python3 tools/fetch_sources.py DIR`` and point ``RGCONF_SOURCES`` at DIR.
"""

import os
import re
import subprocess
from pathlib import Path

import pytest

import import_parameters
from fetch_sources import read_pins

ROOT = Path(__file__).resolve().parents[1]
MAP = ROOT / "generated" / "parameter_map.tsv"
SCHEMA = ROOT / "crates" / "rgconf-core" / "schema"
REPOS = {"eon": "eOn", "rgmin": "rgmin", "rgsaddle": "rgsaddle", "rgpot": "rgpot", "cpmdc": "cpmdc"}
SOURCES_ENV = "RGCONF_SOURCES"
COLUMNS = ["repo", "relpath", "line", "struct", "field", "type", "default", "section", "rgconf", "eon", "note"]

# Structs whose fields were added to the map with an rgconf and eOn mapping.
ANNOTATED_STRUCTS = {
    ("rgmin", "rgmin_rlbfgs_t"),
    ("rgsaddle", "rgsaddle_artn_config_t"),
    ("rgsaddle", "rgsaddle_growing_string_config_t"),
}
MARKERS = {"unmapped", "abi"}

EXPECTED = [
    ("eon", "include/eon/ParametersOptions.h", "25", "constants_t", "kB", "8.6173324e-5"),
    ("eon", "include/eon/ParametersOptions.h", "26", "constants_t", "timeUnit", "10.1805055"),
    ("eon", "include/eon/ParametersOptions.h", "31", "main_options_t", "job", "JobType::Process_Search"),
    ("rgmin", "include/rgmin.h", "206", "rgmin_control_t", "maxiter", "100"),
    ("rgmin", "include/rgmin.h", "206", "rgmin_control_t", "gtol", "1e-5"),
    ("rgmin", "include/rgmin.h", "207", "rgmin_control_t", "maxmove", "0"),
    ("rgsaddle", "include/rgsaddle.h", "320", "rgsaddle_surface_request_t", "version", ""),
    ("rgsaddle", "include/rgsaddle.h", "321", "rgsaddle_surface_request_t", "flags", ""),
    ("rgsaddle", "include/rgsaddle.h", "322", "rgsaddle_surface_request_t", "n_images", ""),
    ("cpmdc", "schema/Potentials.capnp", "22", "ForceInput", "lengthUnit", '"angstrom"'),
    ("cpmdc", "schema/Potentials.capnp", "23", "ForceInput", "energyUnit", '"eV"'),
    ("cpmdc", "schema/Potentials.capnp", "25", "ForceInput", "charge", "0"),
    ("rgpot", "CppCore/rgpot/LennardJones/LJPot.hpp", "37", "LJConfig", "u0", "1.0"),
    ("rgpot", "CppCore/rgpot/LennardJones/LJPot.hpp", "38", "LJConfig", "cutoff", "15.0"),
    ("rgpot", "CppCore/rgpot/LennardJones/LJPot.hpp", "39", "LJConfig", "psi", "1.0"),
    ("rgpot", "CppCore/rgpot/LennardJones/LJPot.hpp", "43", "LJConfig", "switch_width", "0.0"),
]


def rows():
    lines = MAP.read_text().splitlines()
    assert lines[0].split("\t") == COLUMNS
    for line in lines[1:]:
        fields = line.split("\t")
        assert len(fields) == len(COLUMNS), line
        yield dict(zip(COLUMNS, fields))


def test_expected_rows_are_present():
    have = {tuple(row[name] for name in ("repo", "relpath", "line", "struct", "field", "default")) for row in rows()}
    for item in EXPECTED:
        assert item in have, item


def test_every_upstream_is_pinned():
    pins = read_pins()
    assert set(pins) == set(REPOS.values())
    for name, pin in pins.items():
        assert re.fullmatch(r"[0-9a-f]{40}", pin["commit"]), name
        assert pin["url"].startswith("https://github.com/"), name
        assert pin["ref"], name


def schema_has(path):
    """True when a schema path names a declaration in rgconf-core."""
    parts = path.split(".")
    if len(parts) not in (2, 3):
        return False
    schema = SCHEMA / f"{parts[0]}.capnp"
    if not schema.is_file():
        return False
    text = schema.read_text()
    start = re.search(rf"^struct {re.escape(parts[1])} \{{$", text, re.M)
    if not start:
        return False
    if len(parts) == 2:
        return True
    body = text[start.end() :].split("\n}\n", 1)[0]
    return re.search(rf"^\s*{re.escape(parts[2])} @\d+", body, re.M) is not None


def test_new_fields_carry_an_rgconf_and_eon_mapping():
    eon_fields = {f"{row['struct']}.{row['field']}" for row in rows() if row["repo"] == "eon"}
    seen = set()
    for row in rows():
        if (row["repo"], row["struct"]) in ANNOTATED_STRUCTS:
            seen.add((row["repo"], row["struct"]))
            where = f"{row['struct']}.{row['field']}"
            assert row["rgconf"], where
            assert row["eon"], where
            assert row["note"].strip(), where
        if not row["rgconf"]:
            assert not row["eon"] and not row["note"], (row["struct"], row["field"])
            continue
        if row["rgconf"] not in MARKERS:
            assert schema_has(row["rgconf"]), (row["struct"], row["field"], row["rgconf"])
        if row["eon"] != "none":
            assert row["eon"] in eon_fields, (row["struct"], row["field"], row["eon"])
    assert seen == ANNOTATED_STRUCTS


def source_root():
    value = os.environ.get(SOURCES_ENV)
    if not value or not Path(value).is_dir():
        pytest.skip(f"set {SOURCES_ENV} to a directory fetched by tools/fetch_sources.py")
    root = Path(value)
    for name, pin in read_pins().items():
        head = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=root / name, capture_output=True, text=True, check=False
        ).stdout.strip()
        assert head == pin["commit"], f"{name} is at {head or 'no commit'}, not the pinned commit"
    return root


def test_rows_quote_the_source_line():
    root = source_root()
    cache = {}
    stale = []
    for row in rows():
        repo, rel, line, struct, field = (row[name] for name in ("repo", "relpath", "line", "struct", "field"))
        path = root / REPOS[repo] / rel
        if path not in cache:
            assert path.is_file(), f"missing source file {path}"
            cache[path] = path.read_text(errors="replace").splitlines()
        lines = cache[path]
        number = int(line)
        if number < 1 or number > len(lines):
            stale.append(f"{repo}:{rel}:{line} is past the end of the file ({struct}.{field})")
        elif not re.search(rf"\b{re.escape(field)}\b", lines[number - 1]):
            stale.append(f"{repo}:{rel}:{line} does not name {field} ({struct}): {lines[number - 1].strip()!r}")
    assert not stale, f"{len(stale)} stale citations\n" + "\n".join(stale[:50])


def test_map_is_what_the_importer_writes():
    root = source_root()
    layout = {repo: root / name for repo, name in REPOS.items()}
    expected = import_parameters.build_rows(layout, import_parameters.load_annotations())
    have = [tuple(row[name] for name in COLUMNS) for row in rows()]
    assert have == expected
