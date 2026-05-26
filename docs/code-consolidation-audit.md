# Code Consolidation Audit

This audit is for the post-roadmap codebase after the long AI-assisted
implementation run on `des-document`. Its purpose is not to find every possible
cleanup. It is to identify places where simplicity should beat API breadth,
especially where the code shows signs of permutation growth.

## Audit Position

The current direction is usable, but the codebase should stay in consolidation
mode until the remaining helper matrices are reduced.

The risk pattern is clear:

- many commits added parity helpers for action values, dispatch, CSS, widgets,
  projections, and HTML bundles;
- later commits removed large slices of those helpers;
- tests still contain areas whose main effect is proving that synonymous helper
  paths exist.

Treat that as a signal. New helpers should be rare, and old helpers should be
kept only when they are materially better than the canonical workflow.

## Simplicity Rules

Use these rules before adding or preserving any public helper:

- Prefer one canonical workflow plus readable chaining over method-name
  permutations.
- Keep direct shortcuts only for the most common two-step app loops.
- Do not add both frame-returning and value-only variants unless the value-only
  path removes substantial ceremony.
- Do not add both strict-CSS and forgiving-CSS variants at every layer.
- Do not mirror helper families from `DocumentView` into `DocumentActionSurface`,
  widgets, HTML documents, HTML bundles, and named HTML sets.
- A test should prove behavior or a canonical workflow, not the existence of
  every synonym.
- If a helper mostly calls another helper and changes only one axis, prefer
  deleting it.

## Hotspots

### 1. `ContextMenu` Convenience Matrix

File: `crates/des-document/widgets/src/context_menu.rs`

`ContextMenu` still exposes a broad matrix across:

- `action_for` closures vs `(command, action)` pairs
- default stylesheet vs explicit stylesheet vs strict CSS vs forgiving CSS
- update vs input update
- action frame vs action values
- dispatch action objects vs dispatch action values
- fallible vs infallible construction

This is now the strongest remaining permutation-loop smell. Many methods are
thin wrappers around `try_action_surface_with_stylesheet*` followed by
`update_*` on `DocumentActionSurface`.

Recommended direction:

- Keep menu construction and command-registry helpers.
- Keep one or two action-surface constructors:
  `try_action_surface_with_stylesheet` and
  `try_action_surface_with_stylesheet_and_actions`.
- Prefer callers using the returned `DocumentActionSurface` for update, input,
  dispatch, and value extraction.
- Remove direct `update_action_values*` and
  `update_with_input_and_dispatch*` families from `ContextMenu` unless a real
  app call site proves they save meaningful code.

Why simplicity wins:

The menu should define reusable behavior. It should not become a second action
surface API.

### 2. `DocumentActionWidget` Dispatch/CSS Parity

File: `crates/des-document/core/src/document.rs`

`DocumentActionWidget` has default methods for many combinations of:

- update vs update with input
- dispatch vs dispatch action values
- strict CSS vs forgiving CSS
- fallible vs infallible

These defaults are convenient but make every widget inherit a very wide public
surface. They also duplicate capabilities available through
`action_surface(...)`, `try_action_surface(...)`, and the fluent update request
on the resulting surface.

Recommended direction:

- Keep declaration methods:
  `command_binding`, `command_bindings`, `push_commands`, `commands`, and
  `command_registry`.
- Keep action-surface constructors as the official integration convention.
- Strongly consider removing direct dispatch and action-value update helpers
  from the trait.
- If compatibility is not a priority, make widget users explicitly choose:
  construct a surface, then update or dispatch it.

Why simplicity wins:

The trait should describe what a widget contributes to a document. It should
not also reproduce every possible document update loop.

### 3. Legacy `DocumentView` Projection/Text-Measurer/Widget Combinations

File: `crates/des-document/core/src/view.rs`

The newer request APIs give a clear path for uncommon combinations:

```rust
view.update_request()
    .projection(projection)
    .input(input)
    .dispatch(&registry, handler)
```

However, `DocumentView` and `DocumentActionSurface` still expose older direct
families such as:

- `project_and_update_with_input_and_dispatch*`
- `project_with_and_update_with_input_and_dispatch*`
- `project_and_update_with_input_and_text_measurer_and_dispatch*`
- `project_widget(s)_and_update_with_input_and_dispatch*`

Recommended direction:

- Keep short, common methods:
  `update`, `update_with_input`, `update_actions`, `project`, and perhaps
  `project_and_update`.
- Route projection plus input plus dispatch through `DocumentUpdateRequest` or
  `DocumentActionSurfaceUpdateRequest`.
- Treat text-measurer and widget-projection combinations as advanced paths and
  collapse them behind the request model or explicit small-step calls.

Why simplicity wins:

The request builder exists specifically to avoid encoding every combination in
method names.

### 4. HTML Update and Action Helper Breadth

File: `crates/des-document/html/src/lib.rs`

Recent consolidation removed the worst direct dispatch permutations from
`HtmlDocument` and `HtmlSet`, but `HtmlDocument`, `HtmlStylesheet`, and
`HtmlSet` still expose many action mapping shortcuts:

- `update_actions_with_actions`
- `update_action_values_with_actions`
- `update_actions_with_intent_actions`
- `update_action_values_with_intent_actions`
- CSS and forgiving-CSS variants
- named-document variants

Recommended direction:

- Keep parsing and compilation front doors:
  `HtmlDocument`, `HtmlStylesheet`, `HtmlSet`, `to_view`, and
  `to_action_surface`.
- Keep one mapped action-surface route for common app loops.
- Prefer `DocumentCommandRegistry` plus `DocumentView::update_request()` for
  uncommon combinations.
- Continue removing value-only and named-set aliases where tests can express
  the same behavior through `get(name)?.to_view(...)?`.

Why simplicity wins:

HTML should compile into document structures. Once it has produced a view or
surface, the document API should carry the app loop.

### 5. Breadth Tests That Lock In Synonyms

Files:

- `crates/des-document/core/tests/document_contracts.rs`
- `crates/des-document/html/tests/html_contracts.rs`
- `crates/des-document/widgets/src/context_menu.rs`

The tests are valuable, but some still prove helper breadth more than product
behavior. Examples include blocks that exercise every combination of direct
dispatch, action values, mapped actions, CSS, forgiving CSS, and named set
variants.

Recommended direction:

- Keep adversarial semantic tests.
- Keep one canonical workflow test per layer.
- Convert breadth tests into smaller behavior tests:
  parse -> view/surface -> projection/input -> output/action assertions.
- Delete tests whose only unique assertion is “this alias exists.”

Why simplicity wins:

Tests should make refactoring safer. They should not make redundant APIs feel
mandatory.

## Candidate Refactor Order

1. Collapse `ContextMenu` direct update/dispatch helpers.
   This is the clearest remaining widget-specific permutation matrix and is out
   of the critical `des-document` core.

2. Slim `DocumentActionWidget` to declaration plus surface construction.
   This clarifies the official widget integration convention.

3. Remove advanced direct dispatch permutations from `DocumentView` and
   `DocumentActionSurface`.
   Keep the request builder as the canonical answer for complex updates.

4. Continue trimming HTML action-value and named-set aliases.
   Do this after core/widget surfaces settle so HTML does not chase a moving
   target.

5. Rewrite breadth tests into workflow/adversarial tests.
   Pair each API deletion with test rewrites that preserve behavioral coverage.

## What Not To Refactor Yet

- Do not refactor vendored layout internals just because files are large.
  The layout crate is derived from Taffy and should be treated differently from
  app-authored API surfaces.
- Do not chase widget implementation polish while the surface contract is still
  being reduced.
- Do not split crates purely to make files smaller.
- Do not add compatibility shims for deleted helper names unless a real app
  caller needs them.

## Mechanical Signals To Recheck

Useful commands:

```sh
git log --oneline --since='24 hours ago' --stat -- crates/des-document crates/des-app crates/des-ui-lab crates/des-core
find crates/des-document crates/des-app crates/des-ui-lab crates/des-core -name '*.rs' -print0 | xargs -0 wc -l | sort -n | tail -40
rg "pub fn .*(_if|_with|_and_|action_values|dispatch|forgiving|projection|projected)" crates/des-document/core/src crates/des-document/html/src crates/des-document/widgets/src -n
rg "helpers|directly|one_front_door|with_css|forgiving|bundle|set_manages" crates/des-document/core/tests crates/des-document/html/tests crates/des-document/widgets/src -n
```

Use these as smoke detectors, not as automatic deletion instructions.

## Completion Bar

The consolidation pass is done when:

- the main document loop can be explained without listing method families;
- widgets contribute structure, styles, projections, and commands without
  re-exporting every update permutation;
- HTML compiles into document views/surfaces and then gets out of the way;
- tests focus on semantic behavior and canonical workflows;
- adding a new axis, such as another input mode or stylesheet mode, does not
  require adding another cross-product of helper methods.
