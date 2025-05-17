use bevy::{
    input::common_conditions::input_just_released, log::tracing::Instrument, prelude::*,
    ui::RelativeCursorPosition,
};

const MIN_DISTANCE: f32 = 1.0;

// Define a struct to keep some information about our entity.
// Here it's an arbitrary movement speed, the spawn location, and a maximum distance from it.
#[derive(Component, Debug)]
struct Movable {
    spawn: Vec3,
    max_distance: f32,
    speed: f32,
}

// Implement a utility function for easier Movable struct creation.
impl Movable {
    fn new(spawn: Vec3) -> Self {
        Movable {
            spawn,
            max_distance: 300.0,
            speed: 60.0,
        }
    }
}

#[derive(Component)]
struct Follower {
    spawn: Vec3,
    speed: f32,
    target: Entity,
}
impl Follower {
    fn new(spawn: Vec3, target: Entity) -> Self {
        Follower {
            spawn,
            speed: 110.0,
            target,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, move_target)
        .add_systems(
            Update,
            spawn_projectile.run_if(input_just_released(MouseButton::Left)),
        )
        .add_systems(Update, move_projectile)
        .run();
}

// Startup system to setup the scene and spawn all relevant entities.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Add a shape to visualize translation.
    let entity_spawn = Vec3::ZERO;
    commands.spawn((
        Mesh2d(meshes.add(RegularPolygon::new(30.0, 8))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_translation(entity_spawn),
        Movable::new(entity_spawn),
    ));

    // Spawn a camera looking at the entities to show what's happening in this example.
    commands.spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 0.0)));
}

fn spawn_projectile(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut targets: Single<(Entity, &mut Movable)>,
    // relative_cursor_position: Single<&RelativeCursorPosition>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Query<&Window>,
) {
    let (camera, camera_transform) = *camera_query;
    let Ok(window) = window.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Calculate a world position based on the cursor's position.
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let entity_spawn = Vec3::new(world_pos.x, world_pos.y, 0.0);

    let target_entity = targets.0;
    commands.spawn((
        Mesh2d(meshes.add(Triangle2d::new(
            Vec2::Y * 5.0,
            Vec2::new(-5.0, -5.0),
            Vec2::new(5.0, -5.0),
        ))),
        MeshMaterial2d(materials.add(Color::from(bevy::color::palettes::basic::RED))),
        Transform::from_translation(entity_spawn),
        Follower::new(entity_spawn, target_entity),
    ));
}

// This system will move all Movable entities with a Transform
fn move_target(mut targets: Query<(&mut Transform, &mut Movable)>, timer: Res<Time>) {
    for (mut transform, mut target) in &mut targets {
        // Check if the entity moved too far from its spawn, if so invert the moving direction.
        if (target.spawn - transform.translation).length() > target.max_distance {
            target.speed *= -1.0;
        }
        let direction = transform.local_x();
        transform.translation += direction * target.speed * timer.delta_secs();

        // transform.rotate_z(target.speed * timer.delta_secs());
        // transform.rotation += Quat::from_rotation_arc(Vec3::Y, target.speed * timer.delta_secs());
    }
}

fn move_projectile(
    mut projectiles: Query<(&mut Transform, &mut Follower)>,
    mut targets: Query<(&Transform), Without<Follower>>,
    mut commands: Commands,
    timer: Res<Time>,
) {
    for (mut transform, mut projectile) in &mut projectiles {
        // projectile.targ
        let target_transform = targets.get(projectile.target).unwrap();

        let direction = target_transform.translation - transform.translation;

        // despawn if direction is smaller ...
        if direction.length() < MIN_DISTANCE {
            // commands.entity(projectile).despawn();
        }

        transform.translation += direction.normalize() * projectile.speed * timer.delta_secs();
        // move closer to target
        // trasnform.translation

        // rotate to target
        let rotate_to_target =
            Quat::from_rotation_arc(Vec3::Y, direction.xy().normalize().extend(0.));
        transform.rotation = rotate_to_target;
    }
}
