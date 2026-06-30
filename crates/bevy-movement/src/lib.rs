use bevy::prelude::*;

/// Stores movement parameters and runtime state for an entity.
#[derive(Component, Default)]
pub struct MovementData {
    pub max_speed: f32,
    pub friction: f32,
    pub acceleration: f32,

    pub current_speed: f32,
    pub direction: Vec3,
    last_direction: Vec3,
}

/// Moves entities based on their [`MovementData`].
///
/// Accelerates when `direction` is set, decelerates via friction when cleared.
/// Add to your app with `app.add_systems(Update, movement_system)`.
pub fn movement_system(mut query: Query<(&mut Transform, &mut MovementData)>, time: Res<Time>) {
    for (mut transform, mut data) in query.iter_mut() {
        if data.direction != Vec3::ZERO {
            data.last_direction = data.direction.normalize_or_zero();
            data.current_speed = (data.current_speed + data.acceleration * time.delta_secs())
                .clamp(0.0, data.max_speed);
        } else {
            data.current_speed = (data.current_speed - data.friction * time.delta_secs())
                .max(0.0);
        }

        transform.translation += data.last_direction * data.current_speed * time.delta_secs();
    }
}
