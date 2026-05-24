from __future__ import annotations

import importlib
import sys
import types
import unittest
from unittest import mock


def reload_native():
    sys.modules.pop("data_engine_studio.native", None)
    return importlib.import_module("data_engine_studio.native")


class NativeWrapperTests(unittest.TestCase):
    def tearDown(self) -> None:
        sys.modules.pop("data_engine_studio._native", None)
        sys.modules.pop("_native", None)

    def test_prefers_packaged_native_module(self) -> None:
        packaged = types.SimpleNamespace(
            hello=lambda: "packaged",
            runtime_info=lambda: types.SimpleNamespace(name="packaged", version="1"),
        )
        fallback = types.SimpleNamespace(
            hello=lambda: "fallback",
            runtime_info=lambda: types.SimpleNamespace(name="fallback", version="1"),
        )
        with mock.patch.dict(
            sys.modules,
            {"data_engine_studio._native": packaged, "_native": fallback},
        ):
            native = reload_native()

            self.assertEqual(native.hello(), "packaged")
            self.assertEqual(native.runtime_info().name, "packaged")

    def test_falls_back_to_top_level_native_module_for_development(self) -> None:
        fallback = types.SimpleNamespace(
            hello=lambda: "fallback",
            runtime_info=lambda: types.SimpleNamespace(name="fallback", version="1"),
        )
        with mock.patch.dict(sys.modules, {"_native": fallback}):
            native = reload_native()

            self.assertEqual(native.hello(), "fallback")
            self.assertEqual(native.runtime_info().name, "fallback")

    def test_launch_native_forwards_optional_title_keyword(self) -> None:
        calls: list[str | None] = []
        module = types.SimpleNamespace(launch=lambda *, title=None: calls.append(title))
        with mock.patch.dict(sys.modules, {"data_engine_studio._native": module}):
            native = reload_native()

            native.launch_native(title="Studio QA")

        self.assertEqual(calls, ["Studio QA"])


class LauncherTests(unittest.TestCase):
    def test_main_delegates_to_launch(self) -> None:
        launcher = importlib.import_module("data_engine_studio.launcher")

        with mock.patch.object(launcher, "launch") as launch:
            launcher.main()

        launch.assert_called_once_with()

    def test_launch_delegates_to_native_launcher(self) -> None:
        launcher = importlib.import_module("data_engine_studio.launcher")

        with mock.patch.object(launcher, "launch_native") as launch_native:
            launcher.launch()

        launch_native.assert_called_once_with()


if __name__ == "__main__":
    unittest.main()
