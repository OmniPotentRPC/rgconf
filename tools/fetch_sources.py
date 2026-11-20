# Copyright 2023--present rgconf developers
"""Fetch each upstream named in generated/sources.tsv at its pinned commit.

Every tree lands in its own directory under the destination, named as the
first column of the pin file, which is the layout import_parameters.py and
the map tests read.
"""

from __future__ import annotations

import argparse
import csv
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PINS = ROOT / "generated" / "sources.tsv"


def read_pins(path: Path = PINS) -> dict[str, dict[str, str]]:
    with path.open(newline="") as stream:
        return {row["repo"]: row for row in csv.DictReader(stream, delimiter="\t")}


def git(*args: str, cwd: Path) -> str:
    done = subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True, text=True)
    return done.stdout.strip()


def fetch(pin: dict[str, str], dest: Path) -> None:
    dest.mkdir(parents=True, exist_ok=True)
    if not (dest / ".git").exists():
        git("init", "-q", cwd=dest)
        git("remote", "add", "origin", pin["url"], cwd=dest)
    if _has_head(dest) and git("rev-parse", "HEAD", cwd=dest) == pin["commit"]:
        return
    git("fetch", "-q", "--depth", "1", "origin", pin["commit"], cwd=dest)
    git("checkout", "-q", "--detach", "FETCH_HEAD", cwd=dest)
    head = git("rev-parse", "HEAD", cwd=dest)
    if head != pin["commit"]:
        raise SystemExit(f"{pin['repo']}: fetched {head}, expected {pin['commit']}")


def _has_head(dest: Path) -> bool:
    return subprocess.run(["git", "rev-parse", "--verify", "-q", "HEAD"], cwd=dest, capture_output=True).returncode == 0


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dest", type=Path)
    args = parser.parse_args()
    for repo, pin in read_pins().items():
        fetch(pin, args.dest / repo)
        print(f"{repo} {pin['ref']} {pin['commit']}")


if __name__ == "__main__":
    main()
