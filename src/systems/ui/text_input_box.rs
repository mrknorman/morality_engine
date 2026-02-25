//! Reusable owner-scoped text input primitive.
//!
//! `TextInputBox` provides:
//! - pointer focus acquisition via shared `Clickable`/`Hoverable`
//! - owner/layer gated keyboard input
//! - caret state + blinking
//! - changed/submitted/cancelled messages
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
    sprite::Anchor,
    text::TextBounds,
    time::Real,
};

use crate::{
    entities::sprites::compound::HollowRectangle,
    startup::cursor::CustomCursor,
    systems::{
        interaction::{
            clickable_system, hoverable_system, scoped_owner_has_focus,
            ui_input_policy_allows_mode, Clickable, Hoverable, InteractionSystem, UiInputPolicy,
            UiInteractionState,
        },
        ui::layer::{self, UiLayer, UiLayerKind},
    },
};

pub struct TextInputBoxPlugin;

impl Plugin for TextInputBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TextInputBoxChanged>()
            .add_message::<TextInputBoxSubmitted>()
            .add_message::<TextInputBoxCancelled>()
            .add_systems(
                Update,
                (
                    hoverable_system::<TextInputBoxActions>
                        .in_set(InteractionSystem::Hoverable)
                        .run_if(resource_exists::<CustomCursor>)
                        .run_if(resource_exists::<UiInteractionState>),
                    clickable_system::<TextInputBoxActions>
                        .in_set(InteractionSystem::Clickable)
                        .after(InteractionSystem::Hoverable)
                        .run_if(resource_exists::<CustomCursor>)
                        .run_if(resource_exists::<ButtonInput<MouseButton>>)
                        .run_if(resource_exists::<UiInteractionState>),
                    apply_text_input_click_focus
                        .after(InteractionSystem::Clickable)
                        .run_if(resource_exists::<ButtonInput<MouseButton>>)
                        .run_if(resource_exists::<UiInteractionState>),
                    handle_text_input_keyboard
                        .after(apply_text_input_click_focus)
                        .run_if(resource_exists::<ButtonInput<KeyCode>>)
                        .run_if(resource_exists::<UiInteractionState>),
                    tick_text_input_caret
                        .after(handle_text_input_keyboard)
                        .run_if(resource_exists::<Time<Real>>),
                    sync_text_input_visuals.after(tick_text_input_caret),
                ),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextInputBoxActions {
    Focus,
}

#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct TextInputBoxChanged {
    pub entity: Entity,
    pub value: String,
}

#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct TextInputBoxSubmitted {
    pub entity: Entity,
    pub value: String,
}

#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct TextInputBoxCancelled {
    pub entity: Entity,
}

#[derive(Component, Clone, Copy, Debug)]
#[require(
    TextInputBoxStyle,
    TextInputBoxValue,
    TextInputBoxPlaceholder,
    TextInputBoxLimits,
    TextInputBoxFocus,
    TextInputBoxCaretState,
    Transform,
    Visibility,
    Sprite
)]
#[component(on_insert = TextInputBox::on_insert)]
pub struct TextInputBox {
    pub owner: Entity,
    pub input_layer: UiLayerKind,
    insert_ui_layer: bool,
}

impl TextInputBox {
    pub const fn new(owner: Entity, input_layer: UiLayerKind) -> Self {
        Self {
            owner,
            input_layer,
            insert_ui_layer: true,
        }
    }

    pub const fn without_ui_layer(mut self) -> Self {
        self.insert_ui_layer = false;
        self
    }

    #[cfg(test)]
    pub const fn inserts_ui_layer(&self) -> bool {
        self.insert_ui_layer
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let Some(root) = world.entity(entity).get::<TextInputBox>().copied() else {
            return;
        };
        let style = world
            .entity(entity)
            .get::<TextInputBoxStyle>()
            .copied()
            .unwrap_or_default();

        if root.insert_ui_layer && world.entity(entity).get::<UiLayer>().is_none() {
            world
                .commands()
                .entity(entity)
                .insert(UiLayer::new(root.owner, root.input_layer));
        }

        if world
            .entity(entity)
            .get::<Clickable<TextInputBoxActions>>()
            .is_none()
        {
            world
                .commands()
                .entity(entity)
                .insert(Clickable::with_region(
                    vec![TextInputBoxActions::Focus],
                    style.size,
                ));
        }

        let mut root_transform = world
            .entity(entity)
            .get::<Transform>()
            .copied()
            .unwrap_or_default();
        root_transform.translation.z = style.z;
        world.commands().entity(entity).insert((
            root_transform,
            Sprite::from_color(style.background_color, style.size),
        ));

        if world.entity(entity).contains::<TextInputBoxParts>() {
            return;
        }

        let value = world
            .entity(entity)
            .get::<TextInputBoxValue>()
            .map(|value| value.0.clone())
            .unwrap_or_default();
        let placeholder = world
            .entity(entity)
            .get::<TextInputBoxPlaceholder>()
            .map(|placeholder| placeholder.0.clone())
            .unwrap_or_default();
        let focused = world
            .entity(entity)
            .get::<TextInputBoxFocus>()
            .is_some_and(|focus| focus.focused);
        let show_placeholder = value.is_empty() && !focused;
        let label_text = if show_placeholder { placeholder } else { value };
        let label_color = if show_placeholder {
            style.placeholder_color
        } else {
            style.text_color
        };

        let mut border_entity: Option<Entity> = None;
        let mut label_entity: Option<Entity> = None;
        let mut caret_entity: Option<Entity> = None;

        world.commands().entity(entity).with_children(|parent| {
            border_entity = Some(
                parent
                    .spawn((
                        Name::new("text_input_box_border"),
                        TextInputBoxBorder,
                        HollowRectangle {
                            dimensions: (style.size - Vec2::splat(2.0)).max(Vec2::splat(1.0)),
                            thickness: style.border_thickness,
                            color: style.border_color,
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 0.01),
                    ))
                    .id(),
            );
            label_entity = Some(
                parent
                    .spawn((
                        Name::new("text_input_box_label"),
                        TextInputBoxLabel,
                        Text2d::new(label_text),
                        TextFont {
                            font_size: style.font_size,
                            ..default()
                        },
                        TextColor(label_color),
                        TextBounds {
                            width: Some((style.size.x - style.padding.x * 2.0).max(1.0)),
                            height: Some((style.size.y - style.padding.y * 2.0).max(1.0)),
                        },
                        TextLayout {
                            justify: Justify::Left,
                            ..default()
                        },
                        Anchor::CENTER_LEFT,
                        Transform::from_xyz(-style.size.x * 0.5 + style.padding.x, 0.0, 0.02),
                    ))
                    .id(),
            );
            caret_entity = Some(
                parent
                    .spawn((
                        Name::new("text_input_box_caret"),
                        TextInputBoxCaret,
                        Sprite::from_color(
                            style.caret_color,
                            Vec2::new(style.caret_width.max(1.0), style.font_size.max(1.0)),
                        ),
                        if focused {
                            Visibility::Inherited
                        } else {
                            Visibility::Hidden
                        },
                        Transform::from_xyz(-style.size.x * 0.5 + style.padding.x, 0.0, 0.03),
                    ))
                    .id(),
            );
        });

        let (Some(border), Some(label), Some(caret)) = (border_entity, label_entity, caret_entity)
        else {
            return;
        };

        world.commands().entity(entity).insert(TextInputBoxParts {
            border,
            label,
            caret,
        });
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct TextInputBoxValue(pub String);

impl TextInputBoxValue {
    pub fn set(&mut self, value: impl Into<String>) {
        self.0 = value.into();
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct TextInputBoxPlaceholder(pub String);

impl TextInputBoxPlaceholder {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct TextInputBoxLimits {
    pub max_chars: usize,
}

impl Default for TextInputBoxLimits {
    fn default() -> Self {
        Self { max_chars: 96 }
    }
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct TextInputBoxFocus {
    pub focused: bool,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct TextInputBoxStyle {
    pub size: Vec2,
    pub padding: Vec2,
    pub font_size: f32,
    pub text_advance_factor: f32,
    pub caret_width: f32,
    pub border_thickness: f32,
    pub background_color: Color,
    pub border_color: Color,
    pub border_color_hovered: Color,
    pub border_color_focused: Color,
    pub text_color: Color,
    pub placeholder_color: Color,
    pub caret_color: Color,
    pub z: f32,
}

impl Default for TextInputBoxStyle {
    fn default() -> Self {
        Self {
            size: Vec2::new(360.0, 28.0),
            padding: Vec2::new(10.0, 6.0),
            font_size: 13.0,
            text_advance_factor: 0.57,
            caret_width: 1.5,
            border_thickness: 2.0,
            background_color: Color::BLACK,
            border_color: Color::srgba(1.0, 1.0, 1.0, 0.45),
            border_color_hovered: Color::srgba(1.0, 1.0, 1.0, 0.75),
            border_color_focused: Color::WHITE,
            text_color: Color::WHITE,
            placeholder_color: Color::srgba(1.0, 1.0, 1.0, 0.55),
            caret_color: Color::WHITE,
            z: 0.25,
        }
    }
}

#[derive(Component, Debug)]
pub struct TextInputBoxCaretState {
    pub index: usize,
    pub visible: bool,
    pub blink_timer: Timer,
}

impl Default for TextInputBoxCaretState {
    fn default() -> Self {
        Self {
            index: 0,
            visible: true,
            blink_timer: Timer::from_seconds(0.45, TimerMode::Repeating),
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct TextInputBoxParts {
    border: Entity,
    label: Entity,
    caret: Entity,
}

#[derive(Component, Clone, Copy, Debug)]
struct TextInputBoxBorder;

#[derive(Component, Clone, Copy, Debug)]
struct TextInputBoxLabel;

#[derive(Component, Clone, Copy, Debug)]
struct TextInputBoxCaret;

fn mark_caret_interaction(caret: &mut TextInputBoxCaretState) {
    caret.visible = true;
    caret.blink_timer.reset();
}

fn char_count(text: &str) -> usize {
    text.chars().count()
}

fn byte_index_for_char(text: &str, index: usize) -> usize {
    text.char_indices()
        .nth(index)
        .map(|(byte, _)| byte)
        .unwrap_or(text.len())
}

fn remove_char_at(text: &mut String, index: usize) -> bool {
    if index >= char_count(text) {
        return false;
    }
    let start = byte_index_for_char(text, index);
    let end = byte_index_for_char(text, index + 1);
    text.replace_range(start..end, "");
    true
}

fn insert_char_at(text: &mut String, index: usize, value: char) {
    let byte = byte_index_for_char(text, index);
    text.insert(byte, value);
}

fn keycode_to_ascii_char(keycode: KeyCode, shift: bool) -> Option<char> {
    let letter = match keycode {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        _ => None,
    };

    if let Some(letter) = letter {
        return Some(if shift {
            letter.to_ascii_uppercase()
        } else {
            letter
        });
    }

    match keycode {
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some(if shift { '_' } else { '-' }),
        KeyCode::Equal => Some(if shift { '+' } else { '=' }),
        _ => None,
    }
}

fn is_input_interactable(
    text_input: &TextInputBox,
    policy: Option<&UiInputPolicy>,
    inherited_visibility: Option<&InheritedVisibility>,
    interaction_state: &UiInteractionState,
) -> bool {
    if inherited_visibility.is_some_and(|visibility| !visibility.get()) {
        return false;
    }
    if !ui_input_policy_allows_mode(
        policy,
        interaction_state.input_mode_for_owner(text_input.owner),
    ) {
        return false;
    }
    if layer::active_layer_kind_for_owner(
        &interaction_state.active_layers_by_owner,
        text_input.owner,
    ) != text_input.input_layer
    {
        return false;
    }
    scoped_owner_has_focus(Some(text_input.owner), interaction_state.focused_owner)
}

fn apply_text_input_click_focus(
    mouse_input: Res<ButtonInput<MouseButton>>,
    interaction_state: Res<UiInteractionState>,
    mut query: Query<
        (
            Entity,
            &TextInputBox,
            Option<&UiInputPolicy>,
            Option<&InheritedVisibility>,
            &mut Clickable<TextInputBoxActions>,
            &mut TextInputBoxFocus,
            &mut TextInputBoxCaretState,
        ),
        Without<TextSpan>,
    >,
) {
    let mut clicked_by_owner: std::collections::HashMap<Entity, Entity> =
        std::collections::HashMap::new();

    for (entity, text_input, policy, inherited_visibility, mut clickable, _, _) in query.iter_mut()
    {
        let clicked = clickable.triggered;
        clickable.triggered = false;
        if !clicked {
            continue;
        }
        if !is_input_interactable(text_input, policy, inherited_visibility, &interaction_state) {
            continue;
        }

        let replace = clicked_by_owner
            .get(&text_input.owner)
            .is_none_or(|current| entity.to_bits() > current.to_bits());
        if replace {
            clicked_by_owner.insert(text_input.owner, entity);
        }
    }

    let mut clear_focus_owners: std::collections::HashSet<Entity> =
        std::collections::HashSet::new();
    if mouse_input.just_pressed(MouseButton::Left) {
        for (_, text_input, policy, inherited_visibility, _, focus, _) in query.iter_mut() {
            if !focus.focused {
                continue;
            }
            if clicked_by_owner.contains_key(&text_input.owner) {
                continue;
            }
            if !is_input_interactable(text_input, policy, inherited_visibility, &interaction_state)
            {
                continue;
            }
            clear_focus_owners.insert(text_input.owner);
        }
    }

    if clicked_by_owner.is_empty() && clear_focus_owners.is_empty() {
        return;
    }

    for (entity, text_input, _, _, _, mut focus, mut caret) in query.iter_mut() {
        let mut target_focus = focus.focused;

        if let Some(target) = clicked_by_owner.get(&text_input.owner).copied() {
            target_focus = entity == target;
        } else if clear_focus_owners.contains(&text_input.owner) {
            target_focus = false;
        }

        if target_focus != focus.focused {
            focus.focused = target_focus;
            mark_caret_interaction(&mut caret);
            if !target_focus {
                caret.visible = false;
            }
        }
    }
}

fn handle_text_input_keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    interaction_state: Res<UiInteractionState>,
    mut changed_writer: MessageWriter<TextInputBoxChanged>,
    mut submit_writer: MessageWriter<TextInputBoxSubmitted>,
    mut cancel_writer: MessageWriter<TextInputBoxCancelled>,
    mut query: Query<
        (
            Entity,
            &TextInputBox,
            Option<&UiInputPolicy>,
            Option<&InheritedVisibility>,
            &TextInputBoxLimits,
            &mut TextInputBoxValue,
            &mut TextInputBoxFocus,
            &mut TextInputBoxCaretState,
        ),
        Without<TextSpan>,
    >,
) {
    let shift_pressed =
        keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);
    let move_left = keyboard_input.just_pressed(KeyCode::ArrowLeft);
    let move_right = keyboard_input.just_pressed(KeyCode::ArrowRight);
    let move_home = keyboard_input.just_pressed(KeyCode::Home);
    let move_end = keyboard_input.just_pressed(KeyCode::End);
    let backspace = keyboard_input.just_pressed(KeyCode::Backspace);
    let delete = keyboard_input.just_pressed(KeyCode::Delete);
    let submit = keyboard_input.just_pressed(KeyCode::Enter);
    let cancel =
        keyboard_input.just_pressed(KeyCode::Escape) || keyboard_input.just_pressed(KeyCode::Tab);

    let mut inserted = Vec::new();
    for keycode in keyboard_input.get_just_pressed() {
        if let Some(character) = keycode_to_ascii_char(*keycode, shift_pressed) {
            inserted.push(character);
        }
    }

    if !(move_left
        || move_right
        || move_home
        || move_end
        || backspace
        || delete
        || submit
        || cancel
        || !inserted.is_empty())
    {
        return;
    }

    for (
        entity,
        text_input,
        policy,
        inherited_visibility,
        limits,
        mut value,
        mut focus,
        mut caret,
    ) in query.iter_mut()
    {
        if !focus.focused {
            continue;
        }
        if !is_input_interactable(text_input, policy, inherited_visibility, &interaction_state) {
            continue;
        }

        let mut text_changed = false;
        let mut caret_changed = false;
        let char_len = char_count(&value.0);
        if caret.index > char_len {
            caret.index = char_len;
            caret_changed = true;
        }

        if move_home && caret.index != 0 {
            caret.index = 0;
            caret_changed = true;
        }
        if move_end {
            let end_index = char_count(&value.0);
            if caret.index != end_index {
                caret.index = end_index;
                caret_changed = true;
            }
        }
        if move_left && caret.index > 0 {
            caret.index -= 1;
            caret_changed = true;
        }
        if move_right {
            let end_index = char_count(&value.0);
            if caret.index < end_index {
                caret.index += 1;
                caret_changed = true;
            }
        }

        if backspace && caret.index > 0 && remove_char_at(&mut value.0, caret.index - 1) {
            caret.index -= 1;
            text_changed = true;
        }
        if delete && remove_char_at(&mut value.0, caret.index) {
            text_changed = true;
        }

        for character in inserted.iter().copied() {
            if char_count(&value.0) >= limits.max_chars {
                break;
            }
            insert_char_at(&mut value.0, caret.index, character);
            caret.index += 1;
            text_changed = true;
        }

        if submit {
            submit_writer.write(TextInputBoxSubmitted {
                entity,
                value: value.0.clone(),
            });
        }
        if cancel {
            focus.focused = false;
            cancel_writer.write(TextInputBoxCancelled { entity });
        }

        if text_changed || caret_changed || submit || cancel {
            mark_caret_interaction(&mut caret);
            if cancel {
                caret.visible = false;
            }
        }
        if text_changed {
            changed_writer.write(TextInputBoxChanged {
                entity,
                value: value.0.clone(),
            });
        }
    }
}

fn tick_text_input_caret(
    time: Res<Time<Real>>,
    mut query: Query<(&TextInputBoxFocus, &mut TextInputBoxCaretState), Without<TextSpan>>,
) {
    for (focus, mut caret) in query.iter_mut() {
        if !focus.focused {
            caret.visible = false;
            caret.blink_timer.reset();
            continue;
        }
        caret.blink_timer.tick(time.delta());
        if caret.blink_timer.just_finished() {
            caret.visible = !caret.visible;
        }
    }
}

fn sync_text_input_visuals(
    mut root_query: Query<
        (
            &TextInputBoxStyle,
            &TextInputBoxValue,
            &TextInputBoxPlaceholder,
            &TextInputBoxFocus,
            &TextInputBoxCaretState,
            &Hoverable,
            &TextInputBoxParts,
        ),
        Without<TextSpan>,
    >,
    mut border_query: Query<&mut HollowRectangle, With<TextInputBoxBorder>>,
    mut label_query: Query<(&mut Text2d, &mut TextColor), With<TextInputBoxLabel>>,
    mut caret_query: Query<(&mut Transform, &mut Visibility, &mut Sprite), With<TextInputBoxCaret>>,
) {
    for (style, value, placeholder, focus, caret_state, hoverable, parts) in root_query.iter_mut() {
        if let Ok(mut border) = border_query.get_mut(parts.border) {
            border.color = if focus.focused {
                style.border_color_focused
            } else if hoverable.hovered {
                style.border_color_hovered
            } else {
                style.border_color
            };
            border.thickness = style.border_thickness;
            border.dimensions = (style.size - Vec2::splat(2.0)).max(Vec2::splat(1.0));
        }

        if let Ok((mut text, mut color)) = label_query.get_mut(parts.label) {
            let show_placeholder = value.0.is_empty() && !focus.focused;
            text.0 = if show_placeholder {
                placeholder.0.clone()
            } else {
                value.0.clone()
            };
            color.0 = if show_placeholder {
                style.placeholder_color
            } else {
                style.text_color
            };
        }

        if let Ok((mut transform, mut visibility, mut sprite)) = caret_query.get_mut(parts.caret) {
            let effective_index = caret_state.index.min(char_count(&value.0));
            let char_advance = (style.font_size * style.text_advance_factor).max(1.0);
            let max_x = style.size.x * 0.5 - style.padding.x;
            let caret_x =
                (-style.size.x * 0.5 + style.padding.x + effective_index as f32 * char_advance)
                    .clamp(-style.size.x * 0.5 + style.padding.x, max_x);
            transform.translation.x = caret_x;
            transform.translation.y = 0.0;
            transform.translation.z = 0.03;
            *visibility = if focus.focused && caret_state.visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            sprite.custom_size = Some(Vec2::new(
                style.caret_width.max(1.0),
                style.font_size.max(1.0),
            ));
            sprite.color = style.caret_color;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::IntoSystem;

    fn setup_keyboard_test_input(world: &mut World, owner: Entity) -> Entity {
        world
            .spawn((
                TextInputBox::new(owner, UiLayerKind::Base),
                TextInputBoxFocus { focused: true },
                TextInputBoxValue::default(),
                TextInputBoxPlaceholder::new("search"),
                TextInputBoxLimits { max_chars: 8 },
                TextInputBoxCaretState::default(),
                InheritedVisibility::VISIBLE,
            ))
            .id()
    }

    #[test]
    fn text_input_box_insert_hook_wires_clickable_layer_and_parts() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let owner = app.world_mut().spawn_empty().id();
        let entity = app
            .world_mut()
            .spawn((TextInputBox::new(owner, UiLayerKind::Base),))
            .id();

        app.update();

        let layer = app
            .world()
            .entity(entity)
            .get::<UiLayer>()
            .copied()
            .expect("text input box layer");
        assert_eq!(layer.owner, owner);
        assert_eq!(layer.kind, UiLayerKind::Base);
        assert!(app
            .world()
            .entity(entity)
            .contains::<Clickable<TextInputBoxActions>>());
        assert!(app.world().entity(entity).contains::<TextInputBoxParts>());
    }

    #[test]
    fn text_input_box_can_skip_ui_layer_insertion() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let owner = app.world_mut().spawn_empty().id();
        let entity = app
            .world_mut()
            .spawn((TextInputBox::new(owner, UiLayerKind::Base).without_ui_layer(),))
            .id();

        app.update();

        let text_input = app
            .world()
            .entity(entity)
            .get::<TextInputBox>()
            .copied()
            .expect("text input box");
        assert!(!text_input.inserts_ui_layer());
        assert!(app.world().entity(entity).get::<UiLayer>().is_none());
    }

    #[test]
    fn keyboard_input_updates_value_and_emits_changed_message() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<TextInputBoxChanged>();
        app.add_message::<TextInputBoxSubmitted>();
        app.add_message::<TextInputBoxCancelled>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<UiInteractionState>();

        let owner = app.world_mut().spawn_empty().id();
        let entity = setup_keyboard_test_input(app.world_mut(), owner);
        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyA);
        assert!(app
            .world()
            .resource::<ButtonInput<KeyCode>>()
            .just_pressed(KeyCode::KeyA));
        assert!(app
            .world()
            .entity(entity)
            .get::<TextInputBoxFocus>()
            .is_some_and(|focus| focus.focused));
        let interactable = {
            let world = app.world();
            let text_input = world
                .entity(entity)
                .get::<TextInputBox>()
                .expect("text input component");
            let policy = world.entity(entity).get::<UiInputPolicy>();
            is_input_interactable(
                text_input,
                policy,
                None,
                world.resource::<UiInteractionState>(),
            )
        };
        assert!(interactable);
        let mut system = IntoSystem::into_system(handle_text_input_keyboard);
        system.initialize(app.world_mut());
        system
            .run((), app.world_mut())
            .expect("text input keyboard system should run");
        system.apply_deferred(app.world_mut());

        let value = app
            .world()
            .entity(entity)
            .get::<TextInputBoxValue>()
            .expect("text input value");
        assert_eq!(value.0, "a");

        let mut reader = app
            .world_mut()
            .resource_mut::<Messages<TextInputBoxChanged>>()
            .get_cursor();
        let emitted: Vec<TextInputBoxChanged> = reader
            .read(app.world().resource::<Messages<TextInputBoxChanged>>())
            .cloned()
            .collect();
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].entity, entity);
        assert_eq!(emitted[0].value, "a");
    }

    #[test]
    fn text_input_box_systems_initialize_without_query_alias_panics() {
        let mut world = World::new();

        let mut focus_system = IntoSystem::into_system(apply_text_input_click_focus);
        focus_system.initialize(&mut world);

        let mut keyboard_system = IntoSystem::into_system(handle_text_input_keyboard);
        keyboard_system.initialize(&mut world);

        let mut blink_system = IntoSystem::into_system(tick_text_input_caret);
        blink_system.initialize(&mut world);

        let mut visuals_system = IntoSystem::into_system(sync_text_input_visuals);
        visuals_system.initialize(&mut world);
    }
}
