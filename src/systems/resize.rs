use bevy::{
    prelude::*, 
    window::WindowResized
};

pub struct ResizePlugin;
impl Plugin for ResizePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, handle_resize)
            .insert_resource(ResizeDebounce::default());
    }
}

#[derive(Resource)]
pub struct ResizeDebounce {
    pub timer: Timer
}

impl Default for ResizeDebounce {
    fn default() -> Self {

        let mut timer = Timer::from_seconds(0.1, TimerMode::Once);
        timer.pause();
        Self {
            timer
        }
    }
}

fn handle_resize(
    resize_events: EventReader<WindowResized>,
    time: Res<Time>,
    mut debounce: ResMut<ResizeDebounce>,
) {
    // If any resize event is detected, mark as pending and reset timer.
    if !resize_events.is_empty() {
        debounce.timer.reset();
        debounce.timer.unpause();
    }

    debounce.timer.tick(time.delta());
}
