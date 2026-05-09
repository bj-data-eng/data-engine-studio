# Data Engine Studio Roadmap

Data Engine Studio is a Rust-first desktop studio for visual ETL, data exploration, and future semantic modeling. The app is distributed through PyPI under a thin Python process, with Rust owning the application behavior and `egui` acting as the native rendering/input host.

This roadmap is intentionally architecture-first. The core product requirement is clear separation of concerns: each major crate should behave like a small application with a stable public API, private internals, and its own tests.

## Architecture Principles

- Treat each crate as a bounded subsystem with a small public API.
- Prefer typed commands, events, snapshots, and reports over shared mutable structures.
- Keep graph, project, validation, query, and execution logic independent from `egui` and Python.
- Keep Python as packaging and process entry only.
- Keep `egui` as a host adapter. Product layout, hit testing, z-order, retained UI state, and input routing should move toward `des-ui-document`.
- Keep Studio as the app; a selected workspace must not become the application identity.
- Make runtime truth explicit through snapshots and events; logs are history, not the control plane.
- Design for unit tests at crate boundaries before integration tests across the full app.
- If a small behavior change requires touching many crates, revisit the boundary.

## Target Crate Shape

Initial crates:

- `des-core`: shared IDs, diagnostics, errors, time helpers, result types, and small primitives.
- `des-graph`: graph document model, nodes, ports, edges, graph mutations, traversal, and selection-neutral operations.
- `des-nodes`: built-in node definitions, node categories, config schemas, port contracts, defaults, and node registry.
- `des-validation`: pure validation over graph and node definitions, producing diagnostics and validation reports.
- `des-query`: backend-neutral logical expressions and relational plans inspired by fluent query builders such as Ibis.
- `des-project`: project files, workspace layout, workspace catalog, save/load, graph export/import, migrations, ownership state, and project-level commands.
- `des-runtime`: runtime commands, events, snapshots, job IDs, cancellation, and Tokio-backed background coordination.
- `des-execution`: graph-to-plan execution, backend selection, stage execution, progress reporting, and runtime integration.
- `des-artifacts`: previews, profiling outputs, debug artifacts, schema snapshots, parquet/json metadata, and artifact indexing.
- `des-duckdb`: DuckDB connection management, SQL lowering, query execution, and result materialization.
- `des-polars`: Polars LazyFrame/DataFrame support, previews, profiling, and local transform lowering.
- `des-ui-document`: standalone-style UI document and style model with DOM-like element trees, deterministic CSS-like style sheets, resolved elements, retained interaction state, z-order, hit testing, and input routing.
- `des-ui-egui`: egui host adapter, document paint/input integration, text measurement, and host defaults.
- `des-ui-lab`: current document-engine-backed lab app, screenshot harness, lab styles, and lab regression suite.
- `des-app`: application orchestration, command handling, undo/redo, document lifecycle, validation/runtime wiring.
- `des-python`: PyO3 extension module exposing the native app launcher to Python.

Dependency direction:

```text
python package
  -> des-python
    -> des-ui-lab
      -> des-app
      -> des-ui-egui
        -> des-ui-document
      -> des-project
      -> des-validation
      -> des-runtime
      -> des-execution
      -> des-graph
      -> des-nodes
      -> des-core
```

Domain crates must not depend on `des-ui-egui` or `des-python`.

## Public API Expectations

Each crate should expose a narrow front door from `lib.rs`.

Examples:

- `des-validation`: `ValidationService`, `ValidationInput`, `ValidationReport`.
- `des-runtime`: `RuntimeHost`, `RuntimeCommand`, `RuntimeEvent`, `RuntimeSnapshot`.
- `des-project`: `ProjectDocument`, `ProjectCommand`, `ProjectChangeSet`, `ProjectLoadResult`, `ProjectSaveResult`.
- `des-graph`: `GraphDocument`, `GraphCommand`, `GraphChangeSet`, `NodeId`, `PortId`, `EdgeId`.
- `des-nodes`: `NodeDefinition`, `NodeRegistry`, `NodeKind`, `PortDefinition`, `ConfigSchema`.
- `des-ui-document`: `Document`, `ElementSpec`, `StyleSheet`, `StyleSelector`, `DocumentEngine`, `DocumentInput`, `DocumentOutput`, `ResolvedElement`.

Internal modules can change freely, but public API types should be deliberate and tested.

## UI Document Direction

Studio needs complex, browser-grade composition behavior without adopting HTML/CSS/JavaScript as the product platform. The UI document engine should provide the low-level machinery that raw egui code makes difficult to coordinate manually:

- nesting and stable element identity
- deterministic style and size resolution
- margin, padding, layout direction, gaps, z-index, overflow, and clipping
- hover, pressed, focused, selected, and disabled state
- hit testing and event targeting
- scroll and drag ownership
- graph/canvas geometry integration
- host-independent layout tests

The document style model should be CSS-like, not CSS-compatible. Avoid CSS specificity rules. Preferred style resolution order:

```text
element defaults -> classes in rule/declaration order -> state variants -> id overrides
```

The document tree defines what exists: identity, nesting, semantic element, classes, text, and event intent. The style sheet defines how it is sized, positioned, layered, and painted. `des-ui-egui` is the adapter that translates egui input into document input and paints resolved document elements through egui/epaint.

Specialized engines should plug into this document layer rather than being reimplemented inside it. Markdown rendering, code editing, syntax highlighting, virtualized data grids, charts, graph canvases, and transform visualizations can be dedicated subsystems with document-managed bounds, focus, z-order, and input ownership.

## Packaging Plan

The app must launch from Python and be distributed through PyPI.

Target launch path:

```text
data-engine-studio console script
  -> data_engine_studio.launcher.main()
  -> data_engine_studio.native.launch_native()
  -> data_engine_studio._native.launch()
  -> des-python
  -> des-ui-lab
```

Implementation requirements:

- Use `pyo3` and `maturin` to build a native Python extension.
- Use an extension module artifact (`.pyd`, `.so`, `.dylib`), not a user-managed standalone executable.
- Keep the Rust app runnable from Rust tests without Python.
- Keep the distributed user entrypoint Python-first.
- Validate the minimal Python-launched egui window early on Windows, macOS, and Linux.

## Workspaces, Ownership, And Export

Studio must support more than one workspace root. A workspace root is a folder that can contain or discover one or more authored workspaces. A workspace is the concrete project area where flows, graph files, custom code, workspace config, and shared state live.

Studio owns the workspace catalog. Opening a workspace changes the active context, not the identity or lifecycle of the whole app. The app must be able to start with no selected workspace, switch workspaces, show workspace status, and eventually support multiple workspace contexts without rewriting the shell.

The primary UI should be a full-window graph workspace rather than a permanent sidebar beside a graph. Workspace roots, workspaces, and grouped flows can be represented as high-level nodes. Selecting a flow expands its flow graph from that card, with connectors from the selected flow into source nodes and then through transform/sink stages.

Initial workspace model:

- `WorkspaceCatalog`: machine-local list of configured workspace roots.
- `WorkspaceRoot`: one configured root path plus scan status.
- `Workspace`: one discovered authored workspace with stable ID, display name, root path, availability, ownership, and project metadata.
- `SelectedWorkspace`: app state, not a global singleton.

Workspace commands should be exposed through a stable API:

- `AddWorkspaceRoot`
- `RemoveWorkspaceRoot`
- `ScanWorkspaceRoots`
- `SelectWorkspace`
- `OpenProject`
- `AcquireWorkspaceLease`
- `ReleaseWorkspaceLease`
- `CheckWorkspaceStatus`

The project layer owns persistence, graph export/import, save transactions, conflict detection, and workspace ownership. The runtime layer owns run coordination, cancellation, scheduling, and execution state. The UI only renders ownership state and sends commands.

Comprehensive graph export is a first-class requirement. Exported graph packages should be able to represent a complete authored flow without relying on egui memory or local runtime caches.

Graph export should include:

- project and schema version
- stable graph, node, port, edge, trigger, and flow IDs
- graph nodes and edges
- node configs
- trigger definitions
- compound node card lists
- custom Python/SQL node code
- UI layout needed to reopen the flow editor
- dependency metadata and referenced asset manifests where applicable

Graph export should not include by default:

- machine-local paths
- active runtime state
- local caches
- transient UI hover/drag state
- secrets
- workstation lease rows
- temporary previews or debug artifacts unless an explicit bundle export requests them

Expected export modes:

- Save project: writes the authoritative workspace/project document.
- Export graph: produces a portable authored-flow package for sharing, review, import, or backup.
- Export bundle later: graph plus selected scripts, assets, sample data, or debug artifacts.

Save behavior should be transaction-oriented:

```text
base_revision -> save transaction -> new_revision
```

If another workstation changed the project first, the project layer should return a conflict result instead of silently overwriting.

## Node Model

Nodes should be represented as document state, runtime state, and UI state separately.

Core node categories:

- Trigger nodes: manual, schedule, poll, file watch later.
- Source nodes: file, database, API, message queue.
- Transform nodes: parse, select/rename, filter, map/derive, join, aggregate, sort, deduplicate, data quality.
- Compound transform nodes: one canvas node containing a linear card list of smaller Rust-backed operations.
- Custom nodes: Python transform first, SQL transform later.
- Sink nodes: file output, database output, API output, message queue output.

Ports must be typed. Trigger/control ports should not pretend to be data ports.

Example graph:

```text
Manual Trigger --run--> File Source --table--> Compound Transform --table--> Database Output
Schedule Trigger --run--> Database Source --table--> Aggregate --table--> File Output
Poll Trigger --run--> API Source --table--> Python Transform --table--> Sink
```

## Python Transform Node

The Python mini-IDE node is a first-class custom node and an explicit execution boundary.

Initial contract:

- Accept one input table/frame/batch.
- Produce one output table/frame/batch.
- Store code, entry function, input contract, and output contract in node config.
- Prefer Polars for Python dataframe transforms, with Arrow as an interchange option where useful.
- Make schema inference and optimization boundaries visible to the planner.

Python transform nodes should not become the default implementation path for built-in ETL behavior.

## Runtime Plan

Use Tokio for runtime coordination and background work.

Tokio should own:

- execution jobs
- async IO
- cancellation
- progress events
- file/database/API activity
- background validation
- scheduling and polling

Tokio should not be required for:

- graph mutation
- node definitions
- validation rule functions
- project serialization
- command application
- undo/redo
- egui rendering

Runtime communication should be command/event based:

```text
des-app -> RuntimeCommand -> des-runtime
des-runtime -> RuntimeEvent -> des-app
des-app -> RuntimeSnapshot -> des-ui-lab
```

Use Rayon later for CPU-bound work such as semantic model analysis, profiling, lineage, dependency resolution, or large local transforms.

UI lifetime and runtime lifetime must be separate. Closing the window should route through app command handling, not direct runtime shutdown.

Close behavior should eventually support:

- exit application and release ownership
- keep process in background
- keep runtime/scheduler alive while the main window is closed
- restore the window from a background control surface

The Windows notification-area tray is the preferred long-term UX for background mode. Tray support is useful but not critical for early milestones; it can arrive after the core workspace/runtime model is stable.

## Data Engine Requirements To Preserve

Lessons carried forward from `data-engine`:

- Manual, poll, and schedule execution modes are core requirements.
- Graceful stop semantics must be modeled from the beginning.
- Already-started work may finish during graceful stop.
- Queued work must not start after stop is requested.
- Workspace-authored state, generated cache, and machine-local runtime state must stay separate.
- More than one workspace root must be supported.
- Workspace ownership and runtime ownership must be explicit and visible.
- Debug artifacts and dataframe previews are product features, not incidental logs.
- Live runtime snapshots should be authoritative for UI state.
- Persisted logs and history should enrich the UI without becoming the live control plane.
- UI projections should consume stable read models rather than reaching into execution internals.

## Milestones

### Milestone 1: Workspace And Build Skeleton

- Create the Cargo workspace.
- Create the Python package skeleton.
- Add `pyo3`/`maturin` native extension plumbing.
- Launch a minimal egui window from `python -m data_engine_studio`.
- Keep the shell shaped around a full-window graph workspace, with roots/workspaces/flows as high-level nodes and selected flows expanding into editable node graphs.
- Add CI-ready commands for Rust tests, Python import smoke tests, and formatting.

### Milestone 2: Core Graph And Node Registry

- Implement stable ID types.
- Implement graph document model.
- Implement typed node ports and edges.
- Implement graph commands and change sets.
- Implement built-in node definitions for initial trigger/source/transform/sink categories.
- Add unit tests for graph mutation, port compatibility, and node registry behavior.

### Milestone 3: Project Format

- Define initial project file format.
- Implement save/load round trips.
- Implement initial workspace catalog types with support for multiple workspace roots.
- Define graph export/import package shape.
- Add project migrations from version `0`.
- Add project revision metadata and save conflict result types.
- Separate document state from UI state and runtime state.
- Add tests for serialization, invalid files, export/import, conflict results, and migration behavior.

### Milestone 4: Validation

- Implement validation service public API.
- Validate missing configs, invalid edges, incompatible ports, disconnected required inputs, and invalid trigger wiring.
- Return stable diagnostics keyed by project/node/edge/port IDs.
- Add contract tests for validation reports.

### Milestone 5: egui Shell

- Use the document-engine-backed `des-ui-lab` app as the first screen while the UI platform is being built. The lab should expose each document feature in one or more views before that feature is used in the app proper.
- Build the first usable app shell:
  - menu/toolbar
  - left node palette
  - central graph canvas
  - right inspector
  - bottom status/runtime bar
- Keep `des-ui-egui` adapter-only; promote real product surfaces out of `des-ui-lab` into focused app/UI crates as they become real.
- Implement selection, panning, zooming, adding nodes, moving nodes, and connecting ports.
- Keep UI state out of graph/project state.
- Begin migrating reusable layout/interaction behavior into `des-ui-document`.
- Keep egui-specific code focused on hosting, painting, font access, and platform input.

### Milestone 6: Query And Backend Foundations

- Implement `des-query` logical expressions and relational operations.
- Study Ibis-style fluent builders and adopt only the useful internal concepts.
- Add DuckDB and Polars backend crates.
- Implement basic lowering for simple source/filter/select/aggregate flows.
- Add tests that compare logical plans to expected backend plans.

### Milestone 7: Runtime And Execution

- Implement runtime host with command submission, event draining, snapshots, and shutdown.
- Implement execution planning from validated graph snapshots.
- Run a minimal local ETL graph end to end.
- Add cancellation and graceful stop behavior.
- Persist run history and basic logs.
- Keep runtime lifecycle independent from egui window lifecycle.

### Milestone 8: Artifacts, Preview, And Profiling

- Add node-level previews.
- Add schema snapshots, row counts, and lightweight profiling.
- Persist debug artifacts with metadata.
- Render preview/profiling state in the inspector.

### Milestone 9: Python Transform Node

- Add Python transform node config and editor surface.
- Execute Python transform nodes through a controlled boundary.
- Define input/output schema behavior.
- Surface Python errors as structured diagnostics and runtime events.

### Milestone 10: Scheduling And Polling

- Implement manual trigger fully.
- Add schedule trigger execution.
- Add poll trigger execution with source freshness tracking.
- Ensure runtime snapshots remain accurate across concurrent or queued runs.

### Milestone 11: Hardening

- Add integration tests across project, validation, runtime, and execution.
- Add package build checks for PyPI wheels.
- Add app-level smoke tests for launch and minimal graph operations.
- Document public crate APIs and architecture rules.

### Later: Background Process And Tray

- Add a `keep process in background` setting.
- Intercept window close through app command handling.
- Keep the runtime host alive when background mode is enabled.
- Add Windows notification-area tray support.
- Add tray actions for open, runtime status, pause/stop runtime, and quit.
- Ensure explicit quit releases workspace/runtime ownership cleanly.

## Open Design Questions

- Exact project file format: JSON, TOML plus graph JSON, or another structured format.
- Whether Python transform input should be Polars-only at first or Arrow-first with Polars adapters.
- How much SQL editing belongs in first-party nodes versus a dedicated SQL custom node.
- Whether long-running execution should eventually move from in-process Tokio tasks to coordinator/worker process boundaries.
- Where semantic model crates should enter the workspace and how early to reserve API space for them.
- Exact workspace lease semantics for editing ownership versus runtime ownership.
- Whether background mode keeps edit ownership, runtime ownership, both, or a configurable subset.
