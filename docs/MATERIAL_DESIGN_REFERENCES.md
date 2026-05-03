# Material Design References

Data Engine Studio uses Material Design as a reference language, not as a drop-in web
implementation. The UI document system should keep its own deterministic style model while
borrowing well-defined Material ideas such as elevation, shape, motion, typography, and color
roles.

## Canonical References

- Material Design 3: <https://m3.material.io/>
- Elevation overview: <https://m3.material.io/styles/elevation/overview>
- Material Web elevation docs: <https://material-web.dev/components/elevation/>
- Material Web source: <https://github.com/material-components/material-web>
- Material Web elevation implementation: <https://github.com/material-components/material-web/blob/main/elevation/internal/_elevation.scss>
- Material 3 Design Kit: <https://www.figma.com/community/file/1035203688168086460/material-3-design-kit>

## Elevation

Material Web exposes elevation as a small level scale from `0` through `5`. That maps well to
our style model because product UI can ask for an elevation intent instead of hand-tuning
box shadows for every card, menu, overlay, and drag surface.

Material Web renders elevation with two conceptual shadow layers:

- Key shadow: tighter and more directional.
- Ambient shadow: softer and more diffuse.

In Data Engine Studio, keep `z-index` and elevation separate:

- `z-index` controls paint and hit-test stacking.
- Elevation controls perceived depth through shadow and, later, optional surface treatment.

This lets a menu paint above a card with `z-index` while still using a modest elevation recipe,
or a dragged item use both high `z-index` and a stronger elevation.

## Local Interpretation

The first local elevation recipes should stay boring and inspectable:

- Level 0: no shadow.
- Level 1: low resting surface.
- Level 2: raised resting surface.
- Level 3: modal/menu/card emphasis.
- Level 4: hover or focused promotion.
- Level 5: dragged or active floating surface.

Prefer adding reusable token helpers before adding one-off shadow literals to lab or product UI.
