use bevy::{
    asset::Assets,
    camera::visibility::RenderLayers,
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    render::render_resource::TextureFormat,
};

use crate::{
    startup::cursor::CustomCursor,
    systems::interaction::InteractionGate,
    systems::ui::layer::{UiLayer, UiLayerKind},
};

use super::{
    scrollbar_math::{offset_from_thumb_center, thumb_center_for_offset, thumb_extent_for_state},
    ScrollAxis, ScrollBar, ScrollBarDragState, ScrollBarParts, ScrollBarThumb, ScrollBarTrack,
    ScrollPlugin, ScrollRenderSettings, ScrollState, ScrollableContent, ScrollableContentCamera,
    ScrollableContentExtent, ScrollableListAdapter, ScrollableRenderTarget, ScrollableRoot,
    ScrollableSurface, ScrollableViewport,
};

fn make_scroll_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<MouseWheel>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.init_resource::<CustomCursor>();
    app.init_resource::<Assets<Image>>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<ColorMaterial>>();
    app.add_plugins(ScrollPlugin);
    app
}

#[test]
fn default_scroll_render_target_format_is_rgba16float() {
    let settings = ScrollRenderSettings::default();
    assert_eq!(settings.target_format, TextureFormat::Rgba16Float);
}

#[derive(Clone, Copy, Debug)]
struct TestListItem;

#[test]
fn scrollable_list_adapter_new_sets_expected_fields() {
    let owner = Entity::from_bits(42);
    let adapter = ScrollableListAdapter::<TestListItem>::new(owner, 12, 28.0, 6.0);
    assert_eq!(adapter.owner, owner);
    assert_eq!(adapter.item_count, 12);
    assert_eq!(adapter.item_extent, 28.0);
    assert_eq!(adapter.leading_padding, 6.0);
}

#[test]
fn scrollbar_insertion_adds_required_components() {
    let mut app = make_scroll_test_app();
    let root = app.world_mut().spawn_empty().id();
    let scrollbar = app.world_mut().spawn(ScrollBar::new(root)).id();

    assert!(app.world().entity(scrollbar).get::<Transform>().is_some());
    assert!(app.world().entity(scrollbar).get::<Visibility>().is_some());
    assert!(app.world().entity(scrollbar).get::<ScrollBarDragState>().is_some());
}

#[test]
fn scrollbar_insert_hook_seeds_parts_and_parents_to_scroll_root() {
    let mut app = make_scroll_test_app();
    let owner = app.world_mut().spawn_empty().id();
    let root = app
        .world_mut()
        .spawn((
            ScrollableRoot::new(owner, ScrollAxis::Vertical),
            InteractionGate::PauseMenuOnly,
        ))
        .id();
    let scrollbar = app.world_mut().spawn(ScrollBar::new(root)).id();

    let parent = app
        .world()
        .entity(scrollbar)
        .get::<ChildOf>()
        .expect("scrollbar parent");
    assert_eq!(parent.parent(), root);

    let parts = app
        .world()
        .entity(scrollbar)
        .get::<ScrollBarParts>()
        .copied()
        .expect("scrollbar parts");
    assert!(app.world().entity(parts.track).contains::<ScrollBarTrack>());
    assert!(app.world().entity(parts.thumb).contains::<ScrollBarThumb>());
    assert_eq!(
        app.world().entity(parts.track).get::<InteractionGate>(),
        Some(&InteractionGate::PauseMenuOnly),
    );
    assert_eq!(
        app.world().entity(parts.thumb).get::<InteractionGate>(),
        Some(&InteractionGate::PauseMenuOnly),
    );
}

#[test]
fn ensure_scrollbar_parts_rebuilds_when_part_entities_are_stale() {
    let mut app = make_scroll_test_app();
    let owner = app.world_mut().spawn_empty().id();
    let root = app
        .world_mut()
        .spawn((
            ScrollableRoot::new(owner, ScrollAxis::Vertical),
            InteractionGate::PauseMenuOnly,
        ))
        .id();
    let scrollbar = app.world_mut().spawn(ScrollBar::new(root)).id();
    let original_parts = app
        .world()
        .entity(scrollbar)
        .get::<ScrollBarParts>()
        .copied()
        .expect("initial parts");

    app.world_mut().entity_mut(original_parts.track).despawn();
    app.update();

    let repaired_parts = app
        .world()
        .entity(scrollbar)
        .get::<ScrollBarParts>()
        .copied()
        .expect("repaired parts");
    assert_ne!(repaired_parts.track, original_parts.track);
    assert!(app.world().entity(repaired_parts.track).contains::<ScrollBarTrack>());
    assert!(app.world().entity(repaired_parts.thumb).contains::<ScrollBarThumb>());
}

#[test]
fn scroll_root_runtime_ensure_seeds_content_camera_and_surface_children() {
    let mut app = make_scroll_test_app();
    let owner = app.world_mut().spawn_empty().id();
    let root = app
        .world_mut()
        .spawn((
            ScrollableRoot::new(owner, ScrollAxis::Vertical),
            ScrollableViewport::new(Vec2::new(180.0, 120.0)),
        ))
        .id();

    app.update();

    let children = app
        .world()
        .entity(root)
        .get::<Children>()
        .expect("scroll root children");
    let mut has_camera = false;
    let mut has_surface = false;
    let mut has_content = false;
    for child in children.iter() {
        let child_ref = app.world().entity(child);
        has_camera |= child_ref.contains::<ScrollableContentCamera>();
        has_surface |= child_ref.contains::<ScrollableSurface>();
        has_content |= child_ref.contains::<ScrollableContent>();
    }

    assert!(has_camera);
    assert!(has_surface);
    assert!(has_content);
}

#[test]
fn render_target_budget_skips_new_roots_after_limit() {
    let mut app = make_scroll_test_app();
    app.world_mut()
        .resource_mut::<ScrollRenderSettings>()
        .max_render_targets = 1;

    let owner_a = app.world_mut().spawn_empty().id();
    let root_a = app
        .world_mut()
        .spawn((
            ScrollableRoot::new(owner_a, ScrollAxis::Vertical),
            ScrollableViewport::new(Vec2::new(120.0, 90.0)),
        ))
        .id();

    let owner_b = app.world_mut().spawn_empty().id();
    let root_b = app
        .world_mut()
        .spawn((
            ScrollableRoot::new(owner_b, ScrollAxis::Vertical),
            ScrollableViewport::new(Vec2::new(120.0, 90.0)),
        ))
        .id();

    app.update();

    let has_target_a = app.world().entity(root_a).contains::<ScrollableRenderTarget>();
    let has_target_b = app.world().entity(root_b).contains::<ScrollableRenderTarget>();
    let target_count = [has_target_a, has_target_b]
        .into_iter()
        .filter(|has_target| *has_target)
        .count();

    assert_eq!(target_count, 1);
    assert!(has_target_a || has_target_b);
}

fn spawn_owner_and_scroll_root(
    app: &mut App,
    content_extent: f32,
    viewport_extent: f32,
) -> (Entity, Entity) {
    let owner = app.world_mut().spawn_empty().id();
    let root = app
        .world_mut()
        .spawn((
            ScrollableRoot::new(owner, ScrollAxis::Vertical),
            ScrollableViewport::new(Vec2::new(160.0, viewport_extent)),
            ScrollableContentExtent(content_extent),
            ScrollState {
                offset_px: 0.0,
                content_extent,
                viewport_extent,
                max_offset: (content_extent - viewport_extent).max(0.0),
            },
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();
    (owner, root)
}

fn write_wheel(app: &mut App, y: f32) {
    app.world_mut().write_message(MouseWheel {
        unit: MouseScrollUnit::Line,
        x: 0.0,
        y,
        window: Entity::PLACEHOLDER,
    });
}

#[test]
fn thumb_extent_respects_minimum_and_track_bounds() {
    let cases = [
        (120.0, 20.0, 300.0, 18.0, 18.0),
        (120.0, 500.0, 300.0, 18.0, 120.0),
    ];
    for (track_extent, viewport_extent, content_extent, min_thumb_extent, expected) in cases {
        let thumb =
            thumb_extent_for_state(track_extent, viewport_extent, content_extent, min_thumb_extent);
        assert_eq!(thumb, expected);
    }
}

#[test]
fn vertical_thumb_round_trip_maps_offset_consistently() {
    let track = 180.0;
    let thumb = 36.0;
    let max = 400.0;
    let offset = 215.0;
    let center = thumb_center_for_offset(track, thumb, offset, max, ScrollAxis::Vertical);
    let mapped = offset_from_thumb_center(track, thumb, center, max, ScrollAxis::Vertical);

    assert!((mapped - offset).abs() < 0.001);
}

#[test]
fn horizontal_thumb_round_trip_maps_offset_consistently() {
    let track = 260.0;
    let thumb = 42.0;
    let max = 900.0;
    let offset = 510.0;
    let center = thumb_center_for_offset(track, thumb, offset, max, ScrollAxis::Horizontal);
    let mapped = offset_from_thumb_center(track, thumb, center, max, ScrollAxis::Horizontal);

    assert!((mapped - offset).abs() < 0.001);
}

#[test]
fn vertical_thumb_round_trip_property_is_bounded_samples() {
    let tracks: [f32; 5] = [24.0, 72.0, 144.0, 288.0, 400.0];
    let thumb_ratios: [f32; 5] = [0.15, 0.3, 0.5, 0.75, 0.95];
    let max_values: [f32; 5] = [1.0, 25.0, 250.0, 1500.0, 5000.0];
    let normalized_values: [f32; 5] = [0.0, 0.1, 0.33, 0.66, 1.0];
    for track in tracks {
        for thumb_ratio in thumb_ratios {
            for max in max_values {
                for normalized in normalized_values {
                    let thumb = (track * thumb_ratio).clamp(1.0, track);
                    let offset = max * normalized;
                    let center =
                        thumb_center_for_offset(track, thumb, offset, max, ScrollAxis::Vertical);
                    let mapped =
                        offset_from_thumb_center(track, thumb, center, max, ScrollAxis::Vertical);
                    assert!(
                        (mapped - offset).abs() <= 0.01,
                        "vertical round-trip drift: track={track}, thumb={thumb}, max={max}, normalized={normalized}, offset={offset}, mapped={mapped}",
                    );
                }
            }
        }
    }
}

#[test]
fn horizontal_thumb_round_trip_property_is_bounded_samples() {
    let tracks: [f32; 5] = [24.0, 72.0, 144.0, 288.0, 400.0];
    let thumb_ratios: [f32; 5] = [0.15, 0.3, 0.5, 0.75, 0.95];
    let max_values: [f32; 5] = [1.0, 25.0, 250.0, 1500.0, 5000.0];
    let normalized_values: [f32; 5] = [0.0, 0.1, 0.33, 0.66, 1.0];
    for track in tracks {
        for thumb_ratio in thumb_ratios {
            for max in max_values {
                for normalized in normalized_values {
                    let thumb = (track * thumb_ratio).clamp(1.0, track);
                    let offset = max * normalized;
                    let center = thumb_center_for_offset(
                        track,
                        thumb,
                        offset,
                        max,
                        ScrollAxis::Horizontal,
                    );
                    let mapped = offset_from_thumb_center(
                        track,
                        thumb,
                        center,
                        max,
                        ScrollAxis::Horizontal,
                    );
                    assert!(
                        (mapped - offset).abs() <= 0.01,
                        "horizontal round-trip drift: track={track}, thumb={thumb}, max={max}, normalized={normalized}, offset={offset}, mapped={mapped}",
                    );
                }
            }
        }
    }
}

#[test]
fn thumb_center_is_clamped_by_scroll_range() {
    let top = thumb_center_for_offset(200.0, 40.0, 0.0, 100.0, ScrollAxis::Vertical);
    let bottom = thumb_center_for_offset(200.0, 40.0, 100.0, 100.0, ScrollAxis::Vertical);
    let beyond = thumb_center_for_offset(200.0, 40.0, 1000.0, 100.0, ScrollAxis::Vertical);

    assert!(top > bottom);
    assert_eq!(bottom, beyond);
}

#[test]
fn scroll_plugin_update_is_query_safe() {
    let mut app = make_scroll_test_app();
    app.update();
}

#[test]
fn base_layer_allows_scroll_input() {
    let mut app = make_scroll_test_app();
    let (owner, root) = spawn_owner_and_scroll_root(&mut app, 300.0, 100.0);
    app.world_mut()
        .spawn((UiLayer::new(owner, UiLayerKind::Base), Visibility::Visible));
    app.world_mut().resource_mut::<CustomCursor>().position = Some(Vec2::ZERO);
    write_wheel(&mut app, -1.0);
    app.update();

    let state = app.world().get::<ScrollState>(root).expect("scroll state");
    assert!(state.offset_px > 0.0);
}

#[test]
fn dropdown_layer_blocks_scroll_input() {
    let mut app = make_scroll_test_app();
    let (owner, root) = spawn_owner_and_scroll_root(&mut app, 300.0, 100.0);
    app.world_mut()
        .spawn((UiLayer::new(owner, UiLayerKind::Base), Visibility::Visible));
    app.world_mut().spawn((
        UiLayer::new(owner, UiLayerKind::Dropdown),
        Visibility::Visible,
    ));
    app.world_mut().resource_mut::<CustomCursor>().position = Some(Vec2::ZERO);
    write_wheel(&mut app, -1.0);
    app.update();

    let state = app.world().get::<ScrollState>(root).expect("scroll state");
    assert_eq!(state.offset_px, 0.0);
}

#[test]
fn modal_layer_blocks_base_scoped_scroll_input() {
    let mut app = make_scroll_test_app();
    let (owner, root) = spawn_owner_and_scroll_root(&mut app, 300.0, 100.0);
    app.world_mut()
        .spawn((UiLayer::new(owner, UiLayerKind::Base), Visibility::Visible));
    app.world_mut()
        .spawn((UiLayer::new(owner, UiLayerKind::Modal), Visibility::Visible));
    app.world_mut().resource_mut::<CustomCursor>().position = Some(Vec2::ZERO);
    write_wheel(&mut app, -1.0);
    app.update();

    let state = app.world().get::<ScrollState>(root).expect("scroll state");
    assert_eq!(state.offset_px, 0.0);
}

#[test]
fn base_layer_blocks_modal_scoped_scroll_input() {
    let mut app = make_scroll_test_app();
    let (owner, root) = spawn_owner_and_scroll_root(&mut app, 300.0, 100.0);
    let updated_root = app
        .world()
        .get::<ScrollableRoot>(root)
        .copied()
        .expect("scroll root")
        .with_input_layer(UiLayerKind::Modal);
    app.world_mut().entity_mut(root).insert(updated_root);
    app.world_mut()
        .spawn((UiLayer::new(owner, UiLayerKind::Base), Visibility::Visible));
    app.world_mut().resource_mut::<CustomCursor>().position = Some(Vec2::ZERO);
    write_wheel(&mut app, -1.0);
    app.update();

    let state = app.world().get::<ScrollState>(root).expect("scroll state");
    assert_eq!(state.offset_px, 0.0);
}

#[test]
fn mixed_keyboard_and_wheel_input_is_deterministic() {
    let mut app = make_scroll_test_app();
    let (owner, root) = spawn_owner_and_scroll_root(&mut app, 300.0, 100.0);
    app.world_mut()
        .spawn((UiLayer::new(owner, UiLayerKind::Base), Visibility::Visible));
    app.world_mut().resource_mut::<CustomCursor>().position = Some(Vec2::ZERO);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowDown);
    write_wheel(&mut app, -1.0);
    app.update();

    let state = app.world().get::<ScrollState>(root).expect("scroll state");
    assert!((state.offset_px - 76.0).abs() < 0.001);
}

#[test]
fn stress_layer_toggle_keeps_scroll_state_clamped_and_panics_free() {
    let mut app = make_scroll_test_app();
    let (owner, root) = spawn_owner_and_scroll_root(&mut app, 420.0, 100.0);
    let base = app
        .world_mut()
        .spawn((UiLayer::new(owner, UiLayerKind::Base), Visibility::Visible))
        .id();
    let dropdown = app
        .world_mut()
        .spawn((UiLayer::new(owner, UiLayerKind::Dropdown), Visibility::Hidden))
        .id();
    let modal = app
        .world_mut()
        .spawn((UiLayer::new(owner, UiLayerKind::Modal), Visibility::Hidden))
        .id();
    app.world_mut().resource_mut::<CustomCursor>().position = Some(Vec2::ZERO);

    for step in 0..48usize {
        let phase = step % 3;
        *app.world_mut()
            .get_mut::<Visibility>(base)
            .expect("base visibility") = if phase == 0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        *app.world_mut()
            .get_mut::<Visibility>(dropdown)
            .expect("dropdown visibility") = if phase == 1 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        *app.world_mut()
            .get_mut::<Visibility>(modal)
            .expect("modal visibility") = if phase == 2 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        {
            let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keyboard.reset_all();
            if phase == 0 && step % 2 == 0 {
                keyboard.press(KeyCode::ArrowDown);
            }
        }
        write_wheel(&mut app, -1.0);

        let before = app
            .world()
            .get::<ScrollState>(root)
            .expect("scroll state")
            .offset_px;
        app.update();
        let state = *app.world().get::<ScrollState>(root).expect("scroll state");
        if phase != 0 {
            assert_eq!(state.offset_px, before);
        }
        assert!(state.offset_px >= 0.0);
        assert!(state.offset_px <= state.max_offset + 0.001);
    }
}

#[test]
fn keyboard_scroll_works_without_cursor_position() {
    let mut app = make_scroll_test_app();
    let (owner, root) = spawn_owner_and_scroll_root(&mut app, 300.0, 100.0);
    app.world_mut()
        .spawn((UiLayer::new(owner, UiLayerKind::Base), Visibility::Visible));
    app.world_mut().resource_mut::<CustomCursor>().position = None;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowDown);

    app.update();

    let state = app.world().get::<ScrollState>(root).expect("scroll state");
    assert!(state.offset_px > 0.0);
}

#[test]
fn modal_layer_can_receive_scroll_input_when_active() {
    let mut app = make_scroll_test_app();
    let (owner, root) = spawn_owner_and_scroll_root(&mut app, 300.0, 100.0);
    let updated_root = app
        .world()
        .get::<ScrollableRoot>(root)
        .copied()
        .expect("scroll root")
        .with_input_layer(UiLayerKind::Modal);
    app.world_mut().entity_mut(root).insert(updated_root);
    app.world_mut()
        .spawn((UiLayer::new(owner, UiLayerKind::Modal), Visibility::Visible));
    app.world_mut().resource_mut::<CustomCursor>().position = Some(Vec2::ZERO);
    write_wheel(&mut app, -1.0);
    app.update();

    let state = app.world().get::<ScrollState>(root).expect("scroll state");
    assert!(state.offset_px > 0.0);
}

#[test]
fn explicit_child_render_layer_is_not_overridden_by_scroll_layer_sync() {
    let mut app = make_scroll_test_app();
    let (_owner, root) = spawn_owner_and_scroll_root(&mut app, 300.0, 100.0);
    let content = app
        .world_mut()
        .spawn((ScrollableContent, Transform::default()))
        .id();
    app.world_mut().entity_mut(root).add_child(content);

    let custom_layer_child = app
        .world_mut()
        .spawn((RenderLayers::layer(31), Transform::default()))
        .id();
    app.world_mut().entity_mut(content).add_child(custom_layer_child);

    app.update();

    let layers = app
        .world()
        .get::<RenderLayers>(custom_layer_child)
        .expect("custom child render layers");
    assert_eq!(*layers, RenderLayers::layer(31));
}

#[test]
fn scrollbar_wheel_and_drag_path_stays_clamped() {
    let mut app = make_scroll_test_app();
    let (owner, root) = spawn_owner_and_scroll_root(&mut app, 520.0, 140.0);
    app.world_mut()
        .spawn((UiLayer::new(owner, UiLayerKind::Base), Visibility::Visible));
    let scrollbar = app.world_mut().spawn(ScrollBar::new(root)).id();

    // Spawn scrollbar parts and sync initial track/thumb geometry.
    app.update();
    let parts = *app
        .world()
        .get::<ScrollBarParts>(scrollbar)
        .expect("scrollbar parts");

    app.world_mut().resource_mut::<CustomCursor>().position = Some(Vec2::ZERO);
    write_wheel(&mut app, -1.0);
    app.update();

    let after_wheel = app.world().get::<ScrollState>(root).expect("scroll state").offset_px;
    assert!(after_wheel > 0.0);

    let track_center = app
        .world()
        .get::<GlobalTransform>(parts.track)
        .expect("track transform")
        .translation()
        .truncate();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.world_mut()
        .get_mut::<ScrollBarDragState>(scrollbar)
        .expect("drag state")
        .dragging = true;
    app.world_mut().resource_mut::<CustomCursor>().position = Some(track_center);
    app.world_mut().resource_mut::<CustomCursor>().position =
        Some(track_center + Vec2::new(0.0, -90.0));
    app.update();
    let state_after_drag = app.world().get::<ScrollState>(root).expect("scroll state");
    assert!(state_after_drag.offset_px >= 0.0);
    assert!(state_after_drag.offset_px <= state_after_drag.max_offset + 0.001);
}
