use bevy::{ecs::component::{ComponentHooks, StorageType}, prelude::*, sprite::Anchor};
use crate::colors::PRIMARY_COLOR;

pub struct SpritePlugin;
impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(
			Update,
			(
                BorderedBox::update_background,
                BorderedBox::update_border,
                WindowBox::update_body,
                WindowBox::update_header,
                WindowBoxBody::update_box,
                WindowBoxHeader::update_box
        ))
		.register_required_components::<SpriteBox, Transform>()
        .register_required_components::<SpriteBox, Visibility>()
        .register_required_components::<BorderedBox, Transform>()
        .register_required_components::<BorderedBox, Visibility>()
        .register_required_components::<WindowBox, Transform>()
        .register_required_components::<WindowBox, Visibility>()
        .register_required_components::<WindowBoxHeader, Transform>()
        .register_required_components::<WindowBoxHeader, Visibility>()
        .register_required_components::<WindowBoxBody, Transform>()
        .register_required_components::<WindowBoxBody, Visibility>()
        ;
    }
}


pub struct SpriteFactory;

impl SpriteFactory {
    pub fn create_sprite_bundle(
        size: Vec2, 
        position: Vec3
    ) -> (Sprite, Transform) {
        (
            Sprite{
                custom_size: Some(size),
                color : PRIMARY_COLOR,
                ..default()
            },
            Transform::from_xyz(
                position.x,
                position.y, 
                position.z
            )
        )
    }
}

#[derive(Clone)]
pub struct SpriteBox{
    pub width : f32,
    pub height : f32
}

impl SpriteBox {
    pub fn create_sprite_box(
        position: Vec3, 
        width: f32, 
        height: f32
    ) -> Vec<(Sprite, Transform)> {
        let border_thickness = 2.0;

        let sprite_sizes = vec![
            Vec2::new(width, border_thickness),
            Vec2::new(width, border_thickness),  
            Vec2::new(border_thickness, height),
            Vec2::new(border_thickness, height), 
        ];
        
        let sprite_positions = vec![
            Vec3::new(position.x, position.y + height / 2.0, position.z), 
            Vec3::new(position.x, position.y - height / 2.0, position.z), 
            Vec3::new(position.x - width / 2.0, position.y, position.z),  
            Vec3::new(position.x + width / 2.0, position.y, position.z), 
        ];
        
        sprite_sizes.into_iter().zip(sprite_positions.into_iter())
            .map(|(size, pos)| SpriteFactory::create_sprite_bundle(size, pos))
            .collect()
    }
}


impl Component for SpriteBox{
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(

            |mut world, entity, _component_id| {       

                let size: Option<SpriteBox> = {
                    let entity_ref: EntityRef<'_> = world.entity(entity);
                    entity_ref.get::<SpriteBox>().cloned()
                };

				// Step 3: Use commands in a separate scope to spawn the entity
				if let Some(size) = size {
					let mut commands = world.commands();
					commands.entity(entity).with_children(|parent| {
                        for sprite in SpriteBox::create_sprite_box(
                            Vec3::ZERO,
                            size.width,
                            size.height,
                        ) {
                            parent.spawn(sprite);
                        }
                        
					});
				}
            }
        );
    }
}

#[derive(Clone)]
pub struct BorderedBox(pub Vec2);
impl Default for BorderedBox{
    fn default() -> Self {
        Self(Vec2::new(0.0, 0.0))
    }
}

#[derive(Component)]
pub struct BorderedBoxBackground;

#[derive(Component)]
pub struct BorderedBoxBorder;

impl Component for BorderedBox {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {       

                let bordered_box: Option<BorderedBox> = {
                    let entity_mut = world.entity(entity);
                    entity_mut.get::<BorderedBox>()
                        .map(|bordered_box: &BorderedBox| bordered_box.clone())
                };

                if let Some(bordered_box)= bordered_box {
                    let width = bordered_box.0.x;
                    let height = bordered_box.0.y;

                    // Step 2: Extract mesh and material handles in limited scope
                    let (mesh_handle, material_handle) = {
                        // Create resources within this scope to limit mutable borrows
                        let mut meshes = world.resource_mut::<Assets<Mesh>>();
                        let mesh_handle = meshes.add(Mesh::from(Rectangle::new(width, height)));
                        
                        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
                        let material_handle = materials.add(Color::BLACK);
                        
                        (mesh_handle, material_handle)
                    };
                    
                    // Step 3: Use commands in a separate scope to spawn the entity
                    {
                        let mut commands = world.commands();
                        commands.entity(entity).with_children(|parent| {
                            parent.spawn((
                                BorderedBoxBackground,
                                Mesh2d(mesh_handle),
                                MeshMaterial2d(material_handle),
                            ));
                            
                            parent.spawn(
                                (
                                BorderedBoxBorder,
                                SpriteBox{
                                    width,
                                    height
                                }
                            ));
                        });
                    }
                }
            }
        );
    }
}

impl BorderedBox {
    fn update_background(
        mut box_query: Query<(Entity, &BorderedBox), (Changed<BorderedBox>, Without<BorderedBoxBackground>)>,
        mut background_query: Query<&mut Mesh2d, With<BorderedBoxBackground>>,
        children_query: Query<&Children>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        for (text_entity, bordered_box) in box_query.iter_mut() {
            // Get the text dimensions
            let width = bordered_box.0.x;
            let height = bordered_box.0.y;
            
            // Get the children of this text entity
            if let Ok(children) = children_query.get(text_entity) {
                // Iterate through children to find the background (assuming only one BorderedBoxBackground per TextBox)
                for &child in children.iter() {
                    if let Ok(mut mesh2d) = background_query.get_mut(child) {
                        mesh2d.0 = meshes.add(Mesh::from(Rectangle::new(width, height)));
                    }
                }
            }
        }
    }

    fn update_border(
        mut commands : Commands,
        mut text_query: Query<(Entity, &BorderedBox), (Changed<BorderedBox>, Without<BorderedBoxBorder>)>,
        mut border_query: Query<Entity, With<BorderedBoxBorder>>,
        children_query: Query<&Children>
    ) {
        for (text_entity, bordered_box) in text_query.iter_mut() {
            // Get the text dimensions from TextBounds if available, otherwise fallback to TextLayoutInfo
            let width = bordered_box.0.x;
            let height = bordered_box.0.y;
    
    
            // Get the children of this text entity
            if let Ok(children) = children_query.get(text_entity) {
                // Iterate through children to find the background (assuming only one BorderedBoxBackground per TextBox)
                for &child in children.iter() {
                    if let Ok(entity) = border_query.get_mut(child) {
                        // Update the mesh to match the text size
                        commands.entity(entity).despawn_descendants();
                        commands.entity(entity).insert(
                            SpriteBox{
                                width,
                                height
                            }
                        );
                    }
                }
            }
        }
    }
}

#[derive(Component, Clone)]
pub struct WindowTitle{
    pub text : String,
    pub padding : f32
}

impl Default for WindowTitle{
    fn default() -> Self {
        Self{
            text : String::from(""),
            padding : 20.0
        }
    }
}


#[derive(Clone)]
pub struct WindowBox{
    pub title : Option<WindowTitle>,
    pub size : Vec2,
    pub header_height : f32   
}

impl Default for WindowBox{
    fn default() -> Self {
        Self{
            title : None,
            size : Vec2::default(),
            header_height : 20.0
        }
    }
}

impl Component for WindowBox {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {       

                let windowed_box: Option<WindowBox> = {
                    let entity_mut = world.entity(entity);
                    entity_mut.get::<WindowBox>()
                        .map(|windowed_box: &WindowBox| windowed_box.clone())
                };

                if let Some(window_box)= windowed_box {
                    let width = window_box.size.x;
                    let height = window_box.size.y;

                    // Step 3: Use commands in a separate scope to spawn the entity
                    {
                        let mut commands = world.commands();
                        commands.entity(entity).with_children(|parent| {
                            parent.spawn((
                                WindowBoxBody(Vec2::new(width, height)),
                            ));

                            parent.spawn((
                                WindowBoxHeader{
                                    title : window_box.title,
                                    size : Vec2::new(width, window_box.header_height)
                                },
                                Transform::from_xyz(0.0, height/2.0+ window_box.header_height/2.0, 0.0)
                            ));
                        });
                    }
                }
            }
        );
    }
}

impl WindowBox {
    fn update_body(
        mut box_query: Query<(Entity, &WindowBox), (Changed<WindowBox>, Without<WindowBoxBody>)>,
        mut body_query: Query<&mut WindowBoxBody>,
        children_query: Query<&Children>
    ) {
        for (entity, windowed_box) in box_query.iter_mut() {
            // Get the children of this text entity
            if let Ok(children) = children_query.get(entity) {
                // Iterate through children to find the background (assuming only one BorderedBoxBackground per TextBox)
                for &child in children.iter() {
                    if let Ok(mut body) = body_query.get_mut(child) {
                        body.0 = windowed_box.size;
                    }
                }
            }
        }
    }

    fn update_header(
        mut box_query: Query<(Entity, &WindowBox), (Changed<WindowBox>, Without<WindowBoxHeader>)>,
        mut head_query: Query<(&mut WindowBoxHeader, &mut Transform)>,
        children_query: Query<&Children>
    ) {
        for (entity, windowed_box) in box_query.iter_mut() {
            // Get the children of this text entity
            if let Ok(children) = children_query.get(entity) {
                // Iterate through children to find the background (assuming only one BorderedBoxBackground per TextBox)
                for &child in children.iter() {
                    if let Ok((mut head, mut transform)) = head_query.get_mut(child) {
                        head.size = Vec2::new(windowed_box.size.x, windowed_box.header_height);
                        head.title = windowed_box.title.clone();
                        transform.translation = Vec3::new(0.0, windowed_box.size.y/2.0 + windowed_box.header_height/2.0, 0.0);
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
struct WindowBoxBody(pub Vec2);

impl Component for WindowBoxBody {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {       

                let bordered_box: Option<WindowBoxBody> = {
                    let entity_mut: EntityRef<'_> = world.entity(entity);
                    entity_mut.get::<WindowBoxBody>()
                        .map(|bordered_box: &WindowBoxBody| bordered_box.clone())
                };

                if let Some(window_box)= bordered_box {
                    let width = window_box.0.x;
                    let height = window_box.0.y;

                    // Step 3: Use commands in a separate scope to spawn the entity
                    {
                        let mut commands = world.commands();
                        commands.entity(entity).with_children(|parent| {
                            parent.spawn((
                                BorderedBox(Vec2::new(width, height)),
                            ));
                        });
                    }
                }
            }
        );
    }
}

impl WindowBoxBody {
    fn update_box(
        mut body_query: Query<(Entity, &WindowBoxBody), (Changed<WindowBoxBody>, Without<BorderedBox>)>,
        mut box_query: Query<&mut BorderedBox>,
        children_query: Query<&Children>
    ) {
        for (entity, windowed_box_body) in body_query.iter_mut() {
            // Get the children of this text entity
            if let Ok(children) = children_query.get(entity) {
                // Iterate through children to find the background (assuming only one BorderedBoxBackground per TextBox)
                for &child in children.iter() {
                    if let Ok(mut body) = box_query.get_mut(child) {
                        body.0 = windowed_box_body.0;
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
struct WindowBoxHeader{
    pub title : Option<WindowTitle>,
    pub size :Vec2
}

impl Component for WindowBoxHeader {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {       

                let bordered_box: Option<WindowBoxHeader> = {
                    let entity_mut: EntityRef<'_> = world.entity(entity);
                    entity_mut.get::<WindowBoxHeader>()
                        .map(|bordered_box: &WindowBoxHeader| bordered_box.clone())
                };

                if let Some(window_box)= bordered_box {
                    let width = window_box.size.x;
                    let height = window_box.size.y;

                    // Step 3: Use commands in a separate scope to spawn the entity
                    {
                        let mut commands = world.commands();
                        commands.entity(entity).with_children(|parent| {
                            parent.spawn((
                                BorderedBox(Vec2::new(width, height)),
                            ));
                        });


                        if let Some(title) = window_box.title {


                            commands.entity(entity).with_children(|parent| {

                                let title_text = title.clone().text;
                                let padding = title.padding;
                                parent.spawn((
                                    title.clone(),
                                    Text2d(title_text),
                                    TextColor(PRIMARY_COLOR),
                                    TextFont{
                                        font_size : 12.0,
                                        ..default()
                                    },
                                    Anchor::CenterLeft,
                                    Transform::from_xyz((-width + padding) / 2.0, 0.0, 0.0)
                                ));
                            });
                        }
                    }
                }
            }
        );
    }
}


impl WindowBoxHeader {
    fn update_box(
        mut commands : Commands,
        mut body_query: Query<(Entity, &WindowBoxHeader), (Changed<WindowBoxHeader>, Without<BorderedBox>)>,
        mut box_query: Query<&mut BorderedBox>,
        mut title_query: Query<(Entity, &WindowTitle)>,
        children_query: Query<&Children>
    ) {
        for (entity, windowed_box_header) in body_query.iter_mut() {
            // Get the children of this text entity
            if let Ok(children) = children_query.get(entity) {
                // Iterate through children to find the background (assuming only one BorderedBoxBackground per TextBox)
                for &child in children.iter() {
                    if let Ok(mut body) = box_query.get_mut(child) {
                        body.0 = windowed_box_header.size;
                    }

                    if let Ok((entity, title)) = title_query.get_mut(child) {
                        commands.entity(entity).insert(
                            Transform::from_xyz((-windowed_box_header.size.x + title.padding)/2.0 , 0.0, 0.0)
                        );
                    }
                }
            }
        }
    }
}
