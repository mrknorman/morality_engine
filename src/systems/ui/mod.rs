//! Shared UI primitives.
//!
//! This module hosts reusable ECS-first building blocks used by menu and
//! non-menu surfaces:
//! - interaction layering (`layer`)
//! - dropdown/tab/selector primitives
//! - render-to-texture scrolling
//! - hover help presentation
//!
//! `systems::ui::menu` composes these primitives into concrete menu behavior.
pub mod discrete_slider;
pub mod dropdown;
pub mod hover_box;
pub mod layer;
pub mod menu;
pub mod menu_surface;
pub mod scroll;
pub mod selector;
pub mod tabs;
pub mod window;
