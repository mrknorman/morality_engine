use bevy::{
    asset::Assets,
    camera::visibility::RenderLayers,
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    render::render_resource::TextureFormat,
};

use crate::{
    startup::cursor::CustomCursor,
    systems::ui::layer::{UiLayer, UiLayerKind},
};

use super::{
    scrollbar_math::{offset_from_thumb_center, thumb_center_for_offset, thumb_extent_for_state},
    ScrollAxis, ScrollPlugin, ScrollRenderSettings, ScrollState, ScrollableContent,
    ScrollableContentExtent, ScrollableRoot, ScrollableViewport,
};

fn make_scroll_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<MouseWheel>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.init_resource::<CustomCursor>();
    app.init_resource::<Assets<Image>>();
    app.add_plugins(ScrollPlugin);
    app
}

#[test]
fn default_scroll_render_target_format_is_rgba16float() {
    let settings = ScrollRenderSettings::default();
    assert_eq!(settings.target_format, TextureFormat::Rgba16Float);
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
    let thumb = thumb_extent_for_state(120.0, 20.0, 300.0, 18.0);
    assert_eq!(thumb, 18.0);

    let thumb = thumb_extent_for_state(120.0, 500.0, 300.0, 18.0);
    assert_eq!(thumb, 120.0);
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
