use std::collections::{HashMap, HashSet};

use bevy::{
    asset::RenderAssetUsages,
    camera::{visibility::RenderLayers, ClearColorConfig, RenderTarget},
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::Hdr,
    },
};

use super::{
    geometry::viewport_texture_size, ScrollBackend, ScrollLayerManaged, ScrollLayerPool,
    ScrollRenderExhaustionPolicy, ScrollRenderSettings, ScrollableContent,
    ScrollableContentCamera, ScrollableRenderTarget, ScrollableRoot, ScrollableSurface,
    ScrollableViewport, SCROLL_CAMERA_Z, SCROLL_LAYER_COUNT, SCROLL_SURFACE_Z,
};

fn create_scroll_target_image(size_px: UVec2, format: TextureFormat) -> Image {
    let size = Extent3d {
        width: size_px.x,
        height: size_px.y,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0u8; 16],
        format,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
        | TextureUsages::COPY_DST
        | TextureUsages::RENDER_ATTACHMENT;
    image
}

pub(super) fn cleanup_scroll_layer_pool(
    mut layer_pool: ResMut<ScrollLayerPool>,
    root_query: Query<Entity, With<ScrollableRoot>>,
) {
    let live_roots: HashSet<Entity> = root_query.iter().collect();
    layer_pool.release_stale_roots(&live_roots);
}

pub(super) fn ensure_scrollable_render_targets(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut layer_pool: ResMut<ScrollLayerPool>,
    render_settings: Res<ScrollRenderSettings>,
    root_query: Query<
        (Entity, &ScrollableRoot, &ScrollableViewport),
        (With<ScrollableRoot>, Without<ScrollableRenderTarget>),
    >,
) {
    let max_targets = render_settings
        .max_render_targets
        .max(1)
        .min(SCROLL_LAYER_COUNT as usize);
    let mut roots: Vec<(Entity, UVec2)> = root_query
        .iter()
        .filter_map(|(entity, root, viewport)| {
            matches!(root.backend, ScrollBackend::RenderToTexture)
                .then_some((entity, viewport_texture_size(viewport.size)))
        })
        .collect();
    roots.sort_by_key(|(entity, _)| entity.to_bits());

    for (root_entity, size_px) in roots {
        let Some(layer) = layer_pool.layer_for_root(root_entity, max_targets) else {
            if matches!(
                render_settings.exhaustion_policy,
                ScrollRenderExhaustionPolicy::WarnAndSkipRoot
            ) {
                warn!(
                    "Scroll RTT budget exhausted (max_targets={}): skipping root {:?}",
                    max_targets, root_entity
                );
            }
            continue;
        };
        let format = render_settings.target_format;
        let image = images.add(create_scroll_target_image(size_px, format));
        commands.entity(root_entity).insert(ScrollableRenderTarget {
            image,
            size_px,
            layer,
            format,
        });
    }
}

pub(super) fn sync_scrollable_render_targets(
    mut images: ResMut<Assets<Image>>,
    render_settings: Res<ScrollRenderSettings>,
    mut root_query: Query<
        (&ScrollableRoot, &ScrollableViewport, &mut ScrollableRenderTarget),
        With<ScrollableRoot>,
    >,
) {
    let required_format = render_settings.target_format;
    for (root, viewport, mut render_target) in root_query.iter_mut() {
        if !matches!(root.backend, ScrollBackend::RenderToTexture) {
            continue;
        }
        let required_size = viewport_texture_size(viewport.size);
        if render_target.size_px == required_size && render_target.format == required_format {
            continue;
        }

        render_target.size_px = required_size;
        render_target.format = required_format;
        render_target.image = images.add(create_scroll_target_image(required_size, required_format));
    }
}

pub(super) fn ensure_scrollable_runtime_entities(
    mut commands: Commands,
    root_query: Query<(Entity, &ScrollableRenderTarget, Option<&Children>), With<ScrollableRoot>>,
    camera_marker_query: Query<(), With<ScrollableContentCamera>>,
    surface_marker_query: Query<(), With<ScrollableSurface>>,
    content_marker_query: Query<(), With<ScrollableContent>>,
) {
    // Query contract:
    // - root iteration is read-only (`root_query`).
    // - camera/surface/content marker queries are read-only existence checks.
    // - all entity creation and parent mutation occurs through `Commands`,
    //   keeping this system B0001-safe.
    for (root_entity, render_target, children) in root_query.iter() {
        let mut has_camera = false;
        let mut has_surface = false;
        let mut has_content = false;

        if let Some(children) = children {
            for child in children.iter() {
                if camera_marker_query.get(child).is_ok() {
                    has_camera = true;
                }
                if surface_marker_query.get(child).is_ok() {
                    has_surface = true;
                }
                if content_marker_query.get(child).is_ok() {
                    has_content = true;
                }
            }
        }

        if !has_camera {
            commands.entity(root_entity).with_children(|parent| {
                parent.spawn((
                    Name::new("scrollable_content_camera"),
                    Camera2d,
                    ScrollableContentCamera { root: root_entity },
                    RenderLayers::layer(render_target.layer as usize),
                    Hdr,
                    Camera {
                        clear_color: ClearColorConfig::Custom(Color::NONE),
                        ..default()
                    },
                    RenderTarget::Image(render_target.image.clone().into()),
                    Transform::from_xyz(0.0, 0.0, SCROLL_CAMERA_Z),
                ));
            });
        }

        if !has_surface {
            commands.entity(root_entity).with_children(|parent| {
                parent.spawn((
                    Name::new("scrollable_surface"),
                    ScrollableSurface { root: root_entity },
                    Sprite::from_image(render_target.image.clone()),
                    Transform::from_xyz(0.0, 0.0, SCROLL_SURFACE_Z),
                ));
            });
        }

        if !has_content {
            commands.entity(root_entity).with_children(|parent| {
                parent.spawn((
                    Name::new("scrollable_content"),
                    ScrollableContent,
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Visible,
                ));
            });
        }
    }
}

pub(super) fn sync_scrollable_render_entities(
    root_query: Query<(Entity, &ScrollableViewport, &ScrollableRenderTarget), With<ScrollableRoot>>,
    mut camera_query: Query<
        (
            &ScrollableContentCamera,
            &mut Camera,
            &mut RenderLayers,
            &mut RenderTarget,
        ),
    >,
    mut surface_query: Query<(&ScrollableSurface, &mut Sprite, &mut Transform, &mut Visibility)>,
) {
    // Query contract:
    // - root state is read-only and cached into `root_map`.
    // - camera/surface mutations are isolated by marker components
    //   (`ScrollableContentCamera` vs `ScrollableSurface`), avoiding overlap on
    //   mutable render components.
    let mut root_map = HashMap::new();
    for (root_entity, viewport, render_target) in root_query.iter() {
        root_map.insert(
            root_entity,
            (viewport.size, render_target.image.clone(), render_target.layer),
        );
    }

    for (marker, mut camera, mut layers, mut target) in camera_query.iter_mut() {
        let Some((_, image, layer)) = root_map.get(&marker.root) else {
            continue;
        };
        *layers = RenderLayers::layer(*layer as usize);
        *target = RenderTarget::Image(image.clone().into());
        camera.clear_color = ClearColorConfig::Custom(Color::NONE);
    }

    for (marker, mut sprite, mut transform, mut visibility) in surface_query.iter_mut() {
        let Some((size, image, _)) = root_map.get(&marker.root) else {
            continue;
        };
        if sprite.image != *image {
            sprite.image = image.clone();
        }
        sprite.custom_size = Some(*size);
        transform.translation.z = SCROLL_SURFACE_Z;
        *visibility = if size.x <= 0.0 || size.y <= 0.0 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

pub(super) fn sync_scroll_content_layers(
    mut commands: Commands,
    root_query: Query<(Entity, &ScrollableRenderTarget), With<ScrollableRoot>>,
    content_query: Query<Entity, (With<ScrollableContent>, With<ChildOf>)>,
    content_parent_query: Query<&ChildOf, With<ScrollableContent>>,
    children_query: Query<&Children>,
    layer_query: Query<(Option<&RenderLayers>, Option<&ScrollLayerManaged>)>,
) {
    // Query contract:
    // - all source queries are read-only snapshots (`root_query`, `content_query`,
    //   `content_parent_query`, `children_query`, `layer_query`).
    // - layer updates are applied through `Commands`, so no mutable query aliasing
    //   occurs while traversing the content subtree.
    let layer_by_root: HashMap<Entity, u8> = root_query
        .iter()
        .map(|(entity, target)| (entity, target.layer))
        .collect();

    for content_entity in content_query.iter() {
        let Ok(parent) = content_parent_query.get(content_entity) else {
            continue;
        };
        let Some(layer) = layer_by_root.get(&parent.parent()).copied() else {
            continue;
        };
        let target_layers = RenderLayers::layer(layer as usize);
        let mut stack = vec![content_entity];
        while let Some(entity) = stack.pop() {
            let Ok((existing_layers, managed)) = layer_query.get(entity) else {
                continue;
            };
            let should_manage =
                entity == content_entity || managed.is_some() || existing_layers.is_none();
            if should_manage {
                let already_synced = managed.is_some_and(|_| {
                    existing_layers
                        .is_some_and(|existing_layers| *existing_layers == target_layers)
                });
                if !already_synced {
                    commands
                        .entity(entity)
                        .insert((target_layers.clone(), ScrollLayerManaged));
                }
            }
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    stack.push(child);
                }
            }
        }
    }
}
