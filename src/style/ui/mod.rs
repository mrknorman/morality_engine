//! style/io/mod.rs
use bevy::{
    prelude::*, window::PrimaryWindow,
};

/// ---------------------------------------------------------------------------
///     State that turns anchor systems on/off
/// ---------------------------------------------------------------------------
#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum IOSystemsActive {
    #[default]
    False,
    True,
}

fn activate_systems(
    mut next: ResMut<NextState<IOSystemsActive>>,
    anchors: Query<&BottomAnchor>,
) {

    // “Any anchors alive?”  →  True else False
    next.set(if !anchors.is_empty() {
        IOSystemsActive::True
    } else {
        IOSystemsActive::False
    });
}

/// ---------------------------------------------------------------------------
///     Plugin
/// ---------------------------------------------------------------------------
pub struct IOPlugin;
impl Plugin for IOPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<IOSystemsActive>()
            .add_systems(Update, activate_systems)
            .add_systems(
                Update,
                
                    BottomAnchor::update
                
                .run_if(in_state(IOSystemsActive::True)),
            );
    }
}

/// ---------------------------------------------------------------------------
///     Bottom-of-screen anchoring
/// ---------------------------------------------------------------------------
#[derive(Component)]
pub struct BottomAnchor {
    pub distance: f32,
}

impl Default for BottomAnchor {
    fn default() -> Self {
        Self { distance: 100.0 }
    }
}

impl BottomAnchor {
    /// System: keep every `BottomAnchor` at its desired Y-offset
    pub fn update(
        window: Single<&Window, (With<PrimaryWindow>, Changed<Window>)>,
        mut query: Query<(&Self, &mut Transform, Option<&ChildOf>)>,
        parents: Query<&Transform, Without<Self>>,
    ) {
        let base_y = -window.height() / 2.0; // bottom edge in world-space

        for (anchor, mut transform, parent) in &mut query {
            let mut y = base_y + anchor.distance;
            
            if let Some(child_of) = parent {
                if let Ok(p) = parents.get(child_of.parent()) {
                    y -= p.translation.y; // offset by parent
                }
            }

            transform.translation.y = y;
        }
    }
}