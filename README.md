# The Trolley Algorithm

A narrative dilemma game about trolley problems, built from scratch in Rust on [Bevy](https://bevyengine.org).

You are presented with a branching campaign of moral dilemmas — junctions, levers, and the consequences of pulling (or not pulling) them — held together by a narrator with opinions.

<!-- TODO: add a screenshot or GIF here, e.g. ![Morality Engine](docs/screenshot.png) -->

## Status

In active development since 2022. Expect rough edges.

## Building & running

Requires a recent stable Rust toolchain.

```bash
cargo run            # debug build (dependencies optimised, game code unoptimised)
cargo run --release  # optimised build (LTO, single codegen unit)
cargo check          # fast type-check
```

## How it works

The campaign is data-driven: dilemmas and their branching structure are defined in a JSON scene graph
(`src/scenes/flow/content/campaign_graph.json`) validated against a schema
(`src/scenes/flow/schema.rs`, `src/scenes/flow/validate.rs`), and routed by a single canonical
scene navigator (`src/scenes/runtime/`).

Everything else is custom ECS-driven systems on top of Bevy:

- **Scenes** — menu, loading, dialogue, dilemma (junction/lever mechanics), and ending flows
- **UI** — bespoke UI framework with architecture contracts and compliance matrices (see `docs/`)
- **Systems** — particles, physics, motion, cascades, scheduling, colour/inheritance, resize handling
- **Audio** — music and effects via `rodio`

## Documentation

Architecture notes live in [`docs/`](docs/):

- [`scene_architecture_contract.md`](docs/scene_architecture_contract.md) and [`ui_architecture_contract.md`](docs/ui_architecture_contract.md) — the rules the code is held to
- [`scene_flow_reference.md`](docs/scene_flow_reference.md) — how scene routing and campaign branching work
- An mdBook covering the UI architecture: `mdbook build docs`
