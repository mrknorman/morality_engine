use std::f32::consts::FRAC_PI_4;
use bevy::{ecs::component::{ComponentHooks, StorageType}, prelude::*, sprite::Anchor};
use crate::colors::PRIMARY_COLOR;

pub mod compounds;
use compounds::*;

pub struct SpritePlugin;
impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(
			Update,
			(
                Window::update,
                WindowBody::update,
                WindowHeader::update,
                WindowCloseButton::update
        ))
        .register_required_components::<Window, Transform>()
        .register_required_components::<Window, Visibility>()
        .register_required_components::<WindowHeader, Transform>()
        .register_required_components::<WindowHeader, Visibility>()
        .register_required_components::<WindowBody, Transform>()
        .register_required_components::<WindowBody, Visibility>()
        .register_required_components::<WindowCloseButton, Transform>()
        .register_required_components::<WindowCloseButton, Visibility>()
        .add_plugins(CompoundPlugin)
        ;
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
pub struct Window{
    pub boundary : HollowRectangle,
    pub title : Option<WindowTitle>,
    pub header_height : f32,
    pub has_close_button : bool
}

impl Default for Window{
    fn default() -> Self {
        Self{
            title : None,
            boundary : HollowRectangle::default(),
            header_height : 20.0,
            has_close_button : true
        }
    }
}

impl Component for Window {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {      

                let (title, boundary, header_height, has_close_button) = {
                    if let Some(window) = world.entity(entity).get::<Window>() {
                        (window.title.clone(), window.boundary, window.header_height, window.has_close_button)
                    } else {
                        return;
                    }
                };
                
                world.commands().entity(entity).with_children(|parent| {
                    parent.spawn(WindowBody(boundary));

                    parent.spawn((
                        WindowHeader{
                            title,
                            boundary,
                            has_close_button
                        },
                        Transform::from_xyz(0.0, (boundary.dimensions.y + header_height)/2.0, 0.0)
                    ));
                });
            }
        );
    }
}

impl Window {
    pub fn new(
        title : Option<WindowTitle>,
        width : f32,
        height : f32,
        thickness : f32,
        header_height : f32,
        has_close_button : bool
    ) -> Self {
        Self{
            title,
            boundary : HollowRectangle { 
                dimensions: Vec2::new(width, height),
                thickness
            },
            header_height,
            has_close_button
        }
    }

    fn update(
        mut window_query: Query<(Entity, &Window), Changed<Window>>,
        mut body_query: Query<&mut WindowBody, Without<WindowHeader>>,
        mut head_query: Query<(&mut WindowHeader, &mut Transform), Without<WindowBody>>,
        children_query: Query<&Children>,
    ) {
        for (entity, window) in window_query.iter_mut() {
            if let Ok(children) = children_query.get(entity) {
                for &child in children.iter() {
                    // Update the body component if it exists
                    if let Ok(mut body) = body_query.get_mut(child) {
                        body.0 = window.boundary;
                    }
                    
                    // Update the header component if it exists
                    if let Ok((mut header, mut transform)) = head_query.get_mut(child) {
                        header.boundary.dimensions = Vec2::new(window.boundary.dimensions.x, window.header_height);
                        header.boundary.thickness = window.boundary.thickness;
                        header.title = window.title.clone();
                        transform.translation = Vec3::new(
                            0.0, 
                            (window.boundary.dimensions.y + window.header_height) / 2.0, 
                            0.0
                        );
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
struct WindowBody(pub HollowRectangle);

impl Component for WindowBody {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {       

                let bordered_box: Option<WindowBody> = {
                    let entity_mut: EntityRef<'_> = world.entity(entity);
                    entity_mut.get::<WindowBody>()
                        .map(|bordered_box: &WindowBody| bordered_box.clone())
                };

                if let Some(window_body)= bordered_box {
                    world.commands().entity(entity).with_children(|parent| {
                        parent.spawn(
                            BorderedRectangle(window_body.0)
                        );
                    });
                }
            }
        );
    }
}

impl WindowBody {
    fn update(
        mut body_query: Query<(Entity, &WindowBody), (Changed<WindowBody>, Without<BorderedRectangle>)>,
        mut box_query: Query<&mut BorderedRectangle>,
        children_query: Query<&Children>
    ) {
        for (entity, body) in body_query.iter_mut() {
            if let Ok(children) = children_query.get(entity) {
                for &child in children.iter() {
                    if let Ok(mut border) = box_query.get_mut(child) {
                        border.0 = body.0;
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
struct WindowHeader{
    pub title : Option<WindowTitle>,
    pub boundary : HollowRectangle,
    pub has_close_button : bool
}

impl Default for WindowHeader{
    fn default() -> Self {
        Self{
            title : None,
            boundary : HollowRectangle::default(),
            has_close_button : false
        }
    }
}

impl Component for WindowHeader {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {       

                let header: Option<WindowHeader> = {
                    let entity_mut: EntityRef<'_> = world.entity(entity);
                    entity_mut.get::<WindowHeader>()
                        .map(|header: &WindowHeader| header.clone())
                };

                if let Some(header)= header {
                    let width = header.boundary.dimensions.x;
                    let height = header.boundary.dimensions.y;

                    let mut commands = world.commands();
                    commands.entity(entity).with_children(|parent| {
                        parent.spawn(
                            BorderedRectangle::new(
                                height,
                                width,
                                2.0
                            )
                        );

                        if header.has_close_button {
                            parent.spawn( (WindowCloseButton{
                                    boundary : HollowRectangle{
                                        dimensions : Vec2::new(height, height),
                                        thickness : 2.0
                                    }
                                },
                                Transform::from_xyz(width / 2.0 - height / 2.0, 0.0, 0.0) 
                            ));
                        }   
                        
                        if let Some(title) = header.title {
                            parent.spawn((
                                title.clone(),
                                Text2d(title.clone().text),
                                TextColor(PRIMARY_COLOR),
                                TextFont{
                                    font_size : 12.0,
                                    ..default()
                                },
                                Anchor::CenterLeft,
                                Transform::from_xyz((-width + title.padding) / 2.0, 0.0, 0.0)
                            ));
                        }
                    });
                }
            }
        );
    }
}

impl WindowHeader {
    fn update(
        mut header_query: Query<(Entity, &WindowHeader), Changed<WindowHeader>>,
        mut main_box_query: Query<&mut BorderedRectangle, Without<WindowCloseButtonBorder>>,
        children_query: Query<&Children>,
        mut transform_sets: ParamSet<(
            Query<(&mut Transform, &WindowTitle)>,
            Query<(&mut Transform, &mut WindowCloseButton)>,
        )>,
    ) {
        for (entity, header) in header_query.iter_mut() {
            if let Ok(children) = children_query.get(entity) {
                for &child in children.iter() {
                    // Update the main body box (non-close-button)
                    if let Ok(mut body) = main_box_query.get_mut(child) {
                        body.0.dimensions = header.boundary.dimensions;
                    }
    
                    // Update the title transform
                    if let Ok((mut transform, title)) = transform_sets.p0().get_mut(child) {
                        transform.translation = Vec3::new(
                            (-header.boundary.dimensions.x + title.padding) / 2.0,
                            0.0,
                            0.0,
                        );
                    }
    
                    // Update the close button
                    if let Ok((mut transform, mut button)) = transform_sets.p1().get_mut(child) {
                        button.boundary.dimensions = Vec2::new(header.boundary.dimensions.y, header.boundary.dimensions.y);
                        transform.translation = Vec3::new(
                            (header.boundary.dimensions.x - header.boundary.dimensions.y) / 2.0,
                            0.0,
                            0.0,
                        );
                    }
                }
            }
        }
    }
}


#[derive(Clone)]
struct WindowCloseButton {
    boundary : HollowRectangle
}

#[derive(Component)]
struct WindowCloseButtonBorder;

#[derive(Component)]
struct WindowCloseButtonIcon;

impl Component for WindowCloseButton {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {       

                let header: Option<WindowCloseButton> = {
                    let entity_mut: EntityRef<'_> = world.entity(entity);
                    entity_mut.get::<WindowCloseButton>()
                        .map(|header: &WindowCloseButton| header.clone())
                };

                if let Some(header)= header {
                    let width = header.boundary.dimensions.x;
                    let height = header.boundary.dimensions.y;

                    let mut commands = world.commands();
                    commands.entity(entity).with_children(|parent| {

                        parent.spawn((
                            WindowCloseButtonBorder,
                            BorderedRectangle::new(
                                width,
                                height,
                                2.0
                            )
                        ));

                        parent.spawn((
                            WindowCloseButtonIcon,
                            Plus{
                                dimensions : Vec2::new(width - 10.0, height - 10.0),
                                thickness : 1.0
                            }, 
                            Transform{
                                rotation :  Quat::from_rotation_z(FRAC_PI_4),
                                ..default()
                            }
                        ));                
                    });
                }
            }
        );
    }
}

impl WindowCloseButton {
    fn update(
        mut header_query: Query<(Entity, &WindowCloseButton), Changed<WindowCloseButton>>,
        children_query: Query<&Children>,
        mut transform_sets: ParamSet<(
            Query<&mut BorderedRectangle>,
            Query<&mut Plus>
        )>,
    ) {
        for (entity, button) in header_query.iter_mut() {
            if let Ok(children) = children_query.get(entity) {
                for &child in children.iter() {
                    if let Ok(mut bordered_rectangle) = transform_sets.p0().get_mut(child) {
                        bordered_rectangle.0.dimensions = Vec2::new(button.boundary.dimensions.y, button.boundary.dimensions.y);
                    }
    
                    if let Ok(mut plus) = transform_sets.p1().get_mut(child) {
                        plus.dimensions = Vec2::new(button.boundary.dimensions.y - 10.0, button.boundary.dimensions.y - 10.0);
                    }
                }
            }
        }
    }
}


