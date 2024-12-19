use bevy::prelude::*;

pub const PRIMARY_COLOR : bevy::prelude::Color = Color::Srgba(Srgba::new(2.0, 5.0, 2.0, 1.0));
pub const HIGHLIGHT_COLOR : bevy::prelude::Color = Color::Srgba(Srgba::new(3.0, 3.0, 3.0, 1.0)); 
pub const HOVERED_BUTTON: Color = Color::srgb(0.0, 3.0, 3.0);
pub const PRESSED_BUTTON: Color = Color::srgb(4.0, 4.0, 0.0);
pub const DANGER_COLOR: Color = Color::srgb(5.0, 0.0, 0.0);

pub const OPTION_GLOW : f32 = 2.0;
pub const OPTION_1_COLOR : Color = Color::srgb(0.1015*OPTION_GLOW, 0.5195*OPTION_GLOW, 0.9961*OPTION_GLOW);
pub const OPTION_2_COLOR : Color = Color::srgb(0.8314*OPTION_GLOW, 0.0664*OPTION_GLOW, 0.3477*OPTION_GLOW);