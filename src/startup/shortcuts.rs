use bevy::prelude::*;
use bevy::app::AppExit;

pub fn close_on_esc(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_exit_events.write(AppExit::Success);
    }
}
