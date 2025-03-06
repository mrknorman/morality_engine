use std::f32::consts::FRAC_PI_4;
use bevy::{ecs::component::{ComponentHooks, StorageType}, prelude::*, sprite::Anchor};
use enum_map::{enum_map, Enum};
use crate::{audio::{TransientAudio, TransientAudioPallet}, colors::{ColorAnchor, ColorChangeEvent, ColorChangeOn, CLICKED_BUTTON, HOVERED_BUTTON, PRIMARY_COLOR}, interaction::{ActionPallet, Clickable, InputAction}};


use crate::sprites::compound::*;

pub struct WindowPlugin;
impl Plugin for WindowPlugin {
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
    pub has_close_button : bool,
    pub root_entity : Option<Entity>
}

impl Default for Window{
    fn default() -> Self {
        Self{
            title : None,
            boundary : HollowRectangle::default(),
            header_height : 20.0,
            has_close_button : true,
            root_entity : None
        }
    }
}

impl Component for Window {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {      


                if let Some(mut window) = world.entity_mut(entity).get_mut::<Window>() {
                    if window.root_entity.is_none() {
                        window.root_entity = Some(entity);
                    }
                };

                let (root_entity, title, boundary, header_height, has_close_button) = {
                    if let Some(window) = world.entity_mut(entity).get_mut::<Window>() {
                        (window.root_entity, window.title.clone(), window.boundary, window.header_height, window.has_close_button)
                    } else {
                        return;
                    }
                };

                world.commands().entity(entity).with_children(|parent| {
                    parent.spawn(WindowBody{boundary});

                    parent.spawn((
                        WindowHeader{
                            title,
                            boundary,
                            has_close_button,
                            root_entity : Some(root_entity.unwrap_or(entity))
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
        boundary : HollowRectangle,
        header_height : f32,
        has_close_button : bool,
        root_entity : Option<Entity>
    ) -> Self {
        Self{
            title,
            boundary,
            header_height,
            has_close_button,
            root_entity
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
                    if let Ok(mut body) = body_query.get_mut(child) {
                        body.boundary = window.boundary;
                    }
                    
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
        };
    }
}

#[derive(Clone)]
struct WindowBody{
    pub boundary : HollowRectangle
}

impl Component for WindowBody {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {     

                let boundary = {
                    if let Some(window) = world.entity(entity).get::<WindowBody>() {
                        window.boundary
                    } else {
                        return;
                    }
                };  

                world.commands().entity(entity).with_children(|parent| {
                    parent.spawn(
                        BorderedRectangle{boundary}
                    );
                });
            }
        );
    }
}

impl WindowBody {
    fn update(
        mut body_query: Query<(Entity, &WindowBody), (Changed<WindowBody>, Without<BorderedRectangle>)>,
        mut rectangle_query: Query<&mut BorderedRectangle>,
        children_query: Query<&Children>
    ) {
        for (entity, body) in body_query.iter_mut() {
            if let Ok(children) = children_query.get(entity) {
                for &child in children.iter() {
                    if let Ok(mut border) = rectangle_query.get_mut(child) {
                        border.boundary = body.boundary;
                    }
                }
            }
        };
    }
}

#[derive(Clone)]
struct WindowHeader{
    pub title : Option<WindowTitle>,
    pub boundary : HollowRectangle,
    pub has_close_button : bool,
    pub root_entity : Option<Entity>
}

impl Default for WindowHeader{
    fn default() -> Self {
        Self{
            title : None,
            boundary : HollowRectangle::default(),
            has_close_button : false,
            root_entity : None
        }
    }
}

impl Component for WindowHeader {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {

                let (root_entity, boundary, height, width, has_close_button, title, color) = {
                    if let Some(window) = world.entity(entity).get::<WindowHeader>() {
                        (window.root_entity, window.boundary, window.boundary.dimensions.y, window.boundary.dimensions.x, window.has_close_button, window.title.clone(), window.boundary.color)
                    } else {
                        return;
                    }
                };         

                world.commands().entity(entity).with_children(|parent| {
                    parent.spawn(BorderedRectangle{boundary});

                    if has_close_button {
                        parent.spawn( (WindowCloseButton{
                                boundary : HollowRectangle{
                                    dimensions : Vec2::splat(height),
                                    thickness : boundary.thickness,
                                    color,
                                    ..default()
                                },
                                root_entity
                            },
                            Transform::from_xyz((width - height) / 2.0, 0.0, 0.0) 
                        ));
                    }   
                    
                    if let Some(title) = title {
                        parent.spawn((
                            title.clone(),
                            Text2d(title.text),
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
        });
    }
}

impl WindowHeader {
    fn update(
        mut header_query: Query<(Entity, &WindowHeader), Changed<WindowHeader>>,
        mut rectangle_query: Query<&mut BorderedRectangle>,
        children_query: Query<&Children>,
        mut transform_sets: ParamSet<(
            Query<(&mut Transform, &WindowTitle)>,
            Query<(&mut Transform, &mut WindowCloseButton)>,
        )>,
    ) {
        for (entity, header) in header_query.iter_mut() {
            if let Ok(children) = children_query.get(entity) {
                for &child in children.iter() {

                    if let Ok(mut body) = rectangle_query.get_mut(child) {
                        body.boundary = header.boundary;
                    }
    
                    if let Ok((mut transform, title)) = transform_sets.p0().get_mut(child) {
                        transform.translation = Vec3::new(
                            (-header.boundary.dimensions.x + title.padding) / 2.0,
                            0.0,
                            0.0,
                        );
                    }
    
                    if let Ok((mut transform, mut button)) = transform_sets.p1().get_mut(child) {
                        button.boundary.dimensions = Vec2::splat(header.boundary.dimensions.y);
                        transform.translation = Vec3::new(
                            (header.boundary.dimensions.x - header.boundary.dimensions.y) / 2.0,
                            0.0,
                            0.0,
                        );
                    }
                }
            }
        };
    }
}


#[derive(Clone)]
struct WindowCloseButton {
    boundary : HollowRectangle,
    root_entity : Option<Entity>
}

#[derive(Component)]
struct WindowCloseButtonBorder;

#[derive(Component)]
struct WindowCloseButtonIcon;

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowSounds{
    Close
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowActions{
    CloseWindow
}


impl Component for WindowCloseButton {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {       

                let (root_entity, boundary, dimensions, color) = {
                    if let Some(button) = world.entity(entity).get::<WindowCloseButton>() {
                        (button.root_entity, button.boundary, button.boundary.dimensions, button.boundary.color)
                    } else {
                        return;
                    }
                };

                let handle = world.get_resource::<AssetServer>().unwrap().load("sounds/mouse_click.ogg");

                world.commands().entity(entity).with_children(|parent| {

                    parent.spawn((
                        WindowCloseButtonBorder,
                        BorderedRectangle{boundary}
                    ));

                    let interaction_region: Vec2 = dimensions + 10.0;
                    let dimensions = dimensions - 10.0;

                    parent.spawn((
                        WindowCloseButtonIcon,
                        Plus{
                            dimensions,
                            thickness : 1.0,
                            color
                        }, 
                        Clickable::with_region(vec![
                                WindowActions::CloseWindow
                            ],
                            interaction_region
                        ),
                        ColorChangeOn::new(vec![
                            ColorChangeEvent::Click(vec![CLICKED_BUTTON], Some(interaction_region)),
                            ColorChangeEvent::Hover(vec![HOVERED_BUTTON], Some(interaction_region)),
                        ]),
                        ColorAnchor(color),
                        Transform{
                            rotation :  Quat::from_rotation_z(FRAC_PI_4),
                            ..default()
                        },
                        ActionPallet::<WindowActions, WindowSounds>(
							enum_map!(
								WindowActions::CloseWindow => vec![
									    InputAction::PlaySound(WindowSounds::Close),
									    InputAction::Despawn(root_entity)
									]
								)
						),
						TransientAudioPallet::new(
							vec![(
								WindowSounds::Close,
								vec![
									TransientAudio::new(
										handle, 
										0.1, 
										true,
										1.0,
										true
									)
								]
							)]
						)
                    ));                
                });
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
            Query<(&mut Plus, &mut ColorChangeOn, &mut Clickable<WindowActions>)>
        )>,
    ) {
        for (entity, button) in header_query.iter_mut() {
            if let Ok(children) = children_query.get(entity) {
                for &child in children.iter() {

                    if let Ok(mut bordered_rectangle) = transform_sets.p0().get_mut(child) {
                        bordered_rectangle.boundary = button.boundary;
                    }
                    
                    if let Ok((mut plus, mut color_change, mut clickable)) = transform_sets.p1().get_mut(child) {
                        plus.dimensions = button.boundary.dimensions - 10.0;

                        let new_region = Some(button.boundary.dimensions + 10.0);
                        clickable.region = new_region;
                        
                        for event in color_change.events.iter_mut() {
                            match event {
                                ColorChangeEvent::Hover(_, ref mut region) => {
                                    *region = new_region;
                                },
                                ColorChangeEvent::Click(_, ref mut region) => {
                                    *region = new_region;
                                },
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}


