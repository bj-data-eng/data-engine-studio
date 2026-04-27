"""Launch Data Engine Studio through the native Rust UI module."""

from __future__ import annotations

from data_engine_studio.native import launch_native


def launch() -> None:
    """Launch the desktop Studio shell."""
    launch_native()


def main() -> None:
    """Console entrypoint."""
    launch()


__all__ = ["launch", "main"]
