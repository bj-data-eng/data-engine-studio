set shell := ["sh", "-cu"]

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
