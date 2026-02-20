//! Reusable dropdown-layer state primitives.
//!
//! This module tracks per-owner dropdown visibility/anchors and exposes
//! helpers to open/close dropdown layers without hardcoding menu-specific
//! behavior.
use bevy::{
    ecs::{lifecycle::HookContext, query::QueryFilter, world::DeferredWorld},
    prelude::*,
};
use std::collections::{HashMap, HashSet};

use crate::systems::{
    interaction::{SelectableClickActivation, SelectableMenu},
    ui::layer::{UiLayer, UiLayerKind},
};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[require(SelectableMenu, Visibility)]
#[component(on_insert = DropdownSurface::on_insert)]
pub struct DropdownSurface {
    /// Logical owner used for layer/gate arbitration.
    pub owner: Entity,
    /// Click activation mode for the dropdown's `SelectableMenu`.
    pub click_activation: SelectableClickActivation,
}

impl DropdownSurface {
    /// Creates a dropdown root bound to `owner`.
    pub const fn new(owner: Entity) -> Self {
        Self {
            owner,
            click_activation: SelectableClickActivation::SelectedOnAnyClick,
        }
    }

    /// Overrides click activation policy for this dropdown root.
    pub const fn with_click_activation(
        mut self,
        click_activation: SelectableClickActivation,
    ) -> Self {
        self.click_activation = click_activation;
        self
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let Some(surface) = world.entity(entity).get::<DropdownSurface>().copied() else {
            return;
        };

        if world.entity(entity).get::<UiLayer>().is_none() {
            world
                .commands()
                .entity(entity)
                .insert(UiLayer::new(surface.owner, UiLayerKind::Dropdown));
        }

        if let Some(existing) = world.entity(entity).get::<SelectableMenu>().cloned() {
            if existing.click_activation != surface.click_activation {
                world.commands().entity(entity).insert(
                    existing.with_click_activation(surface.click_activation),
                );
            }
        } else {
            world.commands().entity(entity).insert(
                SelectableMenu::default().with_click_activation(surface.click_activation),
            );
        }

        let visibility_is_default = world
            .entity(entity)
            .get::<Visibility>()
            .is_none_or(|visibility| *visibility == Visibility::Inherited);
        if visibility_is_default {
            world
                .commands()
                .entity(entity)
                .insert(Visibility::Hidden);
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct DropdownLayerState {
    open_parent_by_owner: HashMap<Entity, Entity>,
    suppress_toggle_once_by_owner: HashSet<Entity>,
}

impl DropdownLayerState {
    /// Returns the currently open parent entity for an owner, if any.
    pub fn open_parent_for_owner(&self, owner: Entity) -> Option<Entity> {
        self.open_parent_by_owner.get(&owner).copied()
    }

    /// Returns whether `parent` is currently the open dropdown parent for `owner`.
    pub fn is_parent_open_for_owner(&self, owner: Entity, parent: Entity) -> bool {
        self.open_parent_for_owner(owner) == Some(parent)
    }

    /// Returns whether at least one owner currently has an open dropdown parent.
    pub fn is_any_open(&self) -> bool {
        !self.open_parent_by_owner.is_empty()
    }

    /// Snapshot of currently open parent mappings `(owner, parent)`.
    pub fn open_parents_snapshot(&self) -> Vec<(Entity, Entity)> {
        self.open_parent_by_owner
            .iter()
            .map(|(owner, parent)| (*owner, *parent))
            .collect()
    }

    /// Marks currently open owners so one immediate toggle is ignored.
    pub fn mark_suppress_toggle_for_open_owners(&mut self) {
        let open_owners: Vec<Entity> = self.open_parent_by_owner.keys().copied().collect();
        for owner in open_owners {
            self.suppress_toggle_once_by_owner.insert(owner);
        }
    }

    /// Marks `owner` so one immediate toggle is ignored.
    pub fn mark_suppress_toggle_for_owner(&mut self, owner: Entity) {
        if self.open_parent_by_owner.contains_key(&owner) {
            self.suppress_toggle_once_by_owner.insert(owner);
        }
    }

    /// Consumes and returns whether `owner` had a one-shot toggle suppression.
    pub fn take_suppress_toggle_once(&mut self, owner: Entity) -> bool {
        self.suppress_toggle_once_by_owner.remove(&owner)
    }

    /// Clears open/suppression state for `owner`.
    pub fn clear_owner(&mut self, owner: Entity) {
        self.open_parent_by_owner.remove(&owner);
        self.suppress_toggle_once_by_owner.remove(&owner);
    }

    /// Clears all owner dropdown open/suppression state.
    pub fn clear_all(&mut self) {
        self.open_parent_by_owner.clear();
        self.suppress_toggle_once_by_owner.clear();
    }
}

#[derive(Resource, Debug, Default)]
pub struct DropdownAnchorState {
    row_by_owner_parent: HashMap<(Entity, Entity), usize>,
}

impl DropdownAnchorState {
    /// Stores the anchor row used when opening a dropdown for an owner/parent.
    pub fn set_for_parent(&mut self, owner: Entity, parent_entity: Entity, row: usize) {
        self.row_by_owner_parent.insert((owner, parent_entity), row);
    }

    /// Returns stored anchor row for `(owner, parent_entity)` or `fallback`.
    pub fn row_for_parent(&self, owner: Entity, parent_entity: Entity, fallback: usize) -> usize {
        self.row_by_owner_parent
            .get(&(owner, parent_entity))
            .copied()
            .unwrap_or(fallback)
    }

    /// Removes all anchor rows for an owner.
    pub fn remove_owner(&mut self, owner: Entity) {
        self.row_by_owner_parent
            .retain(|(key_owner, _), _| *key_owner != owner);
    }

    /// Retains anchor rows whose parent is still open and still exists.
    pub fn retain_open_parents<R: Component>(
        &mut self,
        dropdown_state: &DropdownLayerState,
        parent_root_query: &Query<Entity, With<R>>,
    ) {
        self.row_by_owner_parent.retain(|(owner, parent), _| {
            parent_root_query.get(*parent).is_ok()
                && dropdown_state.is_parent_open_for_owner(*owner, *parent)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{DropdownAnchorState, DropdownSurface};
    use crate::systems::{
        interaction::{SelectableClickActivation, SelectableMenu},
        ui::layer::{UiLayer, UiLayerKind},
    };
    use bevy::prelude::{Entity, Visibility, World};

    #[test]
    fn anchored_row_overrides_selection_fallback_until_owner_clears() {
        let owner = Entity::from_bits(1);
        let parent = Entity::from_bits(2);
        let mut anchors = DropdownAnchorState::default();
        anchors.set_for_parent(owner, parent, 1);

        assert_eq!(anchors.row_for_parent(owner, parent, 0), 1);
        assert_eq!(anchors.row_for_parent(owner, parent, 2), 1);

        anchors.remove_owner(owner);
        assert_eq!(anchors.row_for_parent(owner, parent, 2), 2);
    }

    #[test]
    fn dropdown_surface_insertion_adds_required_layer_and_hidden_visibility() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let dropdown = world
            .spawn(
                DropdownSurface::new(owner)
                    .with_click_activation(SelectableClickActivation::HoveredOnly),
            )
            .id();

        assert!(world.entity(dropdown).get::<SelectableMenu>().is_some());
        assert_eq!(
            world.entity(dropdown).get::<UiLayer>(),
            Some(&UiLayer::new(owner, UiLayerKind::Dropdown))
        );
        assert_eq!(
            world
                .entity(dropdown)
                .get::<SelectableMenu>()
                .map(|menu| menu.click_activation),
            Some(SelectableClickActivation::HoveredOnly)
        );
        assert_eq!(
            world.entity(dropdown).get::<Visibility>(),
            Some(&Visibility::Hidden)
        );
    }
}

/// Opens dropdown roots of type `D` under `parent_entity` for `owner`.
///
/// Any sibling dropdown roots for the same owner are hidden.
pub fn open_for_parent<D: Component>(
    owner: Entity,
    parent_entity: Entity,
    selected_index: usize,
    dropdown_state: &mut DropdownLayerState,
    dropdown_query: &mut Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<D>>,
    dropdown_menu_query: &mut Query<&mut SelectableMenu, impl QueryFilter>,
) {
    let mut found = false;
    for (dropdown_entity, parent, ui_layer, mut visibility) in dropdown_query.iter_mut() {
        if ui_layer.owner != owner {
            continue;
        }
        if parent.parent() == parent_entity {
            *visibility = Visibility::Visible;
            found = true;
            if let Ok(mut dropdown_menu) = dropdown_menu_query.get_mut(dropdown_entity) {
                dropdown_menu.selected_index = selected_index;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }

    if found {
        dropdown_state
            .open_parent_by_owner
            .insert(owner, parent_entity);
    } else if dropdown_state.is_parent_open_for_owner(owner, parent_entity) {
        dropdown_state.clear_owner(owner);
    }
}

/// Closes all dropdown roots of type `D` and clears open state.
pub fn close_all<D: Component>(
    dropdown_state: &mut DropdownLayerState,
    dropdown_query: &mut Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<D>>,
) {
    for (_, _, _, mut visibility) in dropdown_query.iter_mut() {
        *visibility = Visibility::Hidden;
    }
    dropdown_state.clear_all();
}

/// Closes dropdown roots for a specific owner/parent pair.
pub fn close_for_parent<D: Component>(
    owner: Entity,
    parent_entity: Entity,
    dropdown_state: &mut DropdownLayerState,
    dropdown_query: &mut Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<D>>,
) {
    for (_, parent, ui_layer, mut visibility) in dropdown_query.iter_mut() {
        if ui_layer.owner == owner && parent.parent() == parent_entity {
            *visibility = Visibility::Hidden;
        }
    }
    if dropdown_state.is_parent_open_for_owner(owner, parent_entity) {
        dropdown_state.clear_owner(owner);
    }
}

/// Enforces at most one visible dropdown root per owner.
///
/// Parent roots are constrained by marker `R`, allowing this helper to be reused
/// for menu and non-menu dropdown surfaces.
pub fn enforce_single_visible_layer<D: Component, R: Component>(
    dropdown_state: &mut DropdownLayerState,
    parent_root_query: &Query<Entity, With<R>>,
    block_all_dropdowns: bool,
    dropdown_query: &mut Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<D>>,
) {
    dropdown_state
        .open_parent_by_owner
        .retain(|_, parent_entity| parent_root_query.get(*parent_entity).is_ok());
    if block_all_dropdowns {
        close_all::<D>(dropdown_state, dropdown_query);
        return;
    }

    let mut visible_by_owner: HashMap<Entity, Vec<(Entity, Entity)>> = HashMap::new();
    for (dropdown_entity, parent, ui_layer, mut visibility) in dropdown_query.iter_mut() {
        if parent_root_query.get(parent.parent()).is_err() {
            *visibility = Visibility::Hidden;
            continue;
        }
        if *visibility == Visibility::Visible {
            visible_by_owner
                .entry(ui_layer.owner)
                .or_default()
                .push((dropdown_entity, parent.parent()));
        }
    }

    let mut keep_by_owner: HashMap<Entity, Entity> = HashMap::new();
    let mut keep_dropdowns = HashSet::new();
    for (owner, visible_dropdowns) in visible_by_owner {
        if visible_dropdowns.is_empty() {
            continue;
        }
        let keep_dropdown = if visible_dropdowns.len() == 1 {
            visible_dropdowns[0]
        } else if let Some(open_parent) = dropdown_state.open_parent_for_owner(owner) {
            visible_dropdowns
                .iter()
                .copied()
                .find(|(_, parent_entity)| *parent_entity == open_parent)
                .unwrap_or(visible_dropdowns[0])
        } else {
            visible_dropdowns[0]
        };
        keep_dropdowns.insert(keep_dropdown.0);
        keep_by_owner.insert(owner, keep_dropdown.1);
    }

    for (dropdown_entity, _, ui_layer, mut visibility) in dropdown_query.iter_mut() {
        if keep_by_owner.contains_key(&ui_layer.owner) && keep_dropdowns.contains(&dropdown_entity) {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }

    dropdown_state.open_parent_by_owner = keep_by_owner;
    dropdown_state
        .suppress_toggle_once_by_owner
        .retain(|owner| dropdown_state.open_parent_by_owner.contains_key(owner));
}
