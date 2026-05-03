# Agent Guide

Use this guide when making automated changes in Data Engine Studio.

## Project Shape

Data Engine Studio is a Rust-first visual ETL and data exploration studio. It is distributed through PyPI under a thin Python process. Rust owns application behavior; Python owns packaging and process entry; `egui` is a native rendering/input host rather than the long-term owner of product layout semantics.

Current workspace:

- `crates/des-core`: shared primitives such as app info, diagnostics, errors, and small stable types.
- `crates/des-app`: app state, command handling, snapshots, and orchestration. This is the composition layer.
- `crates/des-ui-document`: standalone-style UI document and style model with deterministic style resolution, resolved layout output, retained element state, and input routing. This crate must not depend on egui.
- `crates/des-ui-widgets`: reusable document-backed widget behavior built on the document model, such as sortable drag/drop policies. This crate must not depend on egui or Python.
- `crates/des-graph-egui`: vendored graph interaction crate kept while graph UX is still being explored.
- `crates/des-ui-egui`: egui host adapter and current UI lab. UI renders app snapshots/document output and sends commands.
- `crates/des-python`: PyO3 extension module exposed to Python as `data_engine_studio._native`.
- `python/data_engine_studio`: Python launcher/wrapper package.
- `ROADMAP.md`: architecture-first plan and milestone map.

Future crates should be introduced only when there is a real API boundary to preserve, not as empty architecture theater.

## App Identity

Keep app identity centralized.

- Rust runtime identity lives in `crates/des-core/src/identity.rs`.
- Python package identity helpers live in `python/data_engine_studio/identity.py`.
- The Python launcher should let Rust provide the default window title instead of hardcoding it.
- `pyproject.toml` and the Cargo workspace version must stay aligned until a single build-time source of truth is introduced.

## Core Architecture Rules

- Treat each crate as a bounded subsystem with a small public API.
- Export intentional front-door types from `lib.rs`; keep implementation modules private unless there is a clear reason.
- Prefer typed commands, events, snapshots, reports, and change sets over shared mutable state.
- Keep graph, project, validation, query, and execution logic independent from `egui` and Python.
- Keep `des-ui-document` independent from `egui` and Python. It should expose product-neutral UI document contracts: element trees, style sheets, resolved elements, input events, and retained interaction state.
- Keep `des-ui-widgets` independent from `egui` and Python. It should expose reusable widget behavior and policies that operate through `des-ui-document` contracts.
- Keep `des-ui-egui` out of business logic. UI should render state and dispatch commands.
- Treat `des-ui-egui` as an adapter/host for the document engine where possible: translate egui input into document input and paint resolved document output through egui/epaint.
- Keep `des-python` thin. It should expose native launch/runtime diagnostics, not own app behavior.
- `des-app` may coordinate crates, but should coordinate through public APIs rather than reaching into internals.
- If a small behavior change requires touching many crates, revisit the boundary before continuing.

Dependency direction should remain one-way:

```text
python package
  -> des-python
    -> des-app
      -> des-ui-egui
        -> des-ui-widgets
        -> des-ui-document
      -> domain/service crates
        -> des-core
```

Domain crates must not depend on `des-ui-egui` or `des-python`.

## UI Iteration Pattern

The UI is allowed to drive architecture discovery, but it must not absorb the architecture.

Preferred pattern:

```text
des-ui-document
  owns the document tree + style model
  resolves layout, z-order, hit testing, and retained UI state

des-ui-widgets
  provides reusable document-backed widget behavior
  emits typed widget changes through document snapshots and events

des-ui-egui
  adapts egui input/output to the document engine
  renders AppSnapshot and sends AppCommand

des-app
  applies AppCommand
  updates state
  returns AppSnapshot

domain crates
  gradually replace des-app stubs behind the same contracts
```

The primary product surface is a full-window graph workspace. Workspace roots, workspaces, and grouped flows can appear as high-level graph nodes. The node graph for a selected flow expands from the flow card into source/transform/sink nodes.

When adding product behavior, add the corresponding command/snapshot shape in `des-app` first or at the same time. When adding low-level UI behavior such as layout, z-order, hover, press, focus, scroll ownership, clipping, or event routing, prefer adding it to `des-ui-document` rather than patching around it in product UI code.

Do not recreate CSS specificity. Document engine style resolution should stay deterministic and boring:

```text
role defaults
then classes in declaration/rule order
then state variants
then id overrides
then explicit local overrides later if needed
```

The element tree defines identity, nesting, roles, classes, text, and event intent. The style sheet defines visual and layout properties such as size, margin, padding, z-index, color, borders, overflow, and layout direction.

Avoid a monolithic GUI crate. The current first screen is a document-engine-backed UI lab, not the final product shell. Use it to exercise layout, styling, input, graph/canvas, table, editor, markdown, and other UI features before promoting them into app surfaces. Split `des-ui-egui` internally by product surface and responsibility as it grows. Prefer small modules such as:

- `ui_lab`: document-engine-backed testing UI for all emerging UI features.
- `egui_adapter`: egui input/output translation for `des-ui-document`.
- `shell`: top-level frame, menus, command routing, global layout.
- `workspace_browser`: workspace roots, workspace list, ownership/status affordances.
- `flow_list`: grouped built ETL flow cards embedded in the workspace/root graph node.
- `flow_editor`: expanded flow editor container.
- `graph_canvas`: node canvas drawing and interactions.
- `node_palette`: source/transform/trigger/sink palette.
- `inspector`: selected flow/node/property editing.
- `runtime_panel`: run state, validation, logs, and status.
- `theme`: visual constants and egui styling.

UI modules may share small view models from `des-app`, but should not reach into graph/project/runtime internals directly.

Avoid new throwaway GUI framework spikes in the main tree. If a spike is needed, keep it short-lived and remove it once a direction is chosen.

## Workspace And Ownership Requirements

Studio must support more than one workspace root.

Keep these concepts distinct:

- workspace root: a configured folder that can contain/discover workspaces
- workspace: a concrete authored project area
- project document: saved graph/config/custom-code state
- workspace state: shared coordination state such as ownership or published revisions
- runtime state: runs, logs, scheduler state, caches, previews, and local process state

The selected workspace must never "become" the app. The app owns a catalog of workspaces and can show zero, one, or many workspace contexts over time. A workspace is selected or opened within Studio; it is not the root application identity.

Workspace ownership and runtime ownership may become separate concepts. Do not assume one global lock unless the roadmap explicitly changes.

Graph export/import is a first-class requirement and belongs in the project/workspace layer, not in the UI or runtime layer.

## Background Process And Tray

Closing the egui window should eventually route through app command handling. Do not wire UI close directly to runtime shutdown in a way that blocks background mode later.

The eventual behavior should support:

- exit application and release ownership
- keep process in background
- keep runtime/scheduler alive after the main window closes
- restore from a Windows notification-area tray icon

Tray support is later-stage, not a Milestone 1 requirement.

## Python Packaging And Launch

Use the workspace-local venv:

```powershell
py -3.14 -m venv .venv
.\.venv\Scripts\python.exe -m pip install --upgrade pip
.\.venv\Scripts\python.exe -m pip install maturin
```

Install the native extension into the venv:

```powershell
.\.venv\Scripts\python.exe -m maturin develop --manifest-path crates\des-python\Cargo.toml
```

Launch the app:

```powershell
.\.venv\Scripts\python.exe -m data_engine_studio
```

Do not require a user-managed `.exe` distribution. The app should launch through Python and PyPI packaging. Native artifacts should be extension modules such as `.pyd`, `.so`, or `.dylib`.

## Build And Test Commands

Rust tests:

```powershell
cargo test
```

Faster Rust tests:

```powershell
cargo nextest run --workspace
```

Format Rust:

```powershell
cargo fmt
```

Python native import smoke:

```powershell
.\.venv\Scripts\python.exe -c "from data_engine_studio.native import hello, runtime_info; info = runtime_info(); print(hello()); print(info.name, info.version)"
```

Native editable install:

```powershell
.\.venv\Scripts\python.exe -m maturin develop --manifest-path crates\des-python\Cargo.toml
```

UI screenshot harness:

```powershell
.\scripts\capture-ui.ps1 -Out target\ui-shots\studio.png -Width 1320 -Height 780
```

The screenshot harness builds `des-ui-egui` with the `ui-screenshot` feature, launches the dedicated `des-ui-shot` binary, writes the PNG named by `EFRAME_SCREENSHOT_TO`, and exits automatically. Use it for automated UI iteration. Do not launch the main Python app unless the user explicitly asks.

Harness knobs:

```powershell
.\scripts\capture-ui.ps1 -DebugOverlay
```

Use `-DebugOverlay` for UI lab diagnostics. As the document lab grows, prefer seeding views through launch-time options and app commands rather than test-only branches in the product UI.

UI graphical testing:

```powershell
just ui-test
```

See `docs/UI_TESTING.md` for the document assertion, reference-test, graphical comparison, interaction, and manual lab testing layers.

Fast UI launch for local iteration:

```powershell
cargo build -p des-ui-egui --bin des-ui-dev
.\target\debug\des-ui-dev.exe
```

During rapid UI iteration, prefer the Rust dev launcher and screenshot harness over rebuilding the Python extension. Run `maturin develop` when PyO3 or Python package code changes, and before commits that need the Python launcher validated.

Command runner:

```powershell
just --list
just ui-shot
just ui-debug
just verify
just security
```

Dependency checks:

```powershell
cargo audit
cargo deny check
```

Run focused checks before committing. Run `cargo test` when Rust code changed. Run the Python import smoke when PyO3 or Python package code changed.

## Compile-Time Hygiene

Multiple crates are used for both architecture and incremental compilation.

- Keep `des-core` tiny and stable because many crates will depend on it.
- Put volatile UI work in `des-ui-egui`.
- Put volatile orchestration in `des-app`.
- Keep heavy backend dependencies isolated in backend crates when introduced, such as `des-duckdb` and `des-polars`.
- Avoid broad "common" crates that everyone edits.

## Editing Guidelines

- Use `apply_patch` for manual file edits.
- Prefer `rg` and `rg --files` for search.
- Do not rewrite unrelated files.
- Do not revert user changes unless explicitly asked.
- Keep generated local artifacts out of git, especially `.venv/`, `target/`, `build/`, `dist/`, and `*.egg-info/`.
- Keep local absolute paths, secrets, tokens, host identity data, and runtime residue out of committed docs and samples.

## Git Hygiene

Check status before and after edits:

```powershell
git status --short --branch
```

Before committing:

```powershell
cargo test
git status --short
```

Commit messages should be short and concrete, for example:

```text
Add app command snapshot model
Scaffold graph document crate
Wire workspace catalog shell
```
