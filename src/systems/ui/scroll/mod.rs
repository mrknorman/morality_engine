//! Scroll system namespace.
//!
//! Stage 1 boundary contract:
//! - All reusable scrolling primitives (`Scrollable`, `ScrollBar`) live under
//!   `systems::ui::scroll`.
//! - Menu-specific adapters remain under `systems::ui::menu`.
//! - Ownership and arbitration must remain owner-scoped via `UiLayer` and
//!   `InteractionGate` (no global singleton scroll state).
//!
//! Implementation systems/components are introduced in later stages.

