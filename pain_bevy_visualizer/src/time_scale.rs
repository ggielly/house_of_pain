use bevy::prelude::*;

#[derive(Resource)]
pub struct TimeScale(pub f32);

impl Default for TimeScale {
    fn default() -> Self {
        TimeScale(1.0)
    }
}
