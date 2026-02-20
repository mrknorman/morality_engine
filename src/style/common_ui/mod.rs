//! style/common_ui/mod.rs
use crate::style::ui::BottomAnchor;
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

/// ---------------------------------------------------------------------------
/// Macro for “exactly-one” UI elements
/// ---------------------------------------------------------------------------
///
/// ```rust
/// unique_element! {
///     struct NextButton,
///     config: NextButtonConfig,
///     distance: 100.0
/// }
/// ```
///
/// * `config:` – the `Resource` that remembers which entity is alive.
/// * `distance:` – vertical offset from the bottom of the first `Window`.
/// * optional `rot:` – radians of Z-rotation (e.g. `rot: PI / 2.0`).
macro_rules! unique_element {
    (
        $(#[$meta:meta])*
        struct $Name:ident,
        config: $Config:ident,
        distance: $distance:expr
        $(, rot: $rot:expr)?
        $(,)?
    ) => {
        // ── Config resource ──────────────────────────────────────────────────
        #[derive(Resource, Default)]
        pub struct $Config(pub Option<Entity>);

        // ── Component ────────────────────────────────────────────────────────
        $(#[$meta])*
        #[derive(Component, Clone)]
        #[require(Transform, Visibility)]
        #[component(on_insert = $Name::on_insert)]
        pub struct $Name;

        impl $Name {
            /// Compute the element’s transform from the current window.
            fn transform(window: &Window) -> (Transform, BottomAnchor) {
                let y = -window.height() / 2.0 + $distance;
                let t = Transform::from_xyz(0.0, y, 1.0);
                $(
                    let t = t.with_rotation(Quat::from_rotation_z($rot));
                )?
                (t, BottomAnchor {distance : $distance})
            }

            /// Shared hook: place the new element, despawn the old, update the
            /// config. 100 % identical for every element – only the types differ.
            fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
                let transform = world
                    .try_query::<&Window>()                      // get a query for all Window components
                    .and_then(|mut q| q.iter(&world).next())     // iterate and take the first one
                    .map(|w| Self::transform(w));

                // Is there a previous one?
                let previous = world
                    .get_resource::<$Config>()
                    .and_then(|cfg| cfg.0)
                    .filter(|&e| world.get_entity(e).is_ok());

                // Apply commands.
                let mut cmd = world.commands();
                if let Some(t) = transform { cmd.entity(entity).insert(t); }
                if let Some(old) = previous { cmd.entity(old).despawn(); }

                // Remember this entity.
                if let Some(mut cfg) = world.get_resource_mut::<$Config>() {
                    cfg.0 = Some(entity);
                }
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Concrete UI elements (only the data – no logic duplication 👍)
// ---------------------------------------------------------------------------
unique_element!(
    /// “Next” button at 100 px from the bottom-centre of the screen.
    struct NextButton,
    config: NextButtonConfig,
    distance: 100.0,
);

unique_element!(
    /// Lever at 150 px, rotated 90 °.
    struct CenterLever,
    config: CenterLeverConfig,
    distance: 150.0,
    rot: std::f32::consts::PI / 2.0,
);

unique_element!(
    /// Dilemma-timer at 250 px.
    struct DilemmaTimerPosition,
    config: DilemmaTimerConfig,
    distance: 250.0,
);

// ---------------------------------------------------------------------------
// Plugin – after all types are visible
// ---------------------------------------------------------------------------
pub struct CommonUIPlugin;
impl Plugin for CommonUIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NextButtonConfig::default())
            .insert_resource(CenterLeverConfig::default())
            .insert_resource(DilemmaTimerConfig::default());
    }
}
