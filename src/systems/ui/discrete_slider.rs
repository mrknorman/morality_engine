use std::collections::HashSet;

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::entities::sprites::compound::{HollowRectangle, RectangleSides};
use crate::systems::interaction::InteractionSystem;

#[derive(Component, Clone, Copy, Debug)]
#[require(Transform, Visibility)]
#[component(on_insert = DiscreteSlider::on_insert)]
pub struct DiscreteSlider {
    pub steps: usize,
    pub layout_steps: usize,
    pub selected: usize,
    pub filled_slots: usize,
    pub slot_size: Vec2,
    pub slot_gap: f32,
    pub fill_color: Color,
    pub empty_color: Color,
    pub border_color: Color,
    pub border_thickness: f32,
    pub fill_inset: f32,
}

impl DiscreteSlider {
    pub fn new(steps: usize, selected: usize) -> Self {
        let clamped_steps = steps.max(1);
        let clamped_selected = selected.min(clamped_steps - 1);
        Self {
            steps: clamped_steps,
            layout_steps: clamped_steps,
            selected: clamped_selected,
            filled_slots: clamped_selected.saturating_add(1).min(clamped_steps),
            slot_size: Vec2::splat(18.0),
            slot_gap: 6.0,
            fill_color: Color::WHITE,
            empty_color: Color::NONE,
            border_color: Color::WHITE,
            border_thickness: 2.0,
            fill_inset: 3.0,
        }
    }

    pub fn with_layout_steps(mut self, layout_steps: usize) -> Self {
        self.layout_steps = layout_steps.max(1);
        self
    }

    pub fn with_slot_size(mut self, slot_size: Vec2) -> Self {
        self.slot_size = slot_size.max(Vec2::splat(1.0));
        self
    }

    pub fn with_slot_gap(mut self, slot_gap: f32) -> Self {
        self.slot_gap = slot_gap.max(0.0);
        self
    }

    pub fn with_fill_color(mut self, fill_color: Color) -> Self {
        self.fill_color = fill_color;
        self
    }

    pub fn with_empty_color(mut self, empty_color: Color) -> Self {
        self.empty_color = empty_color;
        self
    }

    pub fn with_border_color(mut self, border_color: Color) -> Self {
        self.border_color = border_color;
        self
    }

    pub fn with_border_thickness(mut self, border_thickness: f32) -> Self {
        self.border_thickness = border_thickness.max(1.0);
        self
    }

    pub fn with_fill_inset(mut self, fill_inset: f32) -> Self {
        self.fill_inset = fill_inset.max(0.0);
        self
    }

    fn required_slots(self) -> usize {
        self.steps.max(1).max(self.layout_steps.max(1))
    }

    fn slot_inner_size(self) -> Vec2 {
        (self.slot_size - Vec2::splat(self.fill_inset * 2.0)).max(Vec2::splat(1.0))
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let Some(slider) = world.entity(entity).get::<DiscreteSlider>().copied() else {
            return;
        };

        let required_slots = slider.required_slots();
        let mut present_indices = HashSet::new();
        if let Some(children) = world.entity(entity).get::<Children>() {
            for child in children.iter() {
                if let Some(slot) = world.entity(child).get::<DiscreteSliderSlot>() {
                    present_indices.insert(slot.index);
                }
            }
        }
        let missing: Vec<usize> = (0..required_slots)
            .filter(|index| !present_indices.contains(index))
            .collect();
        if missing.is_empty() {
            return;
        }

        spawn_slider_slots(
            &mut world.commands(),
            entity,
            slider,
            required_slots,
            missing.into_iter(),
        );
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct DiscreteSliderSlot {
    pub index: usize,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum DiscreteSliderSystems {
    EnsureSlots,
    SyncSlots,
}

fn spawn_slider_slots(
    commands: &mut Commands,
    slider_entity: Entity,
    slider: DiscreteSlider,
    required_slots: usize,
    missing: impl Iterator<Item = usize>,
) {
    let inner_size = slider.slot_inner_size();
    commands.entity(slider_entity).with_children(|parent| {
        for index in missing {
            parent.spawn((
                Name::new(format!("discrete_slider_slot_{index}")),
                DiscreteSliderSlot { index },
                Sprite::from_color(slider.empty_color, inner_size),
                HollowRectangle {
                    dimensions: slider.slot_size,
                    thickness: slider.border_thickness,
                    color: slider.border_color,
                    sides: RectangleSides::default(),
                },
                Transform::from_xyz(
                    slot_center_x(index, required_slots, slider.slot_size.x, slider.slot_gap),
                    0.0,
                    0.0,
                ),
            ));
        }
    });
}

pub(crate) fn slot_center_x(index: usize, layout_steps: usize, slot_width: f32, slot_gap: f32) -> f32 {
    let total_width =
        layout_steps as f32 * slot_width + (layout_steps.saturating_sub(1)) as f32 * slot_gap;
    let left = -total_width * 0.5 + slot_width * 0.5;
    left + index as f32 * (slot_width + slot_gap)
}

pub(crate) fn slot_span_bounds(
    visible_steps: usize,
    layout_steps: usize,
    slot_width: f32,
    slot_gap: f32,
) -> (f32, f32) {
    let visible_steps = visible_steps.max(1);
    let layout_steps = layout_steps.max(visible_steps);
    let first_center = slot_center_x(0, layout_steps, slot_width, slot_gap);
    let last_center = slot_center_x(
        visible_steps.saturating_sub(1),
        layout_steps,
        slot_width,
        slot_gap,
    );
    (
        first_center - slot_width * 0.5,
        last_center + slot_width * 0.5,
    )
}

pub fn ensure_discrete_slider_slots(
    mut commands: Commands,
    slider_query: Query<(Entity, &DiscreteSlider, Option<&Children>)>,
    slot_query: Query<&DiscreteSliderSlot>,
) {
    for (slider_entity, slider, children) in slider_query.iter() {
        let required_slots = slider.required_slots();
        let mut present_indices = HashSet::new();
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(slot) = slot_query.get(child) {
                    present_indices.insert(slot.index);
                }
            }
        }

        let missing: Vec<usize> = (0..required_slots)
            .filter(|index| !present_indices.contains(index))
            .collect();
        if missing.is_empty() {
            continue;
        }

        spawn_slider_slots(
            &mut commands,
            slider_entity,
            *slider,
            required_slots,
            missing.into_iter(),
        );
    }
}

pub fn sync_discrete_slider_slots(
    mut queries: ParamSet<(
        Query<
            (&DiscreteSlider, &Children, Option<&Visibility>),
            Without<DiscreteSliderSlot>,
        >,
        Query<
            (
                &DiscreteSliderSlot,
                &mut Sprite,
                &mut HollowRectangle,
                &mut Transform,
                &mut Visibility,
            ),
            With<DiscreteSliderSlot>,
        >,
    )>,
) {
    // Query contract:
    // - slider roots are read via `p0` and exclude `DiscreteSliderSlot`.
    // - slot entities are mutated via `p1` and require `DiscreteSliderSlot`.
    // - shared `Visibility` access remains disjoint through these reciprocal
    //   role filters, avoiding B0001 aliasing.
    let mut slider_states: Vec<(DiscreteSlider, Vec<Entity>, bool)> = Vec::new();
    {
        let slider_query = queries.p0();
        for (slider, children, slider_visibility) in slider_query.iter() {
            slider_states.push((
                *slider,
                children.iter().collect(),
                slider_visibility.is_some_and(|visibility| *visibility == Visibility::Hidden),
            ));
        }
    }

    let mut slot_query = queries.p1();
    for (slider, children, slider_hidden) in slider_states {
        let steps = slider.steps.max(1);
        let layout_steps = slider.layout_steps.max(steps);
        let filled = slider.filled_slots.min(steps);
        let inner_size = (slider.slot_size - Vec2::splat(slider.fill_inset * 2.0)).max(Vec2::splat(1.0));

        for child in children {
            let Ok((slot, mut sprite, mut border, mut transform, mut visibility)) =
                slot_query.get_mut(child)
            else {
                continue;
            };

            if slider_hidden || slot.index >= steps {
                *visibility = Visibility::Hidden;
                continue;
            }

            *visibility = Visibility::Visible;
            transform.translation.x =
                slot_center_x(slot.index, layout_steps, slider.slot_size.x, slider.slot_gap);
            border.dimensions = slider.slot_size;
            border.thickness = slider.border_thickness;
            border.color = slider.border_color;
            sprite.custom_size = Some(inner_size);
            sprite.color = if slot.index < filled {
                slider.fill_color
            } else {
                slider.empty_color
            };
        }
    }
}

pub struct DiscreteSliderPlugin;

impl Plugin for DiscreteSliderPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                DiscreteSliderSystems::EnsureSlots,
                DiscreteSliderSystems::SyncSlots.after(DiscreteSliderSystems::EnsureSlots),
            )
                .chain()
                .after(InteractionSystem::Selectable),
        )
        .add_systems(
            Update,
            ensure_discrete_slider_slots.in_set(DiscreteSliderSystems::EnsureSlots),
        )
        .add_systems(
            Update,
            sync_discrete_slider_slots.in_set(DiscreteSliderSystems::SyncSlots),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        slot_center_x, slot_span_bounds, DiscreteSlider, DiscreteSliderPlugin, DiscreteSliderSlot,
    };
    use bevy::prelude::*;

    #[test]
    fn clamps_steps_and_selected() {
        let slider = DiscreteSlider::new(0, 99);
        assert_eq!(slider.steps, 1);
        assert_eq!(slider.layout_steps, 1);
        assert_eq!(slider.selected, 0);
    }

    #[test]
    fn slot_centers_are_symmetric() {
        let first = slot_center_x(0, 4, 20.0, 8.0);
        let last = slot_center_x(3, 4, 20.0, 8.0);
        assert_eq!(first, -last);
    }

    #[test]
    fn slot_span_bounds_tracks_visible_steps() {
        let (left, right) = slot_span_bounds(3, 4, 20.0, 8.0);
        assert_eq!(left, -52.0);
        assert_eq!(right, 24.0);
    }

    #[test]
    fn slider_insertion_adds_required_components() {
        let mut world = World::new();
        let slider = world.spawn(DiscreteSlider::new(3, 1)).id();

        assert!(world.entity(slider).get::<Transform>().is_some());
        assert!(world.entity(slider).get::<Visibility>().is_some());
    }

    #[test]
    fn plugin_builds_slider_slots_without_menu_module() {
        let mut app = App::new();
        app.add_plugins(DiscreteSliderPlugin);
        let slider = app
            .world_mut()
            .spawn(DiscreteSlider::new(4, 2))
            .id();

        app.update();

        let children: Vec<Entity> = app
            .world()
            .entity(slider)
            .get::<Children>()
            .map(|children| children.iter().collect())
            .unwrap_or_default();
        let slot_count = children
            .iter()
            .filter(|entity| app.world().entity(**entity).contains::<DiscreteSliderSlot>())
            .count();
        assert_eq!(slot_count, 4);
    }

    #[test]
    fn insert_hook_seeds_slot_children_without_running_update() {
        let mut world = World::new();
        let slider = world
            .spawn(DiscreteSlider::new(3, 1))
            .id();

        let slot_count = world
            .entity(slider)
            .get::<Children>()
            .map(|children| {
                children
                    .iter()
                    .filter(|child| world.entity(*child).contains::<DiscreteSliderSlot>())
                    .count()
            })
            .unwrap_or(0);
        assert_eq!(slot_count, 3);
    }
}
