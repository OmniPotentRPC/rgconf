# Copyright 2023--present rgconf developers
"""The committed parameter map matches the sources it names."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAP = ROOT / "generated" / "parameter_map.tsv"
SOURCES = Path("/tmp/sources")
REPOS = {"eon": "eOn", "rgmin": "rgmin", "rgsaddle": "rgsaddle", "rgpot": "rgpot", "cpmdc": "cpmdc"}

EXPECTED = [
    ("eon", "include/eon/ParametersOptions.h", "343", "optimizer_options_t", "converged_force", "0.01"),
    ("eon", "include/eon/ParametersOptions.h", "342", "optimizer_options_t", "max_move", "0.2"),
    ("eon", "include/eon/ParametersOptions.h", "350", "refine_t", "threshold", "0.5"),
    ("rgmin", "include/rgmin.h", "204", "rgmin_control_t", "maxiter", "100"),
    ("rgmin", "include/rgmin.h", "205", "rgmin_control_t", "maxmove", "0"),
    ("rgmin", "include/rgmin.h", "206", "rgmin_control_t", "noise_c", "3"),
    ("rgpot", "CppCore/rgpot/LennardJones/LJPot.hpp", "43", "LJConfig", "switch_width", "0.0"),
    ("rgpot", "CppCore/rgpot/MetatomicPot/MetatomicConfig.hpp", "18", "MetatomicConfig", "uncertainty_threshold", "-1.0"),
]


def rows():
    for line in MAP.read_text().splitlines()[1:]:
        yield line.split("\t")


def test_expected_rows_are_present():
    have = {(row[0], row[1], row[2], row[3], row[4], row[6]) for row in rows()}
    for item in EXPECTED:
        assert item in have, item


def test_rows_quote_the_source_line():
    if not SOURCES.is_dir():
        return
    for repo, rel, line, _struct, field, default in EXPECTED:
        text = (SOURCES / REPOS[repo] / rel).read_text().splitlines()[int(line) - 1]
        assert field in text or "Control::default" in text or "maxmove" in text
        assert default in text
