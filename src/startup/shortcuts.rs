use bevy::{
    prelude::*,
    window::{ClosingWindow, PrimaryWindow, WindowCloseRequested},
};

pub fn close_on_esc(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    primary_window: Query<Entity, (With<Window>, With<PrimaryWindow>, Without<ClosingWindow>)>,
    mut close_requests: MessageWriter<WindowCloseRequested>,
) {
    if !keyboard_input.just_pressed(KeyCode::Escape) {
        return;
    }

    if let Ok(window) = primary_window.single() {
        close_requests.write(WindowCloseRequested { window });
    }
}
