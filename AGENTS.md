# Agent Guide

Use this compact guide for automated changes in Data Engine Studio.

## Product Compass

Studio is the first customer of an eventual reusable egui-backed UI toolkit. Optimize for beautiful UI through clear, typed, composable primitives that are easy for humans and AI-assisted workflows to generate against.

Refinement bar:

- Can this be explained in one paragraph?
- Can it be tested without `egui`?
- Can it be styled without changing behavior?
- Can it be reused outside Studio?
- Can an app override it without forking it?

Good refinement lets developers declare structure, classes, and behavior intent while the system handles layout, hit testing, state, animation, and painting predictably. Avoid both bespoke per-widget plumbing and hidden magic.

Craft standard:

- Code should be fluent, well-shaped, and humane to use at every layer, including internal APIs, adapters, tests, and other dark corners users may never see.
- Treat hidden implementation surfaces as part of the product. Names, boundaries, error paths, defaults, and extension points should feel holistically designed rather than merely functional.
- Prefer the Apple/Steve Jobs craft philosophy: the unseen back of the cabinet still matters. If a part exists, it deserves care, consistency, and maximum attention to detail.

## Project Shape

- Rust owns app behavior. Python owns PyPI packaging/process entry. No `.exe` distribution requirement.
- `egui` is the native host, not the owner of product layout semantics.
- `crates/des-core`: tiny shared primitives, identity, diagnostics, errors.
- `crates/des-app`: app state, commands, snapshots, orchestration.
- `crates/des-document/core`: egui-free document tree, style model, layout, input, retained state.
- `crates/des-document/layout`: vendored layout engine derived from Taffy.
- `crates/des-document/template`: runtime and compiled document markup templates.
- `crates/des-document/widgets`: egui-free reusable widget behavior over document contracts.
- `crates/des-document/egui`: egui adapter for document input, painting, text measurement, and host defaults.
- `crates/des-ui-lab`: UI lab app, dev/screenshot binaries, lab styles, and lab regression tests.
- `crates/des-graph-egui`: vendored graph interaction crate while graph UX is explored.
- `crates/des-python` and `python/data_engine_studio`: thin Python launcher/native wrapper.
- `ROADMAP.md`: architecture and milestone plan.

Add crates only for real API boundaries, not architecture theater.

## Boundary Rules

- Export intentional front-door APIs from `lib.rs`; keep internals private by default.
- Prefer typed commands, events, snapshots, reports, and change sets over shared mutable state.
- Domain/project/runtime/query/validation logic must not depend on `des-egui` or Python.
- `des-document` and `des-widgets` must not depend on `egui` or Python.
- `des-egui` should translate egui input/output, measure text, and paint document snapshots. Keep business logic out.
- `des-python` stays thin: launch/runtime diagnostics only.
- If a small change touches many crates, revisit the boundary before continuing.

Dependency direction:

```text
python -> des-python -> des-ui-lab -> des-app -> domain/service crates -> des-core
                          \-> des-egui -> des-widgets -> des-document -> des-layout
                          \-> des-template
```

## UI Toolkit Rules

- The UI can drive discovery, but must not absorb architecture.
- Add product behavior through `des-app` commands/snapshots first or alongside UI work.
- Add low-level layout, z-order, hover, press, focus, scroll, clipping, routing, and retained state to `des-document` when reusable.
- Add reusable widget behavior to `des-widgets`.
- Keep `des-egui` adapter-focused; put exploratory app surfaces and proving-ground views in `des-ui-lab` until promoted into app crates.
- The UI lab is the proving ground for layout, styling, input, graph/canvas, table, editor, markdown, and widgets before app promotion.
- Avoid throwaway GUI framework spikes in the main tree. Remove short-lived spikes once a direction is chosen.

Style model:

```text
role defaults
then classes in declaration/rule order
then state variants
then id overrides
then explicit local overrides later if needed
```

Do not recreate CSS specificity. Keep resolution deterministic and boring. The element tree owns identity/nesting/roles/classes/text/event intent; stylesheets own visual/layout properties.

## Workspace Model

Keep these separate:

- workspace root: configured folder that can discover workspaces
- workspace: authored project area
- project document: saved graph/config/custom-code state
- workspace state: ownership/shared revisions
- runtime state: runs, logs, scheduler, caches, previews, local process state

The selected workspace must never become the app identity. Graph export/import is a first-class project/workspace concern, not UI/runtime glue.

Future close behavior must route through commands so background/tray mode can support exit, release ownership, keep runtime alive, and restore from tray. Tray is later-stage.

## Identity

- Rust app identity: `crates/des-core/src/identity.rs`
- Python identity helpers: `python/data_engine_studio/identity.py`
- Let Rust provide the default window title.
- Keep `pyproject.toml` and Cargo workspace versions aligned until a single source of truth exists.

## Commands

```sh
cargo test
cargo fmt
cargo nextest run --workspace
just --list
just dev-mac
just dev-windows
just ui-shot-mac
just ui-debug-mac
just ui-shot-windows
just ui-debug-windows
just verify
just security
cargo audit
cargo deny check
```

Python packaging:

macOS/Linux:

```sh
python3 -m venv .venv
.venv/bin/python -m pip install --upgrade pip maturin
.venv/bin/python -m maturin develop --manifest-path crates/des-python/Cargo.toml
.venv/bin/python -m data_engine_studio
.venv/bin/python -c "from data_engine_studio.native import hello, runtime_info; info = runtime_info(); print(hello()); print(info.name, info.version)"
```

Windows:

```powershell
py -3.14 -m venv .venv
.\.venv\Scripts\python.exe -m pip install --upgrade pip maturin
.\.venv\Scripts\python.exe -m maturin develop --manifest-path crates\des-python\Cargo.toml
.\.venv\Scripts\python.exe -m data_engine_studio
.\.venv\Scripts\python.exe -c "from data_engine_studio.native import hello, runtime_info; info = runtime_info(); print(hello()); print(info.name, info.version)"
```

UI iteration:

macOS/Linux:

```sh
cargo build -p des-ui-lab --bin des-ui-dev
./target/debug/des-ui-dev
just dev-mac
./scripts/capture-ui.sh --out target/ui-shots/studio.png --width 1320 --height 780
./scripts/capture-ui.sh --debug-overlay --lab-view graph
just ui-test
```

Windows:

```powershell
cargo build -p des-ui-lab --bin des-ui-dev
.\target\debug\des-ui-dev.exe
just dev-windows
.\scripts\capture-ui.ps1 -Out target\ui-shots\studio.png -Width 1320 -Height 780
.\scripts\capture-ui.ps1 -DebugOverlay
just ui-test
```

Prefer the Rust dev launcher and screenshot harness for UI iteration. Rebuild the Python extension when PyO3/Python code changes or before commits that need launcher validation.

## Compile-Time Hygiene

- Keep `des-core` tiny and stable.
- Put volatile lab UI work in `des-ui-lab`.
- Put volatile orchestration in `des-app`.
- Isolate heavy backend dependencies in future backend crates, e.g. `des-duckdb`, `des-polars`.
- Avoid broad common crates.

## Editing And Git

- Use `apply_patch` for manual edits.
- Prefer `rg`/`rg --files`.
- Do not rewrite unrelated files or revert user changes unless explicitly asked.
- Keep `.venv/`, `target/`, `build/`, `dist/`, `*.egg-info/`, secrets, host identity, and runtime residue out of git.
- Check status before/after edits: `git status --short --branch`.
- Before commits, run focused checks; run `cargo test` when Rust changed; run Python smoke when PyO3/Python changed.
- Commit at logical checkpoints so the trail stays clear and auditable; avoid letting unrelated or oversized diffs accumulate.
- Commit messages should be short and concrete.
