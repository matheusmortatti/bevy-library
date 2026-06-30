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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    const DT: f32 = 0.1;

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(DT)));
        app.add_systems(Update, movement_system);
        app
    }

    fn spawn(app: &mut App, data: MovementData) -> Entity {
        app.world_mut().spawn((Transform::default(), data)).id()
    }

    // Tick twice: first update initialises time (delta may be 0), second has full DT.
    fn tick2(app: &mut App) {
        app.update();
        app.update();
    }

    #[test]
    fn accelerates_when_direction_set() {
        let mut app = make_app();
        let e = spawn(&mut app, MovementData {
            max_speed: 100.0,
            acceleration: 50.0,
            friction: 0.0,
            direction: Vec3::X,
            ..Default::default()
        });

        tick2(&mut app);

        let t = app.world().get::<Transform>(e).unwrap();
        assert!(t.translation.x > 0.0, "entity must move in +X");
        assert_eq!(t.translation.y, 0.0);
        assert_eq!(t.translation.z, 0.0);
    }

    #[test]
    fn decelerates_when_direction_cleared() {
        let mut app = make_app();
        let e = spawn(&mut app, MovementData {
            max_speed: 100.0,
            acceleration: 0.0,
            friction: 10.0,
            direction: Vec3::ZERO,
            current_speed: 50.0,
            last_direction: Vec3::X,
        });

        tick2(&mut app);

        let data = app.world().get::<MovementData>(e).unwrap();
        assert!(data.current_speed < 50.0, "speed must decrease via friction");
    }

    #[test]
    fn speed_clamped_to_max() {
        let mut app = make_app();
        let e = spawn(&mut app, MovementData {
            max_speed: 10.0,
            acceleration: 1000.0,
            friction: 0.0,
            direction: Vec3::X,
            ..Default::default()
        });

        for _ in 0..20 {
            app.update();
        }

        let data = app.world().get::<MovementData>(e).unwrap();
        assert!(
            data.current_speed <= 10.0,
            "speed {} exceeded max_speed 10.0",
            data.current_speed
        );
    }

    #[test]
    fn no_movement_when_stationary_and_no_direction() {
        let mut app = make_app();
        let e = spawn(&mut app, MovementData {
            max_speed: 100.0,
            acceleration: 50.0,
            friction: 5.0,
            direction: Vec3::ZERO,
            ..Default::default()
        });

        tick2(&mut app);

        let t = app.world().get::<Transform>(e).unwrap();
        assert_eq!(t.translation, Vec3::ZERO, "entity must not move");
    }

    #[test]
    fn coasts_in_last_direction_after_direction_cleared() {
        let mut app = make_app();
        let e = spawn(&mut app, MovementData {
            max_speed: 100.0,
            acceleration: 0.0,
            friction: 5.0,
            direction: Vec3::ZERO,
            current_speed: 50.0,
            last_direction: Vec3::X,
        });

        tick2(&mut app);

        let t = app.world().get::<Transform>(e).unwrap();
        assert!(t.translation.x > 0.0, "entity must coast in +X");
        assert_eq!(t.translation.y, 0.0);
        assert_eq!(t.translation.z, 0.0);
    }

    #[test]
    fn diagonal_direction_normalised() {
        let mut app = make_app();
        let e = spawn(&mut app, MovementData {
            max_speed: 100.0,
            acceleration: 50.0,
            friction: 0.0,
            direction: Vec3::new(1.0, 1.0, 0.0), // not normalised
            ..Default::default()
        });

        tick2(&mut app);

        let data = app.world().get::<MovementData>(e).unwrap();
        // last_direction must be a unit vector (stored normalised)
        let len = data.last_direction.length();
        assert!(
            (len - 1.0).abs() < 1e-5,
            "last_direction length {len} must be 1"
        );
    }
}
