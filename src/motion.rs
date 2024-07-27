use bevy::prelude::*;

#[derive(Component)]
pub struct PointToPointTranslation {
    start: Vec3,
    end: Vec3,
    speed: f32,
    has_started: bool,
    has_finished: bool
}

impl PointToPointTranslation {
    pub fn new(start: Vec3, end: Vec3, duration_seconds: f32) -> PointToPointTranslation {
        let distance: f32 = (end - start).length();
        let speed: f32 = distance / duration_seconds;

        PointToPointTranslation {
            start,
            end,
            speed,
            has_started: false,
            has_finished: false,
        }
    }

    pub fn set_duration(
            &mut self, 
            new_duration_seconds : f32
        ) {

        let distance: f32 = (self.end - self.start).length();
        self.speed =  distance / new_duration_seconds;
    }

    pub fn start(&mut self) {
        self.has_started = true;
    }
    
    pub fn end(&mut self) {
        if self.has_started {
            self.has_finished = true;
        } else {
            panic!("Cannot end motion that has not started!");
        }
    }
}

pub fn point_to_point_translations(
        time: Res<Time>, 
        mut query: Query<(&mut PointToPointTranslation, &mut Transform)>
    ) {

    for (mut motion, mut transform) in query.iter_mut() {
        if motion.has_started && !motion.has_finished {
            let direction = (motion.end - motion.start).normalize();
            let distance_to_travel = motion.speed * time.delta_seconds();
            let current_position = transform.translation;
            
            let distance_to_end = (motion.end - current_position).length();
            
            if distance_to_travel >= distance_to_end {
                transform.translation = motion.end;
                motion.has_finished = true;
            } else {
                transform.translation += direction * distance_to_travel;
            }
        }
    }
}