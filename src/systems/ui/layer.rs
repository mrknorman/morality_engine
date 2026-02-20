//! Owner-scoped UI layer arbitration utilities.
//!
//! This module defines the shared layered interaction model used by menu and
//! non-menu surfaces. Each UI owner can expose multiple layer roots
//! (`Base`, `Dropdown`, `Modal`), and systems resolve a single active layer
//! per owner for deterministic input routing.
use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    data::states::PauseState,
    systems::interaction::{
        interaction_context_active_for_owner, interaction_gate_allows, InteractionCapture,
        InteractionCaptureOwner, InteractionGate,
    },
};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UiLayerKind {
    /// Base/root layer for a UI owner.
    Base,
    /// Transient dropdown layer for a UI owner.
    Dropdown,
    /// Blocking modal layer for a UI owner.
    Modal,
}

impl UiLayerKind {
    pub const fn priority(self) -> u8 {
        match self {
            Self::Base => 0,
            Self::Dropdown => 1,
            Self::Modal => 2,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UiLayer {
    /// Logical owner of this layer root.
    pub owner: Entity,
    /// Layer kind used for active-layer arbitration.
    pub kind: UiLayerKind,
}

impl UiLayer {
    pub const fn new(owner: Entity, kind: UiLayerKind) -> Self {
        Self { owner, kind }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ActiveUiLayer {
    /// Active root entity for the owner.
    pub entity: Entity,
    /// Active layer kind.
    pub kind: UiLayerKind,
}

/// Returns the resolved active layer for `owner` when present.
pub fn active_layer_for_owner(
    active_layers: &HashMap<Entity, ActiveUiLayer>,
    owner: Entity,
) -> Option<ActiveUiLayer> {
    active_layers.get(&owner).copied()
}

/// Returns the resolved active-layer kind for `owner`.
///
/// Falls back to `UiLayerKind::Base` when no active layer exists.
pub fn active_layer_kind_for_owner(
    active_layers: &HashMap<Entity, ActiveUiLayer>,
    owner: Entity,
) -> UiLayerKind {
    active_layer_for_owner(active_layers, owner)
        .map(|active| active.kind)
        .unwrap_or(UiLayerKind::Base)
}

/// Returns whether `layer_entity` is currently the active layer for `owner`.
pub fn is_active_layer_entity_for_owner(
    active_layers: &HashMap<Entity, ActiveUiLayer>,
    owner: Entity,
    layer_entity: Entity,
) -> bool {
    active_layer_for_owner(active_layers, owner).is_some_and(|active| active.entity == layer_entity)
}

/// Returns active layers sorted by owner/entity rank for deterministic routing.
pub fn ordered_active_layers_by_owner(
    active_layers: &HashMap<Entity, ActiveUiLayer>,
) -> Vec<(Entity, ActiveUiLayer)> {
    let mut ordered: Vec<(Entity, ActiveUiLayer)> = active_layers
        .iter()
        .map(|(owner, active_layer)| (*owner, *active_layer))
        .collect();
    ordered.sort_by_key(|(owner, active_layer)| (owner.index(), active_layer.entity.index()));
    ordered
}

/// Returns owners whose active layer matches `kind`, in deterministic order.
pub fn ordered_active_owners_by_kind(
    active_layers: &HashMap<Entity, ActiveUiLayer>,
    kind: UiLayerKind,
) -> Vec<Entity> {
    ordered_active_layers_by_owner(active_layers)
        .into_iter()
        .filter_map(|(owner, active_layer)| (active_layer.kind == kind).then_some(owner))
        .collect()
}

fn is_visible(visibility: Option<&Visibility>) -> bool {
    visibility.copied().unwrap_or(Visibility::Visible) != Visibility::Hidden
}

/// Resolves one active UI layer per owner with visibility + gate filtering.
///
/// Priority is `Modal > Dropdown > Base`. Ties within a kind are broken by
/// deterministic entity rank.
pub fn active_layers_by_owner_scoped(
    pause_state: Option<&Res<State<PauseState>>>,
    capture_query: &Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    layer_query: &Query<(
        Entity,
        &UiLayer,
        Option<&Visibility>,
        Option<&InteractionGate>,
    )>,
) -> HashMap<Entity, ActiveUiLayer> {
    let mut active: HashMap<Entity, ActiveUiLayer> = HashMap::new();
    for (entity, layer, visibility, gate) in layer_query.iter() {
        let interaction_captured =
            interaction_context_active_for_owner(pause_state, capture_query, layer.owner);
        if !is_visible(visibility) || !interaction_gate_allows(gate, interaction_captured) {
            continue;
        }

        match active.get(&layer.owner) {
            Some(current) if current.kind.priority() > layer.kind.priority() => {}
            Some(current)
                if current.kind.priority() == layer.kind.priority()
                    && current.entity.index() > entity.index() => {}
            _ => {
                active.insert(
                    layer.owner,
                    ActiveUiLayer {
                        entity,
                        kind: layer.kind,
                    },
                );
            }
        }
    }
    active
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;

    #[test]
    fn active_layer_prefers_higher_priority_kind() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let base = world
            .spawn((UiLayer::new(owner, UiLayerKind::Base), Visibility::Visible))
            .id();
        let modal = world
            .spawn((UiLayer::new(owner, UiLayerKind::Modal), Visibility::Visible))
            .id();

        let mut capture_state: SystemState<
            Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
        > = SystemState::new(&mut world);
        let mut layer_state: SystemState<
            Query<(
                Entity,
                &UiLayer,
                Option<&Visibility>,
                Option<&InteractionGate>,
            )>,
        > = SystemState::new(&mut world);

        let capture_query = capture_state.get(&world);
        let layer_query = layer_state.get(&world);
        let active = active_layers_by_owner_scoped(None, &capture_query, &layer_query);

        assert_eq!(active.get(&owner).map(|layer| layer.entity), Some(modal));
        assert_ne!(Some(base), active.get(&owner).map(|layer| layer.entity));
    }

    #[test]
    fn same_priority_layer_resolution_is_deterministic_by_entity_rank() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let first = world
            .spawn((UiLayer::new(owner, UiLayerKind::Base), Visibility::Visible))
            .id();
        let second = world
            .spawn((UiLayer::new(owner, UiLayerKind::Base), Visibility::Visible))
            .id();

        let mut capture_state: SystemState<
            Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
        > = SystemState::new(&mut world);
        let mut layer_state: SystemState<
            Query<(
                Entity,
                &UiLayer,
                Option<&Visibility>,
                Option<&InteractionGate>,
            )>,
        > = SystemState::new(&mut world);

        let capture_query = capture_state.get(&world);
        let layer_query = layer_state.get(&world);
        let active = active_layers_by_owner_scoped(None, &capture_query, &layer_query);

        let resolved = active.get(&owner).map(|layer| layer.entity);
        let expected = if first.index() >= second.index() {
            Some(first)
        } else {
            Some(second)
        };
        assert_eq!(resolved, expected);
    }

    #[test]
    fn ordered_active_layers_by_owner_returns_stable_owner_order() {
        let mut world = World::new();
        let owner_a = world.spawn_empty().id();
        let owner_b = world.spawn_empty().id();
        assert!(owner_a.index() < owner_b.index());
        let mut active_layers = HashMap::new();
        active_layers.insert(
            owner_b,
            ActiveUiLayer {
                entity: world.spawn_empty().id(),
                kind: UiLayerKind::Dropdown,
            },
        );
        active_layers.insert(
            owner_a,
            ActiveUiLayer {
                entity: world.spawn_empty().id(),
                kind: UiLayerKind::Base,
            },
        );

        let ordered = ordered_active_layers_by_owner(&active_layers);
        assert_eq!(ordered.len(), 2);
        assert_eq!(ordered[0].0, owner_a);
        assert_eq!(ordered[1].0, owner_b);
    }

    #[test]
    fn ordered_active_owners_by_kind_filters_and_keeps_owner_order() {
        let mut world = World::new();
        let owner_a = world.spawn_empty().id();
        let owner_b = world.spawn_empty().id();
        let owner_c = world.spawn_empty().id();
        assert!(owner_a.index() < owner_b.index());
        assert!(owner_b.index() < owner_c.index());

        let mut active_layers = HashMap::new();
        active_layers.insert(
            owner_b,
            ActiveUiLayer {
                entity: world.spawn_empty().id(),
                kind: UiLayerKind::Dropdown,
            },
        );
        active_layers.insert(
            owner_a,
            ActiveUiLayer {
                entity: world.spawn_empty().id(),
                kind: UiLayerKind::Base,
            },
        );
        active_layers.insert(
            owner_c,
            ActiveUiLayer {
                entity: world.spawn_empty().id(),
                kind: UiLayerKind::Dropdown,
            },
        );

        let ordered = ordered_active_owners_by_kind(&active_layers, UiLayerKind::Dropdown);
        assert_eq!(ordered, vec![owner_b, owner_c]);
    }

    #[test]
    fn active_layer_resolution_is_independent_per_owner_under_mixed_layers() {
        let mut world = World::new();
        let owner_a = world.spawn_empty().id();
        let owner_b = world.spawn_empty().id();

        let owner_a_base = world
            .spawn((
                UiLayer::new(owner_a, UiLayerKind::Base),
                Visibility::Visible,
            ))
            .id();
        let owner_a_modal = world
            .spawn((
                UiLayer::new(owner_a, UiLayerKind::Modal),
                Visibility::Visible,
            ))
            .id();

        let owner_b_base = world
            .spawn((
                UiLayer::new(owner_b, UiLayerKind::Base),
                Visibility::Visible,
            ))
            .id();
        let owner_b_dropdown = world
            .spawn((
                UiLayer::new(owner_b, UiLayerKind::Dropdown),
                Visibility::Visible,
            ))
            .id();
        let owner_b_modal = world
            .spawn((
                UiLayer::new(owner_b, UiLayerKind::Modal),
                Visibility::Hidden,
            ))
            .id();

        let mut capture_state: SystemState<
            Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
        > = SystemState::new(&mut world);
        let mut layer_state: SystemState<
            Query<(
                Entity,
                &UiLayer,
                Option<&Visibility>,
                Option<&InteractionGate>,
            )>,
        > = SystemState::new(&mut world);

        let capture_query = capture_state.get(&world);
        let layer_query = layer_state.get(&world);
        let mut active = active_layers_by_owner_scoped(None, &capture_query, &layer_query);
        assert_eq!(
            active_layer_for_owner(&active, owner_a).map(|layer| layer.entity),
            Some(owner_a_modal)
        );
        assert_eq!(
            active_layer_for_owner(&active, owner_b).map(|layer| layer.entity),
            Some(owner_b_dropdown)
        );

        world.entity_mut(owner_b_modal).insert(Visibility::Visible);
        let capture_query = capture_state.get(&world);
        let layer_query = layer_state.get(&world);
        active = active_layers_by_owner_scoped(None, &capture_query, &layer_query);
        assert_eq!(
            active_layer_for_owner(&active, owner_a).map(|layer| layer.entity),
            Some(owner_a_modal)
        );
        assert_eq!(
            active_layer_for_owner(&active, owner_b).map(|layer| layer.entity),
            Some(owner_b_modal)
        );

        world.entity_mut(owner_a_modal).insert(Visibility::Hidden);
        let capture_query = capture_state.get(&world);
        let layer_query = layer_state.get(&world);
        active = active_layers_by_owner_scoped(None, &capture_query, &layer_query);
        assert_eq!(
            active_layer_for_owner(&active, owner_a).map(|layer| layer.entity),
            Some(owner_a_base)
        );
        assert_eq!(
            active_layer_for_owner(&active, owner_b).map(|layer| layer.entity),
            Some(owner_b_modal)
        );

        assert!(is_active_layer_entity_for_owner(
            &active,
            owner_a,
            owner_a_base
        ));
        assert!(!is_active_layer_entity_for_owner(
            &active,
            owner_a,
            owner_a_modal
        ));
        assert!(is_active_layer_entity_for_owner(
            &active,
            owner_b,
            owner_b_modal
        ));
        assert!(!is_active_layer_entity_for_owner(
            &active,
            owner_b,
            owner_b_base
        ));
    }

    #[test]
    fn interaction_capture_is_owner_scoped_for_gate_resolution() {
        let mut world = World::new();
        let owner_a = world.spawn_empty().id();
        let owner_b = world.spawn_empty().id();

        let owner_a_pause_menu = world
            .spawn((
                UiLayer::new(owner_a, UiLayerKind::Base),
                Visibility::Visible,
                InteractionGate::PauseMenuOnly,
            ))
            .id();
        let owner_b_pause_menu = world
            .spawn((
                UiLayer::new(owner_b, UiLayerKind::Base),
                Visibility::Visible,
                InteractionGate::PauseMenuOnly,
            ))
            .id();
        let owner_b_gameplay = world
            .spawn((
                UiLayer::new(owner_b, UiLayerKind::Base),
                Visibility::Visible,
                InteractionGate::GameplayOnly,
            ))
            .id();

        world.spawn((InteractionCapture, InteractionCaptureOwner::new(owner_a)));

        let mut capture_state: SystemState<
            Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
        > = SystemState::new(&mut world);
        let mut layer_state: SystemState<
            Query<(
                Entity,
                &UiLayer,
                Option<&Visibility>,
                Option<&InteractionGate>,
            )>,
        > = SystemState::new(&mut world);

        let capture_query = capture_state.get(&world);
        let layer_query = layer_state.get(&world);
        let active = active_layers_by_owner_scoped(None, &capture_query, &layer_query);

        assert_eq!(
            active_layer_for_owner(&active, owner_a).map(|layer| layer.entity),
            Some(owner_a_pause_menu)
        );
        assert_eq!(
            active_layer_for_owner(&active, owner_b).map(|layer| layer.entity),
            Some(owner_b_gameplay)
        );
        assert!(!is_active_layer_entity_for_owner(
            &active,
            owner_b,
            owner_b_pause_menu
        ));
    }
}
