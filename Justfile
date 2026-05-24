set shell := ["sh", "-cu"]

default:
    just --list

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

check:
    cargo check --workspace

build-mac:
    cargo build --release -p des-ui-lab --bin des-ui-dev
    exec ./target/release/des-ui-dev

test:
    cargo nextest run --workspace

test-cargo:
    cargo test

dev-mac:
    ./scripts/run-dev.sh

dev-windows:
    pwsh -NoLogo -NoProfile -File ./scripts/run-dev.ps1

ui-shot-mac out='target/ui-shots/studio.png':
    ./scripts/capture-ui.sh --out "{{out}}" --width 1320 --height 780

ui-shot-windows out='target/ui-shots/studio.png':
    pwsh -NoLogo -NoProfile -File ./scripts/capture-ui.ps1 -Out "{{out}}" -Width 1320 -Height 780

ui-debug-mac out='target/ui-shots/studio-debug.png':
    ./scripts/capture-ui.sh --out "{{out}}" --width 1320 --height 780 --debug-overlay --lab-view graph

ui-debug-windows out='target/ui-shots/studio-debug.png':
    pwsh -NoLogo -NoProfile -File ./scripts/capture-ui.ps1 -Out "{{out}}" -Width 1320 -Height 780 -DebugOverlay -LabView graph

ui-test:
    cargo test -p des-ui-lab ui_lab::tests

python-test:
    PYTHONPATH=python python3 -m unittest discover -s python/tests -p 'test_*.py'

python-smoke:
    python3 -m venv .venv
    .venv/bin/python -m pip install --upgrade pip maturin==1.13.3
    .venv/bin/python -m maturin develop --manifest-path crates/des-python/Cargo.toml
    .venv/bin/python -c "from data_engine_studio.native import hello, runtime_info; info = runtime_info(); print(hello()); print(info.name, info.version)"

audit:
    cargo audit

deny:
    cargo deny check

security:
    cargo audit
    cargo deny check

verify:
    cargo fmt --check
    cargo check --workspace
    cargo nextest run --workspace

verify-all:
    just verify
    just security
    just python-test
    just python-smoke
