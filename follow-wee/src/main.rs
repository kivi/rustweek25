use bevy::{input::common_conditions::input_just_pressed, prelude::*};

const MIN_DISTANCE: f32 = 10.0;

// Target is a moveable with movement speed, the spawn location, and a maximum distance from it.
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
            speed: 70.0,
        }
    }
}

// The follower needs to know target as an Entity and the projectile speed.
#[derive(Component)]
struct Follower {
    speed: f32,
    target: Entity,
}
impl Follower {
    fn new(target: Entity) -> Self {
        Follower {
            speed: 110.0,
            target,
        }
    }
}

// Resource for 2d Mesh handler and material mostly for projectile
#[derive(Resource)]
struct WorldAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    // material: MeshMaterial2d<Handle<ColorMaterial>>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, move_target)
        .add_systems(
            Update,
            spawn_projectile.run_if(input_just_pressed(MouseButton::Left)),
        )
        // projectile is added to Update, if there is any entigy with the Movable component type. This would be the target
        .add_systems(
            Update,
            move_projectile.run_if(any_with_component::<Movable>),
        )
        .run();
}

// Startup system to setup the scene and spawn all relevant entities.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // create mesh handle and material for projectile
    let projectile_asset = WorldAssets {
        mesh: meshes.add(Triangle2d::new(
            Vec2::Y * 5.0,
            Vec2::new(-5.0, -5.0),
            Vec2::new(5.0, -5.0),
        )),
        material: materials.add(Color::from(bevy::color::palettes::basic::RED)),
    };

    commands.insert_resource(projectile_asset);

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
    world_assets: Res<WorldAssets>,
    targets: Single<(Entity, &mut Movable)>,
    camera_query: Single<(&Camera, &GlobalTransform)>,

    window: Query<&Window>,
) -> Result<(), BevyError> {
    let (camera, camera_transform) = *camera_query;
    let window = window.single()?;

    let Some(cursor_position) = window.cursor_position() else {
        return Ok(());
    };

    // Calculate a world position based on the cursor's position.
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return Ok(());
    };

    let entity_spawn = Vec3::new(world_pos.x, world_pos.y, 0.0);

    let target_entity = targets.0;
    commands.spawn((
        Mesh2d(world_assets.mesh.clone()),
        MeshMaterial2d(world_assets.material.clone()),
        Transform::from_translation(entity_spawn),
        Follower::new(target_entity),
    ));

    Ok(())
}

// This system will move all Movable entities with a Transform
fn move_target(mut targets: Query<(&mut Transform, &mut Movable)>, timer: Res<Time>) {
    for (mut transform, mut target) in &mut targets {
        // Check if the entity moved too far from its spawn, if so invert the moving direction.
        if (target.spawn - transform.translation).length() > target.max_distance {
            target.speed *= -1.0;
        }
        let direction = Dir3::X;
        transform.translation += direction * target.speed * timer.delta_secs();

        transform.rotate_z(-1.2 * target.speed.signum() * timer.delta_secs());
    }
}

fn move_projectile(
    mut projectiles: Query<(Entity, &mut Transform, &mut Follower)>,
    targets: Query<&Transform, Without<Follower>>,
    mut commands: Commands,
    timer: Res<Time>,
) {
    for (entity, mut transform, projectile) in &mut projectiles {
        let target_transform = targets.get(projectile.target).unwrap();

        let direction = target_transform.translation - transform.translation;

        if direction.length() < MIN_DISTANCE {
            commands.entity(entity).despawn();
        }

        transform.translation += direction.normalize() * projectile.speed * timer.delta_secs();

        // rotate to point to target
        let rotate_to_target =
            Quat::from_rotation_arc(Vec3::Y, direction.xy().normalize().extend(0.));
        transform.rotation = rotate_to_target;
    }
}
