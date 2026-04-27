"""Central Python package identity helpers."""

from __future__ import annotations

from importlib.metadata import PackageNotFoundError, version

APP_DISPLAY_NAME = "Data Engine Studio"
APP_PACKAGE_NAME = "data-engine-studio"
APP_MODULE_NAME = "data_engine_studio"


def package_version() -> str:
    """Return the installed Python package version."""
    try:
        return version(APP_PACKAGE_NAME)
    except PackageNotFoundError:
        return "0+unknown"


__version__ = package_version()

__all__ = [
    "APP_DISPLAY_NAME",
    "APP_MODULE_NAME",
    "APP_PACKAGE_NAME",
    "__version__",
    "package_version",
]
