use bevy::prelude::*;
use bevy_physx::prelude::*;
use bevy_physx::prelude as bpx;
use bevy_physx::physx::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysXPlugin::default())
        .add_plugin(PhysXDebugRenderPlugin)
        .add_startup_system(setup)
        .add_system(update)
        .run();
}

pub fn update(
    query: Query<&Velocity>,
) {
    for velocity in query.iter() {
        println!("linear  velocity: {:?}", velocity.linear);
        println!("angular velocity: {:?}", velocity.angular);
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // actor with velocity
    commands.spawn((
        bpx::RigidBody::Dynamic,
        Velocity::default(),
        ExternalForce { force: Vec3::X, torque: Vec3::X, mode: ForceMode::Force },
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.5, ..default() } )),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(-2.0, 7.0, 0.0),
            ..default()
        }
    ));

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 2.5, 10.0),
        ..default()
    });

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
        ..default()
    });
}
