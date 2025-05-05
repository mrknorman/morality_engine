use bevy::{ecs::{component::HookContext, world::DeferredWorld}, prelude::*};
use crate::systems::colors::PRIMARY_COLOR;

pub struct CompoundPlugin;
impl Plugin for CompoundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                (
                    propagate_changes::<HollowRectangle>,
                    propagate_changes::<Plus>,
                    BorderedRectangle::propagate_changes,
                )
            );
    }
}

// Define a trait for shapes that assemble sprite children.
pub trait AssembleShape {
    type Side: Component + Default;

    fn color(&self) -> Color;    
    fn color_mut(&mut self) -> &mut Color;

    // Default color setter implementation.
    fn set_color(&mut self, new_color: Color) {
        *self.color_mut() = new_color;
    }

    fn dimensions(&self) -> Vec2;

    // Returns an iterator of (size, translation) updates.
    fn updates(&self) -> Vec<(Vec2, Vec3)>;

    // Default method to create sprite bundles for the sides.
    fn assemble(&self) -> impl Iterator<Item = (Sprite, Transform, Self::Side)> {
        let color = self.color();
        self.updates().into_iter().map(move |(size, pos)| (
            Sprite {
                custom_size: Some(size),
                color,
                ..default()
            },
            Transform::from_translation(pos),
            Self::Side::default(),
        ))
    }
}

pub fn propagate_changes<T: AssembleShape + Component>(
    shape_query: Query<(Entity, &T), Changed<T>>,
    children_query: Query<&Children>,
    mut side_query: Query<(&mut Sprite, &mut Transform), With<T::Side>>,
) {
    for (entity, shape) in shape_query.iter() {
        if let Ok(children) = children_query.get(entity) {
            let color = shape.color();
            let updates = shape.updates();
            // Assuming the number of children matches the number of updates
            for (child, (size, pos)) in children.iter().zip(updates.iter()) {
                if let Ok((mut sprite, mut transform)) = side_query.get_mut(child) {
                    sprite.custom_size = Some(size.max(Vec2::ZERO));
                    sprite.color = color;
                    *transform = Transform::from_translation(*pos);
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct RectangleSides {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

impl Default for RectangleSides {
    fn default() -> Self {
        Self {
            top: true,
            bottom: true,
            left: true,
            right: true,
        }
    }
}

impl HollowRectangle {
    fn on_insert(
        mut world: DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
        let shape = {
            if let Some(shape) = world.entity(entity).get::<Self>() {
                shape.clone()
            } else {
                return;
            }
        };
        
        world.commands().entity(entity).with_children(|parent| {
            shape.assemble().for_each(|sprite_bundle| { parent.spawn(sprite_bundle); });
        });
    }
}

#[derive(Clone, Copy, Component)]
#[component(on_insert = HollowRectangle::on_insert)]
#[require(Transform, Visibility)]
pub struct HollowRectangle {
    pub dimensions: Vec2,
    pub thickness: f32,
    pub color: Color,
    pub sides: RectangleSides, // New field to control which sides are drawn.
}

impl Default for HollowRectangle {
    fn default() -> Self {
        Self {
            dimensions: Vec2::ZERO,
            thickness: 2.0,
            color: PRIMARY_COLOR,
            sides: RectangleSides::default(),
        }
    }
}

#[derive(Component, Default)]
pub struct RectangleSide;

impl AssembleShape for HollowRectangle {
    type Side = RectangleSide;

    fn color(&self) -> Color {
        self.color
    }
    fn color_mut(&mut self) -> &mut Color {
        &mut self.color
    }
    fn dimensions(&self) -> Vec2 {
        self.dimensions
    }
    fn updates(&self) -> Vec<(Vec2, Vec3)> {
        let width = self.dimensions.x;
        let height = self.dimensions.y;
        let thickness = self.thickness;
        let half_width = width / 2.0;
        let half_height = height / 2.0;
        let mut updates = Vec::new();
        
        if self.sides.top {
            updates.push((Vec2::new(width, thickness), Vec3::ZERO.with_y(half_height)));
        }
        if self.sides.bottom {
            updates.push((Vec2::new(width, thickness), Vec3::ZERO.with_y(-half_height)));
        }
        if self.sides.left {
            updates.push((Vec2::new(thickness, height), Vec3::ZERO.with_x(-half_width)));
        }
        if self.sides.right {
            updates.push((Vec2::new(thickness, height), Vec3::ZERO.with_x(half_width)));
        }
        updates
    }
}
#[derive(Clone, Component)]
#[component(on_insert = Plus::on_insert)]
#[require(Transform, Visibility)]
pub struct Plus {
    pub dimensions: Vec2,
    pub thickness: f32,
    pub color: Color,
}

impl Plus {
    fn on_insert(
        mut world: DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
        let shape = {
            if let Some(shape) = world.entity(entity).get::<Self>() {
                shape.clone()
            } else {
                return;
            }
        };
        
        world.commands().entity(entity).with_children(|parent| {
            shape.assemble().for_each(|sprite_bundle| { parent.spawn(sprite_bundle); });
        });
    }
}

#[derive(Component, Default)]
pub struct PlusSide;

impl AssembleShape for Plus {
    type Side = PlusSide;
    
    fn color(&self) -> Color {
        self.color
    }
    fn color_mut(&mut self) -> &mut Color {
        &mut self.color
    }
    fn dimensions(&self) -> Vec2 {
        self.dimensions
    }
    fn updates(&self) -> Vec<(Vec2, Vec3)> {
        let width = self.dimensions.x;
        let height = self.dimensions.y;
        let thickness = self.thickness;

        vec![
            (Vec2::new(width, thickness), Vec3::ZERO),
            (Vec2::new(thickness, height), Vec3::ZERO),
        ]
    }
}

#[derive(Clone, Copy, Component)]
#[component(on_insert = BorderedRectangle::on_insert)]
#[require(Transform, Visibility)]
pub struct BorderedRectangle {
    pub boundary: HollowRectangle,
}

impl Default for BorderedRectangle {
    fn default() -> Self {
        Self { boundary: HollowRectangle::default() }
    }
}

#[derive(Component)]
pub struct RectangleBackground;

#[derive(Component)]
pub struct RectangleBorder;

impl BorderedRectangle {
    fn propagate_changes(
        rectangle_query: Query<(Entity, &BorderedRectangle), Changed<BorderedRectangle>>,
        children_query: Query<&Children>,
        mut background_query: Query<&mut Mesh2d, With<RectangleBackground>>,
        mut border_query: Query<&mut HollowRectangle, With<RectangleBorder>>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        for (entity, bordered_rectangle) in &rectangle_query {
            if let Ok(children) = children_query.get(entity) {
                let width = bordered_rectangle.boundary.dimensions.x;
                let height = bordered_rectangle.boundary.dimensions.y;
                
                for child in children.iter() {
                    if let Ok(mut mesh2d) = background_query.get_mut(child) {
                        mesh2d.0 = meshes.add(Mesh::from(Rectangle::new(width, height)));
                    }
                    
                    if let Ok(mut hollow_rectangle) = border_query.get_mut(child) {
                        *hollow_rectangle = bordered_rectangle.boundary;
                    }
                }
            }
        }
    }

    fn on_insert(
        mut world: DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
        let (boundary, width, height) = {
            if let Some(bordered_rectangle) = world.entity(entity).get::<BorderedRectangle>() {
                (
                    bordered_rectangle.boundary,
                    bordered_rectangle.boundary.dimensions.x,
                    bordered_rectangle.boundary.dimensions.y,
                )
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
                boundary
            ));
        });
    }
}