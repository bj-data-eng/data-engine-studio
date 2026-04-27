"""Python wrapper around the Data Engine Studio native module."""

from __future__ import annotations

from importlib import import_module
from types import ModuleType
from typing import Any


def _import_first(*names: str) -> ModuleType:
    last_error: ModuleNotFoundError | None = None
    for name in names:
        try:
            return import_module(name)
        except ModuleNotFoundError as error:
            last_error = error
    if last_error is not None:
        raise last_error
    raise ModuleNotFoundError("No native module candidates were provided.")


def _module() -> ModuleType:
    return _import_first("data_engine_studio._native", "_native")


def hello() -> str:
    """Return a small native-module diagnostic string."""
    return str(_module().hello())


def launch_native(*, title: str | None = None) -> None:
    """Launch the native Rust egui shell."""
    _module().launch(title=title)


def runtime_info() -> Any:
    """Return native runtime metadata."""
    return _module().runtime_info()


__all__ = ["hello", "launch_native", "runtime_info"]

