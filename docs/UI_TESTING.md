# UI Testing Plan

Data Engine Studio's UI runtime should be tested like a small browser engine:
most proof belongs in deterministic runtime assertions, while graphical tests
verify that the rendered result still matches the contract.

## Test Layers

1. Runtime assertion tests

   Test the product UI runtime without egui. These tests own layout, style
   resolution, selector precedence, box model math, hit testing, scroll
   ownership, retained state, and animation interpolation.

2. Runtime reference tests

   Compare two independently built scenes that should resolve to equivalent
   layout frames or visual output. The reference scene should avoid the exact
   feature under test when possible.

3. Graphical kittest tests

   Render egui-hosted scenes through `egui_kittest` and compare images. Use
   these for primitives where visual regressions are meaningful: clipping,
   borders, scrollbars, transitions, z-order, graph connectors, and focus
   affordances.

4. Interaction tests

   Drive pointer, scroll, keyboard, drag, and focus input through the host
   adapter and assert runtime state changes. When useful, render the final
   frame and compare it to a directly seeded scene.

5. Screenshot harness checks

   Use `scripts/capture-ui.ps1` for human-facing iteration and larger app
   screenshots. These are exploratory unless paired with a machine assertion.

## Current Commands

```powershell
just ui-test
cargo test -p des-ui-runtime
cargo test -p des-ui-egui
.\scripts\capture-ui.ps1 -Out target\ui-shots\studio.png -Width 1320 -Height 780
```

## Graphical Comparison Rules

- Prefer exact image comparisons only when the same scene is rendered twice in
  the same process.
- Use tolerances for cross-machine baselines, anti-aliased geometry, font
  rendering, or GPU-sensitive output.
- Keep visual tests narrow. A diff should point to one broken primitive, not an
  entire application redesign.
- Pair interaction-image tests with direct-state assertions so failures explain
  whether input routing or painting changed.
- Store generated artifacts under `target/`; do not commit local render output
  unless it is an intentional baseline.

## Box Model Decision

The runtime uses a CSS-inspired box model, but it does not aim for one-to-one
CSS compatibility. `width` and `height` mean the final painted frame size:
padding and border are inside that frame. This is equivalent to a border-box
mental model and should remain the default because Studio is a dense product UI
where "a 320px card" should visually occupy 320px.

Margin remains outside the painted box. It affects layout footprint and sibling
placement, but it is not included in `LayoutFrame::rect.size` and is not
painted as part of the element.

Auto sizing measures children plus padding and border. Fill and percent sizing
resolve against the parent content rect after parent padding and border are
removed. These contracts should be represented in both runtime assertions and
the UI lab specimen view.

## Browser-Inspired Test Types

- `testharness` equivalent: Rust assertion tests over runtime state.
- `reftest` equivalent: compare an implementation scene to an independent
  reference scene.
- pixel/visual equivalent: compare rendered images when structural assertions
  cannot catch the regression.
- manual equivalent: the UI lab and screenshot harness, used for exploration
  before a behavior graduates into automated tests.
