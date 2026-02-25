# Dilemma Visual Profiles

This document defines how dilemma backgrounds and ambient scene FX are now authored.

## Goals

- Allow per-dilemma background sets through content JSON.
- Keep visuals data-driven for easy content iteration.
- Scale atmosphere by branch depth (intensity) without code edits.

## Authoring Surface

### 1) Per-dilemma selection

Each dilemma JSON can now include:

```json
"visuals": {
  "profile": "apocalypse",
  "intensity": 3
}
```

Location: `src/scenes/dilemma/content/**.json`

If omitted, the runtime defaults to:

```json
{
  "profile": "desert_default",
  "intensity": 0
}
```

### 2) Profile catalog

Profiles are defined in:

`src/scenes/dilemma/content/visual_profiles.json`

Each profile contains:

- `background_layers`: layered background sets (independent of intensity for apocalypse).
- `ambient_smoke`: optional animated smoke plume settings.
- `ambient_viscera`: optional static clutter using real in-game explosion types.

## Background Sets

Background glyph sets are in:

- `src/systems/backgrounds/content/desert.json`
- `src/systems/backgrounds/content/apocalypse.json`

`BackgroundTypes` now supports:

- `desert`
- `apocalypse`

## Runtime Behavior

- Dilemma setup resolves `visuals.profile` + `visuals.intensity` into a spawn plan.
- Unknown profile IDs log a warning and fall back to `desert_default`.
- Layer density is clamped to `>= 0`.
- In the apocalypse profile, intensity does not increase background clutter.
- Ambient smoke uses `steam_train` rising smoke frames (same frame set as wreck smoke), but without spawning wrecked trains.
- Smoke count scales with intensity via:
  `base_count + (count_per_intensity * intensity)`.
- Ambient body-part clutter uses the actual `ExplodedGlyph` component type.
- Ambient blood clutter uses the actual `BloodSprite` component type.
- Viscera counts scale with intensity via:
  `base + (per_intensity * intensity)` for both body parts and blood.
- Ambient elements are tagged as background ambience and excluded from
  gameplay viscera transition overrides, so they keep background-driven
  speed behavior through phase transitions.

## Psychopath Path Mapping

Current wiring:

- `path_psychopath/one_or_two.json` -> `apocalypse`, intensity `1`
- `path_psychopath/death_at_a_convent.json` -> `apocalypse`, intensity `2`
- `path_psychopath/prolonged_suffering.json` -> `apocalypse`, intensity `3`
- `path_psychopath/train_of_mass_destruction.json` -> `apocalypse`, intensity `4`

This gives progressively:

- no layered desert/apocalypse parallax conflict (single apocalypse layer),
- more smoke plumes.
- more ambient body parts and blood particles.

## Validation and Tests

Unit tests for profile parsing and scaling are in:

`src/scenes/dilemma/visuals.rs`

Run:

```bash
cargo test scenes::dilemma::visuals::tests
```

## Extending

To add a new visual theme:

1. Add a glyph set JSON under `src/systems/backgrounds/content/`.
2. Add a variant in `BackgroundTypes`.
3. Add a profile entry in `visual_profiles.json`.
4. Reference it from dilemma JSON via `"visuals": { "profile": "...", "intensity": N }`.
