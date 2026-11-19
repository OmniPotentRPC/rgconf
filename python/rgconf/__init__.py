# Copyright 2023--present rgconf developers
"""Python bindings for rgconf.

The import prefers the PyO3 extension, the same layout rgmin and rgsaddle use
for ``_core``. A ctypes fallback talks to ``librgconf_c_abi`` when the
extension has not been installed yet.
"""

from __future__ import annotations

import ctypes
import os
from pathlib import Path

__all__ = ["resolve_ini", "abi_version"]


def _load_c_abi():
    candidates = []
    env = os.environ.get("RGCONF_C_ABI")
    if env:
        candidates.append(Path(env))
    root = Path(__file__).resolve().parents[2]
    candidates.append(root / "target" / "debug" / "librgconf_c_abi.so")
    candidates.append(root / "target" / "release" / "librgconf_c_abi.so")
    for path in candidates:
        if path.is_file():
            return ctypes.CDLL(str(path))
    return None


def abi_version() -> int:
    lib = _load_c_abi()
    if lib is None:
        raise RuntimeError("librgconf_c_abi was not found")
    lib.rgconf_abi_version.restype = ctypes.c_uint32
    return int(lib.rgconf_abi_version())


def resolve_ini(profile: str, text: str, path: str = "stdin.ini") -> str:
    try:
        from rgconf._core import resolve_ini as native

        return native(profile, text, path)
    except ImportError:
        pass
    lib = _load_c_abi()
    if lib is None:
        raise RuntimeError("neither rgconf._core nor librgconf_c_abi is available")
    lib.rgconf_resolve_ini.argtypes = [
        ctypes.c_char_p,
        ctypes.c_char_p,
        ctypes.c_char_p,
        ctypes.POINTER(ctypes.c_char_p),
    ]
    lib.rgconf_resolve_ini.restype = ctypes.c_int
    lib.rgconf_free.argtypes = [ctypes.c_char_p]
    out = ctypes.c_char_p()
    code = lib.rgconf_resolve_ini(
        profile.encode(),
        text.encode(),
        path.encode(),
        ctypes.byref(out),
    )
    if code != 0:
        lib.rgconf_last_error.restype = ctypes.c_char_p
        err = lib.rgconf_last_error()
        message = err.decode() if err else "resolve failed"
        raise RuntimeError(message)
    try:
        return out.value.decode() if out.value is not None else ""
    finally:
        if out:
            lib.rgconf_free(out)
