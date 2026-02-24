# Project Backlog

Last updated: 2026-02-24

This backlog tracks actionable work. Raw brainstorming lives in
`docs/idea_inbox.md`.

## Working Rules

- Keep source files free of large planning lists.
- Use inline code comments only for local debt with IDs: `TODO(SCN-###): ...`.
- Every backlog item must include acceptance criteria.
- Move ideas from inbox to backlog only when they are implementation-ready.

## Now

### SCN-001 - Dialogue reliability and fade lifecycle

- Status: todo
- Priority: high
- Area: scenes/dialogue
- Why: dialogue stability is core to scene quality and currently has known robustness/fade issues.
Acceptance criteria:
- Chained dialogue transitions complete without stuck fade states.
- `Dialogue::new` and setup path return recoverable errors instead of hard panics in runtime flow.
- Manual check: run at least one chained dialogue sequence from menu to next scene without visual/audio desync.

### SCN-002 - Close remaining panic-driven scene flow paths

- Status: todo
- Priority: high
- Area: scenes/runtime, scenes/menu, content loaders
- Why: high-level review item #2 is only partially closed; remaining panic/expect paths are still brittle.
Acceptance criteria:
- Runtime scene advancement uses fallbacks/errors for empty/mismatched routes.
- Scene setup/content loading call sites avoid `panic!`/`expect(...)` in gameplay flow.
- Add/update tests for fallback behavior when invalid scene/content is encountered.

### SCN-003 - Interaction loop bug guardrails

- Status: todo
- Priority: high
- Area: systems/interaction
- Why: occasional interaction loops can soft-lock UX and undermine confidence in input systems.
Acceptance criteria:
- Repro case documented.
- Guard added to prevent repeated self-trigger loops per frame.
- Manual validation confirms no regression in clickable/pressable actions.

### SCN-004 - Debug + progression level selector

- Status: todo
- Priority: medium
- Area: scenes/menu, scene queue/runtime
- Why: content iteration is slower without controlled scene entry points.
Acceptance criteria:
- Debug mode supports direct jump to selected dilemma/dialogue.
- Player mode gates selection to unlocked/completed content.
- SceneQueue integration remains compatible with campaign and single-level flow modes.

### SCN-005 - Ending collection and restart UX

- Status: todo
- Priority: medium
- Area: scenes/ending, menu UI
- Why: repeat-play motivation and testability improve with visible ending progress and quick restart.
Acceptance criteria:
- Endings discovered are persisted in session state.
- Ending screen offers restart-to-menu and replay path.
- Manual run confirms no stale scene state after restart.

## Next

### SCN-006 - Audio direction pass for dilemmas

- Status: todo
- Priority: medium
- Area: systems/audio, scene content
- Why: current content calls out missing variation and special-case audio identity.
Acceptance criteria:
- Add at least one additional track family for dilemma variation.
- Special dilemma/audio cue mapping documented and test-played.
- Decision-phase music transition is intentional and consistent.

### SCN-007 - Dilemma readability polish pass

- Status: todo
- Priority: medium
- Area: scenes/dilemma, UI presentation
- Why: decision clarity and payoff can improve with stronger feedback.
Acceptance criteria:
- Results screen includes colored numeric emphasis.
- Hover/selector feedback is visually clear.
- Train interaction message path is implemented and validated.

### SCN-008 - Background placement system cleanup

- Status: todo
- Priority: medium
- Area: systems/backgrounds
- Why: current background placement precision is a known limitation.
Acceptance criteria:
- Introduce deterministic placement controls for background sprite composition.
- Existing scenes render without layout regressions at current target resolution.
- Authoring guidance updated for new placement knobs.

### SCN-009 - Cursor storytelling pack

- Status: todo
- Priority: medium
- Area: startup cursor, interaction UX
- Why: cursor behavior is a recurring design target and can carry narrative tone.
Acceptance criteria:
- Cursor states include idle, waiting, and pull intent.
- At least one animated cursor state validated in-game.
- Input affordances remain readable and performant.

### SCN-010 - Title/menu atmospheric polish

- Status: todo
- Priority: low
- Area: menu/title scene
- Why: title feedback and menu identity are valuable polish multipliers.
Acceptance criteria:
- Path-reactive title visual rule defined and implemented.
- CRT options color update applied consistently.
- No regressions in existing menu command actions.

## Later

### SCN-011 - Meta progression systems (hardware/software architect)

- Status: todo
- Priority: low
- Area: game design, menu/meta loop
- Why: this is a large product layer requiring data model and economy decisions.
Acceptance criteria:
- Initial design spec for stats/resources and unlock dependencies.
- Minimal ECS data model prototype for one subsystem.

### SCN-012 - Mode unlock framework

- Status: todo
- Priority: low
- Area: progression/state
- Why: multiple planned modes need a common unlock + routing mechanism.
Acceptance criteria:
- Unlock registry exists for Rampage/Sandbox/Lever Heaven placeholders.
- Menu visibility follows unlock state.

### SCN-013 - Trust and systemic-failure arc

- Status: todo
- Priority: low
- Area: narrative systems
- Why: trust is a strategic pillar but currently not represented as a coherent mechanic.
Acceptance criteria:
- Define trust variables and trigger events.
- Prototype one trust-driven branch transition.

### SCN-014 - Expanded ethical scenario schema

- Status: todo
- Priority: low
- Area: dilemma content schema
- Why: planned scenario families require richer participant and consequence modeling.
Acceptance criteria:
- Schema proposal for active/passive culpability variants and participant attributes.
- One sample scenario authored and loaded through existing content pipeline.

### SCN-015 - Persistent save/options/achievements baseline

- Status: todo
- Priority: low
- Area: platform UX
- Why: long-term retention and progression require persistence + options hygiene.
Acceptance criteria:
- Save/load contract defined.
- Volume controls in options menu.
- Achievement registry scaffold created.
