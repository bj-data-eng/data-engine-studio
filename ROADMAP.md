# Data Engine Studio Roadmap

Data Engine Studio is a Rust-first desktop studio for visual ETL, data
exploration, and future semantic modeling. Rust owns application behavior,
Python remains a thin PyPI/process entry layer, and `egui` is the native host
adapter rather than the owner of product layout semantics.

This roadmap is currently centered on `des-document`. The audit found that the
core direction is sound, but the public authoring surface is approaching the
point where more helper abstraction would be counterproductive. The next phase
should harden semantics and consolidate front doors before adding new broad API
families.

## Current Verdict

Pause broad abstraction. Consolidate and harden.

What is working:

- `des-document` is egui-free, typed, testable, and usable through Rust-authored
  document trees.
- HTML/CSS ingestion is a credible first-class authoring path.
- Behavior hooks, document events, typed commands, projection, retained state,
  and action collection form a coherent app loop.
- `DocumentWidget` and `DocumentActionWidget` are the right integration seam for
  reusable document-backed behavior.

What is risky:

- Public APIs have grown combinatorially across event intent, projection,
  update, dispatch, action-value, HTML, CSS, and named-document
  variants.
- `DocumentOutput` and `DocumentInteractionState` now answer many of the same
  questions through separate public surfaces.
- Some tests lock in helper breadth instead of only proving canonical workflows.
- A few adversarial paths can misroute commands or emit surprising lifecycle
  events.

Roadmap rule for the next phase: add a public helper only when it clearly
removes real app authoring ceremony and cannot be expressed cleanly through an
existing canonical front door.

## Scope For This Phase

In scope:

- `crates/des-document/core`
- `crates/des-document/html`
- document contracts and HTML contracts
- the generic widget integration contracts in core

Out of scope for now:

- implementation details of the current widgets themselves
- expanding context menu, sortable, or autoscroll APIs
- new widget behavior abstractions unless a core contract is demonstrably
  missing

The widget crates should continue to compile and pass tests, but the immediate
roadmap should not chase widget-specific polish until the core and HTML
contracts are tightened.

## Architecture Principles

- Keep `des-document` and `des-widgets` independent from `egui` and Python.
- Export intentional front-door APIs from `lib.rs`; keep internals private by
  default.
- Prefer typed commands, events, snapshots, reports, and change sets over shared
  mutable state.
- Use deterministic browser-inspired HTML/CSS concepts without recreating
  JavaScript or CSS specificity.
- Treat HTML/CSS as compilable authoring input into fast Rust structures, not as
  the runtime platform.
- Favor one or two canonical app workflows over many method permutations.
- If a small behavior change requires touching many public helper families,
  revisit the abstraction before continuing.

## Target Document Loop

The product-grade loop should be explainable in one paragraph:

```text
author structure + hooks + classes
compose styles
project app state into retained document state
route host input through DocumentInput
read DocumentOutput / DocumentInteractionState
map authored commands into typed app actions
```

The canonical Rust flow should remain:

```text
DocumentView / DocumentActionSurface
  -> DocumentProjection
  -> DocumentInput
  -> DocumentOutput / DocumentInteractionState
  -> DocumentCommandRegistry<Action>
  -> DocumentActionFrame<Action>
```

The canonical HTML flow should remain:

```text
HtmlDocument / HtmlStylesheet
  -> Document / DocumentView
  -> DocumentProjection
  -> DocumentInput
  -> typed Rust actions through behavior hooks
```

## Immediate Hardening Priorities

### 1. Fix HTML command attribute grammar

Problem: HTML ingestion currently treats any attribute beginning with
`data-command` as a behavior hook. Metadata such as
`data-command-id="delete.project"` can accidentally become a default click
command.

Required outcome:

- Accept exactly `data-command` and `data-command:<event>`.
- Preserve unrelated `data-command-*` attributes as ordinary attributes.
- Add adversarial contract tests proving metadata does not emit commands.

### 2. Define full-document HTML body semantics

Problem: full HTML parsing can leak `head`, `title`, `meta`, `link`, or `style`
metadata into the retained document tree because unknown tags fall back to
generic document elements.

Required outcome:

- Decide whether `HtmlDocument::parse` projects only the body subtree or
  explicitly ignores non-body metadata.
- Add tests for full documents with head/title/style/meta/link.
- Keep `parse_fragment` behavior separate and predictable.

### 3. Tighten JavaScript event attribute filtering

Problem: the current JavaScript-event detection treats many `on*` attributes as
JavaScript handlers. Legitimate attributes such as `once`, `onboarding`, or
domain-specific metadata can be dropped.

Required outcome:

- Match a known HTML event-handler attribute set such as `onclick`, `oninput`,
  `onkeydown`, etc.
- Preserve unrelated `on*` attributes.
- Keep `on:<event>` as the Rust behavior hook syntax.

### 4. Normalize command names consistently

Problem: hook commands and command object predicates trim whitespace, but all
registry binding and output helper paths must agree.

Required outcome:

- Normalize command names at construction for hooks and bindings.
- Ensure `has_command`, `has_command_kind`, `has_command_intent`, binding lookup,
  and dispatch agree with `command_is`.
- Add tests with padded command input at every public matching layer.

### 5. Define text selection lifecycle

Problem: retained inactive selections can restart without `SelectionStarted`, or
clearing an already-ended selection can emit duplicate `SelectionEnded`.

Required outcome:

- Specify lifecycle semantics for selection start, change, end, retained
  inactive selection, restart, and clear.
- Add tests for select -> release -> select again.
- Add tests for select -> release -> click elsewhere, with no duplicate end
  event unless explicitly intended.

### 6. Define focus semantics

Problem: multiple focused elements can exist, but focus event tracking observes a
single target.

Required outcome:

- Decide whether focus is singular or multi-focus is legal.
- If singular, make focusing one element clear peers or reject ambiguity.
- If multi-focus is legal, expose and test multi-focus events explicitly.
- Define what happens when a focused element is removed: no blur, blur without
  command dispatch, or retained hook dispatch.

### 7. Clarify scroll-only input

Problem: `DocumentInput::scroll` creates scroll input without pointer position,
but input routing may ignore scroll when no pointer is present.

Required outcome:

- Decide the contract: no-op by design, focused/root scroll, or require targeted
  scroll helpers.
- Add a contract test so adapters do not infer different behavior.

### 8. Preserve mixed-content HTML whitespace intentionally

Problem: mixed content such as `Hello <span>world</span>` can lose boundary
spaces because standalone text nodes are trimmed while text-only elements are
not.

Required outcome:

- Define a simple HTML whitespace policy for mixed inline content.
- Add tests for leading, trailing, and inter-element spaces.
- Keep semantic text and layout text behavior compatible with existing text
  normalization.

## Consolidation Priorities

### 1. Stop expanding per-intent helper families

The generic APIs already cover event intent and target matching. Treat helpers
such as `contains_clicked_action`, `dispatch_focus`, or
`collect_select_action_values` as convenience only. Do not add more families
until the canonical query model is settled.

Preferred canonical shape:

```rust
frame.actions_for_intent(ElementBehaviorEvent::Click)
frame.contains_action_for_target_intent("run", ElementBehaviorEvent::Click, &Action::Run)
frame.dispatch_intent(ElementBehaviorEvent::Click, handler)
```

### 2. Choose the canonical frame query surface

`DocumentOutput` and `DocumentInteractionState` now overlap heavily. Pick one:

- Make `DocumentInteractionState` the app-facing frame query facade and keep
  `DocumentOutput` lower-level, or
- Keep `DocumentOutput` canonical and shrink `DocumentInteractionState` to a
  small curated summary.

Do not continue mirroring every method both ways.

### 3. Reduce update/projection method permutations

`DocumentView`, `DocumentActionSurface`, `HtmlDocument`, `HtmlStylesheet`, and
`HtmlSet` expose many combinations of:

- no CSS vs CSS
- input vs no input
- projection vs no projection
- registry vs mapped actions
- dispatch vs action values
- single widget vs many widgets

Preferred direction:

- Keep a few common shortcuts.
- Introduce one fluent request/config builder for uncommon combinations.
- Avoid adding new `update_with_input_and_X_and_Y_and_Z` methods.

### 4. Trim or split preludes

The current prelude is close to “export everything.” Consider:

- `des_document::prelude` for app authoring essentials.
- Direct crate-root imports for specialized layout/style/text primitives.
- Optional narrower preludes later only if real call sites justify them.

### 5. Rewrite breadth tests into workflow tests

Tests should prove behavior and canonical workflows:

```text
author hook -> update -> inspect interaction -> dispatch typed action
```

Avoid tests whose primary purpose is proving that every synonym exists. Those
tests make future consolidation expensive.

## Public API Expectations

The intentional front door should emphasize:

- `Document`
- `DocumentView`
- `DocumentActionSurface`
- `DocumentProjection`
- `ElementSpec`
- `ElementBehaviorEvent`
- `StyleSheet`
- `DocumentInput`
- `DocumentOutput`
- `DocumentInteractionState`
- `DocumentCommandRegistry`
- `DocumentActionFrame`
- `DocumentWidget`
- `DocumentActionWidget`
- `HtmlDocument`
- `HtmlStylesheet`
- `HtmlSet`

Everything else should be exported only when it is a real extension point or a
necessary typed data structure for app code.

## Verification Gates

Before considering `des-document` product-grade:

- `cargo fmt --all -- --check`
- `cargo test -p des-document --tests`
- `cargo test -p des-html --tests`
- `cargo test -p des-widgets --tests`
- targeted adversarial tests for each hardening priority above

Green tests are not enough. Completion requires that the public surface can be
explained without listing dozens of method permutations and that adversarial
HTML/input/selection/focus cases have explicit contracts.

## Non-Document Roadmap Holding Pattern

The broader Studio roadmap still includes graph, project, runtime, execution,
Python packaging, and UI lab work, but those areas should not drive new
`des-document` abstraction until the document authoring surface is consolidated.

The stable outer direction remains:

```text
python -> des-python -> des-ui-lab -> des-app -> domain/service crates -> des-core
                          \-> des-document/egui -> des-document/widgets -> des-document/core
                          \-> des-document/html
```

Domain logic must remain independent of `egui` and Python. The document layer
should provide reusable structure, style, event, projection, and command
contracts that Studio can depend on without becoming a second application
framework inside the app.
