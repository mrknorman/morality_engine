# UI Architecture Contract

This contract defines strict boundaries for UI code so features remain composable, owner-scoped, and safe under ECS query rules.

## 1. Module Boundaries

### `src/systems/ui/*` (primitives)
- Contains reusable, domain-agnostic UI primitives.
- Allowed responsibilities:
  - Generic UI state/resources.
  - Generic interaction/navigation behavior.
  - Generic layer/dropdown/tab/selector mechanics.
- Must not depend on:
  - Scene-specific enums or menu page/domain logic.
  - Video-settings-specific data structures.

### `src/systems/ui/menu/*` (composition/policy)
- Composes primitives into concrete menu UX and routes domain commands.
- Allowed responsibilities:
  - Menu page composition.
  - Mapping primitive outputs to domain commands.
  - Wiring visual styles/config values.
- Must not re-implement primitive behavior that already exists in `systems/ui`.

### `src/scenes/*` (usage only)
- Uses menu composition APIs or reusable primitives.
- Must not duplicate menu engines or custom selection stacks when shared composition exists.

## 2. Owner and Layer Scope Rules

Every layered UI element must resolve through an explicit owner/root identity.

- `UiLayer { owner, kind }` is the canonical layer identity model.
- Layer behavior (`Base`, `Dropdown`, `Modal`) must be resolved by owner, not globally.
- Interaction gating must be owner-scoped:
  - One owner's modal/capture must not block unrelated owners.
- Dropdown/tab focus state must be keyed by owner/root when state can coexist.

## 3. Primitive Contract Rules

### Primitive Construction Style (Required Components + Insert Hooks)
- New UI primitives must not use Bundle-first construction APIs as their primary integration path.
- For reusable UI primitives, use:
  - A root component.
  - `#[require(...)]` for mandatory contracts.
  - `#[component(on_insert = ...)]` to spawn and wire internal hierarchy.
- Adding the root primitive component should produce a functional unit without callers manually spawning hidden internal child contracts.
- Primitive internals (selection roots, clickables, layer markers, state resources, visuals) should be owned and wired by the primitive itself.
- Existing Bundle-based helpers may remain temporarily for compatibility, but are migration targets and should not be expanded.

### Self-Contained Primitive Rule
- Each primitive should be independently operable in any parent context (`menu`, `window`, `scene`), provided owner/layer gates are supplied.
- Composition modules should configure primitives, not re-implement primitive internals.
- If a primitive cannot be fully self-contained, document the minimal external contract explicitly near the primitive and in UI docs.

### Interaction Source of Truth
- Behavioral truth must come from interaction primitives:
  - Hover: `Hoverable.hovered`
  - Pointer/select activation: `Clickable<T>.triggered`
  - Keyboard activation: `Pressable<T>.triggered_mapping`
  - Selection/focus arbitration: `SelectableMenu.selected_index` and `SelectableMenu.keyboard_locked`
  - Cycling intent/bounds: `OptionCycler`
- `InteractionVisualState` is visual-only derived state and must not be used as behavioral input.
- Any new interaction system must consume primitives above and only write visual state for rendering feedback.

### Dropdowns
- Generic open/close/single-visible/outside-click logic lives in `systems/ui/dropdown`.
- Composition modules may only provide:
  - Anchoring/layout.
  - Domain value mapping.
  - Command dispatch.

### Tabs
- Tab selection, focus, keyboard/mouse arbitration, and activation semantics should come from one reusable primitive path.
- Feature modules may only map tab index to content/commands.

### Selectables/Cyclers
- `SelectableMenu` + `Selectable` control selection.
- `OptionCycler` handles left/right trigger intent.
- Feature modules consume intent; they should not duplicate core intent generation logic.
- Feature modules must not rely on `InteractionVisualState.selected/pressed/hovered` for commands.

## 4. Dependency Direction

Allowed direction:
1. `systems/ui` -> may depend on `systems/interaction` types and generic engine data.
2. `systems/ui/menu` -> may depend on `systems/ui` and `systems/interaction`.
3. `startup/*` and `scenes/*` -> may depend on `systems/ui/menu` and `systems/ui` public APIs.

Disallowed direction:
- `systems/ui` -> `systems/ui/menu` or `scenes/*`.
- `systems/ui` -> domain-specific menu/video command enums.

## 5. Query-Safety Rules (B0001 prevention)

When one system touches related component sets:

1. Use `ParamSet` whenever mutable and immutable access to the same component type can overlap in one system.
2. Use `Without<T>` filters to enforce disjoint queries where possible.
3. Add a short inline comment for each disjointness contract.
4. Avoid multiple mutable queries over overlapping archetypes without strict filters.
5. For layer-resolution + visibility mutation in one system:
   - Resolve active layers with read query first.
   - Mutate visibility with separate `ParamSet` query second.

## 6. Input Arbitration Policy

When multiple input sources can affect selection:

Priority order is fixed:
1. Active layer gate.
2. Focus group owner (e.g. tabs vs options vs modal).
3. Keyboard lock (until mouse movement).
4. Hover fallback.

No "first matching query entity wins" behavior for core navigation decisions.
Do not substitute `InteractionVisualState` flags for this arbitration order.

## 7. Composition and Extension Rules

To add new UI behavior:
1. Extend a primitive in `systems/ui` if behavior is reusable.
2. Compose it in `systems/ui/menu` (or other UI composition modules) with domain wiring.
3. Keep scene modules as consumers of composition APIs.

If a feature cannot be primitive-backed, document why in-code near the system.

## 8. Acceptance Criteria for This Contract

1. Multiple layered UI owners can coexist without interaction cross-talk.
2. Dropdown/tab/selectable behavior is primitive-driven and reusable.
3. Menu composition contains policy, not duplicated engine logic.
4. Query conflicts are explicitly prevented by design (`ParamSet`/`Without`).
