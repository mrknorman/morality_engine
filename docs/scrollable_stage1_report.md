# Scrollable RTT - Stage 1 Execution Report

Date: 2026-02-19  
Stage: `Phase 1: UI/Menu Pre-Cleanup and Hardening`  
Status: Completed

## Objective
Prepare the current UI/menu architecture for robust RTT scrolling integration without introducing scroll behavior yet.

## Work Completed

### 1. Primitive Boundary and Ownership Placement
Implemented:
- Added a dedicated scroll namespace module:
  - `src/systems/ui/scroll/mod.rs`
- Exposed it via:
  - `src/systems/ui/mod.rs`

Outcome:
- `Scrollable` and `ScrollBar` now have a reserved generic primitive boundary under `systems::ui::scroll`.
- Menu-specific behavior remains separated under `systems::ui::menu`.

### 2. Deterministic Integration Points in Menu Pipeline
Implemented scheduling contracts as comments in:
- `src/systems/ui/menu/mod.rs`

Defined integration points:
1. `Core` stage:
   - scroll input collection should occur after `InteractionSystem::Selectable`
   - before menu command mutation systems.
2. `Commands` stage:
   - scroll click/drag command handling belongs with direct UI command handlers.
3. `PostCommands` stage:
   - scroll reducer/focus-follow belongs after base command mutations settle.
4. `Visual` stage:
   - scroll viewport/scrollbar visuals should render after all logical state settles.

Outcome:
- The pipeline insertion strategy is now explicit and consistent with existing deterministic ordering goals.

### 3. Query-Safety and Owner-Scoped Contract Confirmation
Confirmed against current architecture:
- Layer arbitration remains owner-scoped via `UiLayer` resolution.
- Interaction gating remains owner-aware through `InteractionGate`.
- Existing menu command systems already use disjoint query patterns (`ParamSet`, `Without`) where needed.

Stage 1 rule locked:
- No global singleton scroll state is allowed in later stages.
- Scroll state must be owner-scoped and layer-aware.

### 4. Stage Plan Alignment Update
Updated plan status and linked report in:
- `docs/scrollable_rtt_plan.md`

Outcome:
- Stage 1 is now marked executed with explicit artifacts and references.

## Non-Goals (Intentionally Not Done in Stage 1)
1. No `Scrollable` behavior implementation.
2. No render target or camera creation for scrolling yet.
3. No `ScrollBar` behavior implementation yet.
4. No menu adapter wiring yet.

These remain in later phases by design.

## Artifacts Produced
1. `src/systems/ui/scroll/mod.rs`
2. `src/systems/ui/mod.rs` (module export update)
3. `src/systems/ui/menu/mod.rs` (integration contract comments)
4. `docs/scrollable_rtt_plan.md` (phase execution status update)
5. `docs/scrollable_stage1_report.md` (this report)

## Exit Criteria Check
1. Primitive boundary documented and established: Pass
2. Integration points declared: Pass
3. Owner/layer constraints preserved: Pass
4. No premature behavior coupling: Pass

Stage 1 exit verdict: Complete.

