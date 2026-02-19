use bevy::{ecs::query::QueryFilter, prelude::*};
use std::collections::{HashMap, HashSet};

use crate::systems::{
    interaction::SelectableMenu,
    ui::layer::UiLayer,
};

#[derive(Resource, Debug, Default)]
pub struct DropdownLayerState {
    open_parent_by_owner: HashMap<Entity, Entity>,
    suppress_toggle_once_by_owner: HashSet<Entity>,
}

impl DropdownLayerState {
    pub fn open_parent_for_owner(&self, owner: Entity) -> Option<Entity> {
        self.open_parent_by_owner.get(&owner).copied()
    }

    pub fn is_parent_open_for_owner(&self, owner: Entity, parent: Entity) -> bool {
        self.open_parent_for_owner(owner) == Some(parent)
    }

    pub fn is_any_open(&self) -> bool {
        !self.open_parent_by_owner.is_empty()
    }

    pub fn open_parents_snapshot(&self) -> Vec<(Entity, Entity)> {
        self.open_parent_by_owner
            .iter()
            .map(|(owner, parent)| (*owner, *parent))
            .collect()
    }

    pub fn mark_suppress_toggle_for_open_owners(&mut self) {
        let open_owners: Vec<Entity> = self.open_parent_by_owner.keys().copied().collect();
        for owner in open_owners {
            self.suppress_toggle_once_by_owner.insert(owner);
        }
    }

    pub fn mark_suppress_toggle_for_owner(&mut self, owner: Entity) {
        if self.open_parent_by_owner.contains_key(&owner) {
            self.suppress_toggle_once_by_owner.insert(owner);
        }
    }

    pub fn take_suppress_toggle_once(&mut self, owner: Entity) -> bool {
        self.suppress_toggle_once_by_owner.remove(&owner)
    }

    pub fn clear_owner(&mut self, owner: Entity) {
        self.open_parent_by_owner.remove(&owner);
        self.suppress_toggle_once_by_owner.remove(&owner);
    }

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
    pub fn set_for_parent(&mut self, owner: Entity, parent_entity: Entity, row: usize) {
        self.row_by_owner_parent.insert((owner, parent_entity), row);
    }

    pub fn row_for_parent(&self, owner: Entity, parent_entity: Entity, fallback: usize) -> usize {
        self.row_by_owner_parent
            .get(&(owner, parent_entity))
            .copied()
            .unwrap_or(fallback)
    }

    pub fn remove_owner(&mut self, owner: Entity) {
        self.row_by_owner_parent
            .retain(|(key_owner, _), _| *key_owner != owner);
    }

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
    use super::DropdownAnchorState;
    use bevy::prelude::Entity;

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
}

pub fn any_open<D: Component>(dropdown_query: &Query<&Visibility, With<D>>) -> bool {
    dropdown_query
        .iter()
        .any(|visibility| *visibility == Visibility::Visible)
}

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

pub fn close_all<D: Component>(
    dropdown_state: &mut DropdownLayerState,
    dropdown_query: &mut Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<D>>,
) {
    for (_, _, _, mut visibility) in dropdown_query.iter_mut() {
        *visibility = Visibility::Hidden;
    }
    dropdown_state.clear_all();
}

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

pub fn close_for_owner<D: Component>(
    owner: Entity,
    dropdown_state: &mut DropdownLayerState,
    dropdown_query: &mut Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<D>>,
) {
    for (_, _, ui_layer, mut visibility) in dropdown_query.iter_mut() {
        if ui_layer.owner == owner {
            *visibility = Visibility::Hidden;
        }
    }
    dropdown_state.clear_owner(owner);
}

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
