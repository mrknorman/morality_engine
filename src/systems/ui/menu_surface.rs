//! Reusable selectable menu-surface primitive.
//!
//! `MenuSurface` standardizes owner/layer wiring for selectable menu roots:
//! - ensures a `UiLayer` exists for active-layer arbitration
//! - ensures a `SelectableMenu` exists for navigation
//! - configures click activation policy for pointer behavior
//!
//! Nested interaction roots (for example tab bars that live inside a parent
//! menu surface) can opt out of layer insertion via `without_layer()` so they
//! don't compete with their owner's active base layer.
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::systems::{
    interaction::{SelectableClickActivation, SelectableMenu},
    ui::layer::{UiLayer, UiLayerKind},
};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[require(SelectableMenu, Visibility)]
#[component(on_insert = MenuSurface::on_insert)]
pub struct MenuSurface {
    pub owner: Entity,
    pub layer: UiLayerKind,
    pub insert_layer: bool,
    pub click_activation: SelectableClickActivation,
}

impl MenuSurface {
    pub const fn new(owner: Entity) -> Self {
        Self {
            owner,
            layer: UiLayerKind::Base,
            insert_layer: true,
            click_activation: SelectableClickActivation::SelectedOnAnyClick,
        }
    }

    pub const fn with_layer(mut self, layer: UiLayerKind) -> Self {
        self.layer = layer;
        self.insert_layer = true;
        self
    }

    pub const fn without_layer(mut self) -> Self {
        self.insert_layer = false;
        self
    }

    pub const fn with_click_activation(
        mut self,
        click_activation: SelectableClickActivation,
    ) -> Self {
        self.click_activation = click_activation;
        self
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let Some(surface) = world.entity(entity).get::<MenuSurface>().copied() else {
            return;
        };

        if surface.insert_layer && world.entity(entity).get::<UiLayer>().is_none() {
            world
                .commands()
                .entity(entity)
                .insert(UiLayer::new(surface.owner, surface.layer));
        }

        if let Some(existing) = world.entity(entity).get::<SelectableMenu>().cloned() {
            if existing.click_activation != surface.click_activation {
                world.commands().entity(entity).insert(
                    existing.with_click_activation(surface.click_activation),
                );
            }
            return;
        }

        world
            .commands()
            .entity(entity)
            .insert(SelectableMenu::default().with_click_activation(surface.click_activation));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_surface_insertion_adds_layer_and_click_policy() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let surface = world
            .spawn(MenuSurface::new(owner).with_click_activation(SelectableClickActivation::HoveredOnly))
            .id();

        assert_eq!(
            world.entity(surface).get::<UiLayer>(),
            Some(&UiLayer::new(owner, UiLayerKind::Base))
        );
        assert_eq!(
            world
                .entity(surface)
                .get::<SelectableMenu>()
                .map(|menu| menu.click_activation),
            Some(SelectableClickActivation::HoveredOnly)
        );
    }

    #[test]
    fn menu_surface_keeps_navigation_keys_when_selectable_menu_exists() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let selectable_menu = SelectableMenu::new(
            2,
            vec![KeyCode::KeyW],
            vec![KeyCode::KeyS],
            vec![KeyCode::Space],
            false,
        );
        let surface = world
            .spawn((
                selectable_menu.clone(),
                MenuSurface::new(owner)
                    .with_click_activation(SelectableClickActivation::HoveredOnly),
            ))
            .id();

        let menu = world
            .entity(surface)
            .get::<SelectableMenu>()
            .expect("selectable menu");
        assert_eq!(menu.selected_index, 2);
        assert_eq!(menu.up_keys, vec![KeyCode::KeyW]);
        assert_eq!(menu.down_keys, vec![KeyCode::KeyS]);
        assert_eq!(menu.activate_keys, vec![KeyCode::Space]);
        assert!(!menu.wrap);
        assert_eq!(menu.click_activation, SelectableClickActivation::HoveredOnly);
    }

    #[test]
    fn menu_surface_can_opt_out_of_layer_insertion() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let surface = world
            .spawn(
                MenuSurface::new(owner)
                    .without_layer()
                    .with_click_activation(SelectableClickActivation::HoveredOnly),
            )
            .id();

        assert!(world.entity(surface).get::<UiLayer>().is_none());
        assert_eq!(
            world
                .entity(surface)
                .get::<SelectableMenu>()
                .map(|menu| menu.click_activation),
            Some(SelectableClickActivation::HoveredOnly)
        );
    }
}
