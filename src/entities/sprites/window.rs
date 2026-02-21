//! Compatibility shim for the UI window primitive.
//!
//! Canonical window primitive logic now lives in `crate::systems::ui::window`.
//! This module re-exports both the legacy `Window*` names and the new
//! `UiWindow*` names while callers migrate.

#[allow(unused_imports)]
pub use crate::systems::ui::window::*;
