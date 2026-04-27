# Agent Guide

Use this guide when making automated changes in Data Engine Studio.

## Project Shape

Data Engine Studio is a Rust-first visual ETL and data exploration studio. It is distributed through PyPI under a thin Python process. Rust owns application behavior; Python owns packaging and process entry; `egui` owns rendering and interaction.

Current workspace:

- `crates/des-core`: shared primitives such as app info, diagnostics, errors, and small stable types.
- `crates/des-app`: app state, command handling, snapshots, and orchestration. This is the composition layer.
- `crates/des-ui-egui`: egui UI shell and widgets. UI renders snapshots and sends commands.
- `crates/des-python`: PyO3 extension module exposed to Python as `data_engine_studio._native`.
- `python/data_engine_studio`: Python launcher/wrapper package.
- `ROADMAP.md`: architecture-first plan and milestone map.

Future crates should be introduced only when there is a real API boundary to preserve, not as empty architecture theater.

## Core Architecture Rules

- Treat each crate as a bounded subsystem with a small public API.
- Export intentional front-door types from `lib.rs`; keep implementation modules private unless there is a clear reason.
- Prefer typed commands, events, snapshots, reports, and change sets over shared mutable state.
- Keep graph, project, validation, query, and execution logic independent from `egui` and Python.
- Keep `des-ui-egui` out of business logic. UI should render state and dispatch commands.
- Keep `des-python` thin. It should expose native launch/runtime diagnostics, not own app behavior.
- `des-app` may coordinate crates, but should coordinate through public APIs rather than reaching into internals.
- If a small behavior change requires touching many crates, revisit the boundary before continuing.

Dependency direction should remain one-way:

```text
python package
  -> des-python
    -> des-app
      -> des-ui-egui
      -> domain/service crates
        -> des-core
```

Domain crates must not depend on `des-ui-egui` or `des-python`.

## UI Iteration Pattern

The UI is allowed to drive architecture discovery, but it must not absorb the architecture.

Preferred pattern:

```text
des-ui-egui
  renders AppSnapshot
  sends AppCommand

des-app
  applies AppCommand
  updates state
  returns AppSnapshot

domain crates
  gradually replace des-app stubs behind the same contracts
```

The primary product surface starts as a workspace/flow list. The node graph is an expanded "view/edit this flow" surface, not necessarily the first screen forever.

When adding UI behavior, add the corresponding command/snapshot shape in `des-app` first or at the same time.

## Workspace And Ownership Requirements

Studio must support more than one workspace root.

Keep these concepts distinct:

- workspace root: a configured folder that can contain/discover workspaces
- workspace: a concrete authored project area
- project document: saved graph/config/custom-code state
- workspace state: shared coordination state such as ownership or published revisions
- runtime state: runs, logs, scheduler state, caches, previews, and local process state

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

