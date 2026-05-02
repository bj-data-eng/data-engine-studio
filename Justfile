set shell := ["pwsh", "-NoLogo", "-NoProfile", "-Command"]

default:
    just --list

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

check:
    cargo check --workspace

test:
    cargo nextest run --workspace

test-cargo:
    cargo test

ui-shot out='target/ui-shots/studio.png':
    ./scripts/capture-ui.ps1 -Out "{{out}}" -Width 1320 -Height 780

ui-debug out='target/ui-shots/studio-debug.png':
    ./scripts/capture-ui.ps1 -Out "{{out}}" -Width 1320 -Height 780 -DebugOverlay -LabView graph

ui-test:
    cargo test -p des-ui-egui ui_lab::tests

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
