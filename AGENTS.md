# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
cargo build              # Debug build (deps optimized at O3, game code at O0)
cargo run                # Run the game
cargo build --release    # Release build (LTO, O3, single codegen unit)
cargo run --release      # Run optimized build
cargo check              # Fast type-check without producing binary
```

No test suite exists. No linter or formatter configuration is set up.

## Project Overview

"The Trolley Algorithm" — a Bevy 0.18 game exploring moral dilemmas (trolley problem variants). The player works through lab calibration scenarios that branch into philosophical paths (Utilitarian, Deontological, Inaction). Uses `#[forbid(unsafe_code)]`.

## Architecture

### Plugin Hierarchy

The app composes two top-level plugins from `GamePlugin`:

- **`StartupPlugin`** — engine infrastructure: post-processing renderer (CRT scan lines + bloom), custom cursor, RNG, resize handling, state machine, stats, common UI, motion, colors, inheritance, particles (`bevy_hanabi`)
- **`ScenePlugin`** — game scenes: Menu, Loading, Dialogue, Dilemma (with 6 sub-phases), Ending

Scenes dynamically add further plugins (Train, Person, Lever, Junction, Background, Cascade, Audio, Interaction, Physics, Text, etc.).

### State Machine

```
MainState { Menu, InGame }
  └─ GameState (sub-state of InGame) { Loading, Dialogue, Dilemma, Ending }
       └─ DilemmaPhase (sub-state of Dilemma) { Intro, Decision, DilemmaTransition, Skip, Consequence, Results }
```

A `SceneQueue` resource (`VecDeque<Scene>`) drives scene progression. Each scene's `OnEnter` system spawns its entities.

### Content System

All game content (dilemmas, dialogue trees, train configs, backgrounds) is **embedded at compile time** via `include_str!()` and deserialized with `serde_json`. Content lives in JSON files alongside their Rust modules in `src/scenes/*/content/`. Dilemma and dialogue variants are defined via macros (`define_dilemma!`, `define_dialogue!`) that generate enums mapping variants to JSON file paths.

### Key Patterns

- **Component hooks (`on_insert`)**: Entities self-initialize when components are inserted (e.g., `Train::on_insert` loads JSON and spawns audio/animation children, `PersonSprite::on_insert` creates ASCII glyph art)
- **Generic typed interactions**: `Clickable<T>` and `Pressable<T>` are generic over action enums. The `register_interaction_systems!` macro generates all action handling per type
- **Generic audio pallets**: `ContinuousAudioPallet<T>` and `OneShotAudioPallet<T>` use `enum-map` for type-safe sound selection per scene
- **Time dilation**: A global `Dilation` resource scales physics, audio playback speed, and animations consistently
- **Color inheritance**: `BequeathTextColor`/`BequeathTextAlpha` propagate parent visual properties to children
- **Conditional timers**: `TimerPallet<K>` supports timers that start immediately or wait for `NarrationAudioFinished`

### Post-Processing Pipeline

`RenderPlugin` (in `startup/render.rs`) sets up a two-camera system: an off-screen camera (layer 0) renders to a texture, then a screen-space camera (layer 1) displays it through a `ScanLinesMaterial` shader with HDR bloom.

### Key Modules

- `src/data/` — game state types: `GlobalRng` (PCG + Perlin), `GameStats`/`DilemmaStats`, `MainState`/`GameState`/`DilemmaPhase`, `Character`
- `src/entities/` — entity definitions with spawn logic: Train, Person, Text, Sprites, Track, Graph, LargeFonts
- `src/scenes/` — scene content and phase systems, each with a `content/` subdirectory holding JSON data
- `src/systems/` — reusable ECS systems: audio, cascade, colors, interaction, motion, physics, backgrounds, inheritance, time dilation, scheduling, particles, resize
- `src/startup/` — engine setup: render pipeline, cursor, textures, keyboard shortcuts, RNG seeding
- `src/style/` — UI positioning: `BottomAnchor` for responsive layout, `unique_element!` macro for singleton UI components (NextButton, CenterLever, DilemmaTimerPosition)
- `src/shaders/` — WGSL material definitions (pulsing alpha shader)
- `assets/shaders/` — GPU shaders (scan lines post-processing)
- `assets/audio/` — sound effects, music, narration audio files
