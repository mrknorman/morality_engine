use bevy::prelude::*;

#[derive(Resource)]
pub struct Dilation(pub f32);

pub struct DilationPLugin;
impl Plugin for DilationPLugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Dilation(1.0));
    }
}
