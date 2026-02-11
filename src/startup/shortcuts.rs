use bevy::{
    prelude::*,
    window::{ClosingWindow, PrimaryWindow, WindowCloseRequested},
};

use crate::data::states::MainState;

pub fn close_on_esc(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    main_state: Res<State<MainState>>,
    primary_window: Query<Entity, (With<Window>, With<PrimaryWindow>, Without<ClosingWindow>)>,
    mut close_requests: MessageWriter<WindowCloseRequested>,
) {
    if !keyboard_input.just_pressed(KeyCode::Escape) {
        return;
    }

    if *main_state.get() == MainState::InGame {
        return;
    }

    if let Ok(window) = primary_window.single() {
        close_requests.write(WindowCloseRequested { window });
    }
}
