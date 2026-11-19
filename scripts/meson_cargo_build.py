# Copyright 2023--present rgconf developers
"""Build the rgconf C ABI with cargo and copy the shared and static libraries."""

import shutil
import subprocess
import sys
from pathlib import Path


def main() -> None:
    cargo, source, target, profile, shared_name, static_name, out_shared, out_static = sys.argv[1:9]
    command = [
        cargo,
        "build",
        "-p",
        "rgconf-c-abi",
        "--manifest-path",
        str(Path(source) / "Cargo.toml"),
        "--target-dir",
        target,
    ]
    if profile == "release":
        command.append("--release")
    subprocess.run(command, check=True)
    built = Path(target) / profile
    shutil.copy2(built / shared_name, out_shared)
    shutil.copy2(built / static_name, out_static)


if __name__ == "__main__":
    main()
