use bevy::{ecs::component::{ComponentHooks, StorageType}, prelude::*};
use crate::colors::PRIMARY_COLOR;

pub struct CompoundPlugin;
impl Plugin for CompoundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                (
                    HollowRectangle::update,
                    BorderedRectangle::update,
                    Plus::update
                )
            )
            .register_required_components::<HollowRectangle, Transform>()
            .register_required_components::<HollowRectangle, Visibility>()
            .register_required_components::<BorderedRectangle, Transform>()
            .register_required_components::<BorderedRectangle, Visibility>()
            .register_required_components::<Plus, Transform>()
            .register_required_components::<Plus, Visibility>();
    }
}

#[derive(Clone, Copy)]
pub struct HollowRectangle{
    pub dimensions : Vec2,
    pub thickness : f32
}

impl Default for HollowRectangle {
    fn default() -> Self {
        Self{
            dimensions : Vec2::ZERO,
            thickness : 0.0
        }
    }
}

#[derive(Component)]
pub struct RectangleSide;

impl HollowRectangle {
    fn assemble(
        width: f32, 
        height: f32,
        thickness : f32
    ) -> impl Iterator<Item = (Sprite, Transform, RectangleSide)> {
        let half_width = width / 2.0;
        let half_height = height / 2.0;

        [
            (Vec2::new(width, thickness), Vec3::new(0.0, half_height, 0.0)),
            (Vec2::new(width, thickness), Vec3::new(0.0, -half_height, 0.0)),
            (Vec2::new(thickness, height), Vec3::new(-half_width, 0.0, 0.0)),
            (Vec2::new(thickness, height), Vec3::new(half_width, 0.0 , 0.0)),
        ]
        .into_iter()
        .map(|(size, pos)| (
                Sprite {
                    custom_size: Some(size),
                    color: PRIMARY_COLOR,
                    ..default()
                },
                Transform::from_translation(pos),
                RectangleSide
        ))
    }

    pub fn update(
        rectangle_query: Query<(Entity, &HollowRectangle), Changed<HollowRectangle>>,
        children_query: Query<&Children>,
        mut side_query: Query<(&mut Sprite, &mut Transform), With<RectangleSide>>,
    ) {
        for (entity, hollow_rectangle) in &rectangle_query {
            if let Ok(children) = children_query.get(entity) {
                // Get dimensions
                let width = hollow_rectangle.dimensions.x;
                let height = hollow_rectangle.dimensions.y;
                let thickness = hollow_rectangle.thickness;
                let half_width = width / 2.0;
                let half_height = height / 2.0;
                
                // Define the updates for each side
                let updates = [
                    (Vec2::new(width, thickness), Vec3::new(0.0, half_height, 0.0)),
                    (Vec2::new(width, thickness), Vec3::new(0.0, -half_height, 0.0)),
                    (Vec2::new(thickness, height), Vec3::new(-half_width, 0.0, 0.0)),
                    (Vec2::new(thickness, height), Vec3::new(half_width, 0.0, 0.0)),
                ];
                
                // Keep track of which side we're updating
                let mut side_index = 0;
                
                // Update each side
                for &child in children.iter() {
                    if let Ok((mut sprite, mut transform)) = side_query.get_mut(child) {
                        if side_index < updates.len() {
                            // Update side with new size and position
                            let (new_size, new_pos) = updates[side_index];
                            sprite.custom_size = Some(new_size);
                            *transform = Transform::from_translation(new_pos);
                            side_index += 1;
                        }
                    }
                }
            }
        }
    }
}

impl Component for HollowRectangle {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(|mut world, entity, _component_id| {       
            let (width, height, thickness) = {
                if let Some(hollow_rectangle) = world.entity(entity).get::<HollowRectangle>() {
                    (hollow_rectangle.dimensions.x, hollow_rectangle.dimensions.y, hollow_rectangle.thickness)
                } else {
                    return;
                }
            };
            
            world.commands().entity(entity).with_children(|parent| {
                HollowRectangle::assemble(width, height, thickness)
                    .for_each(|sprite_bundle| {parent.spawn(sprite_bundle);});
            });
        });
    }
}


#[derive(Component)]
pub struct PlusSide;

#[derive(Clone)]
pub struct Plus {
    pub dimensions: Vec2,
    pub thickness: f32,
}

impl Plus {
    fn assemble(
        width: f32,
        height: f32,
        thickness: f32
    ) -> impl Iterator<Item = (Sprite, PlusSide)> {
        [
            Vec2::new(width, thickness),
            Vec2::new(thickness, height),
        ]
        .into_iter()
        .map(|size| (
                Sprite {
                    custom_size: Some(size),
                    color: PRIMARY_COLOR,
                    ..default()
                },
                PlusSide
        ))
    }

    pub fn update(
        plus_query: Query<(Entity, &Plus), Changed<Plus>>,
        children_query: Query<&Children>,
        mut side_query: Query<(&mut Sprite, &mut Transform), With<PlusSide>>,
    ) {
        for (entity, plus) in &plus_query {
            if let Ok(children) = children_query.get(entity) {
                // Get dimensions
                let width = plus.dimensions.x;
                let height = plus.dimensions.y;
                let thickness = plus.thickness;
                
                // Define the updates for each side
                let updates = [
                    // Horizontal line (centered)
                    (Vec2::new(width, thickness), Vec3::new(0.0, 0.0, 0.0)),
                    // Vertical line (centered)
                    (Vec2::new(thickness, height), Vec3::new(0.0, 0.0, 0.0)),
                ];
                
                // Keep track of which side we're updating
                let mut side_index = 0;
                
                // Update each side
                for &child in children.iter() {
                    if let Ok((mut sprite, mut transform)) = side_query.get_mut(child) {
                        if side_index < updates.len() {
                            // Update side with new size and position
                            let (new_size, new_pos) = updates[side_index];
                            sprite.custom_size = Some(new_size);
                            *transform = Transform::from_translation(new_pos);
                            side_index += 1;
                        }
                    }
                }
            }
        }
    }
}


impl Component for Plus {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    
    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(|mut world, entity, _component_id| {
            let (width, height, thickness) = {
                if let Some(plus) = world.entity(entity).get::<Plus>() {
                    (plus.dimensions.x, plus.dimensions.y, plus.thickness)
                } else {
                    return;
                }
            };
            
            world.commands().entity(entity).with_children(|parent| {
                Plus::assemble(width, height, thickness).for_each(|sprite| {parent.spawn(sprite);});
            });
        });
    }
}


#[derive(Clone)]
pub struct BorderedRectangle(
    pub HollowRectangle
);

impl Default for BorderedRectangle {
    fn default() -> Self {
        Self(HollowRectangle::default())
    }
}

#[derive(Component)]
pub struct RectangleBackground;

#[derive(Component)]
pub struct RectangleBorder;

impl Component for BorderedRectangle {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(|mut world, entity, _component_id| {
            let (width, height, thickness) = {
                if let Some(bordered_rectangle) = world.entity(entity).get::<BorderedRectangle>() {
                    (bordered_rectangle.0.dimensions.x, bordered_rectangle.0.dimensions.y, bordered_rectangle.0.thickness)
                } else {
                    return;
                }
            };
            
            let (mesh_handle, material_handle) = {
                let mut meshes = world.resource_mut::<Assets<Mesh>>();
                let mesh_handle = meshes.add(Mesh::from(Rectangle::new(width, height)));
                
                let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
                let material_handle = materials.add(Color::BLACK);
                
                (mesh_handle, material_handle)
            };

            world.commands().entity(entity).with_children(|parent| {
                parent.spawn((
                    RectangleBackground,
                    Mesh2d(mesh_handle),
                    MeshMaterial2d(material_handle),
                ));
                
                parent.spawn((
                    RectangleBorder,
                    HollowRectangle{
                        dimensions : Vec2::new(width, height),
                        thickness
                    }
                ));
            });
        });
    }
}

impl BorderedRectangle {
    pub fn new(width : f32, height : f32, thickness : f32) -> Self {
        Self(HollowRectangle{
            dimensions : Vec2::new(width, height),
            thickness
        })
    }

    fn update(
        rectangle_query: Query<(Entity, &BorderedRectangle), Changed<BorderedRectangle>>,
        children_query: Query<&Children>,
        mut background_query: Query<&mut Mesh2d, With<RectangleBackground>>,
        mut border_query: Query<&mut HollowRectangle, With<RectangleBorder>>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        for (entity, bordered_rectangle) in &rectangle_query {
            if let Ok(children) = children_query.get(entity) {
                let width = bordered_rectangle.0.dimensions.x;
                let height = bordered_rectangle.0.dimensions.y;
                
                for &child in children.iter() {
                    if let Ok(mut mesh2d) = background_query.get_mut(child) {
                        mesh2d.0 = meshes.add(Mesh::from(Rectangle::new(width, height)));
                    }
                    
                    if let Ok(mut hollow_rectangle) = border_query.get_mut(child) {
                        hollow_rectangle.dimensions = Vec2::new(width, height);
                        hollow_rectangle.thickness = bordered_rectangle.0.thickness;
                    }
                }
            }
        }
    }
}