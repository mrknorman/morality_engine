use bevy::{prelude::*, time::Timer};

#[derive(Component)]
pub struct Narration {
    pub timer : Timer
}

pub fn start_narration(
    time: Res<Time>,
    mut query: Query<(&mut Narration, &mut AudioSink), With<Narration>>
) {
    for (mut timer, sink) in query.iter_mut() {
        if timer.timer.tick(time.delta()).just_finished() {
            sink.play();
        }
    }
}
