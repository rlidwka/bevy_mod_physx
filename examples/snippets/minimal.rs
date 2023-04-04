use bevy::prelude::*;
use bevy_physx::prelude::*;
use bevy_physx::prelude as bpx;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysXPlugin::default())
        .add_plugin(PhysXDebugRenderPlugin)
        .add_plugin(bevy_inspector_egui::quick::WorldInspectorPlugin::default())
        .add_startup_system(setup)
        .run();
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
) {
    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(1000.0).into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        },
        RigidBody::Static,
        bpx::Shape {
            geometry: px_geometries.add(bpx::Geometry::halfspace(Vec3::Y)),
            ..default()
        },
    ));

    // static actor
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.5, ..default() } )),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(-2.0, 5.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        bpx::Shape {
            geometry: px_geometries.add(bpx::Geometry::ball(0.5)),
            ..default()
        },
    ));

    // dynamic actor
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(2.0, 5.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Velocity::default(),
        bpx::Shape {
            geometry: px_geometries.add(bpx::Geometry::cuboid(0.5, 0.5, 0.5)),
            ..default()
        },
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
