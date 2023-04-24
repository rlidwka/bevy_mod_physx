mod common;

use bevy::prelude::*;
use bevy_physx::prelude::{*, self as bpx};

fn main() {
    // ported from https://github.com/MasterOfMarkets/bevy_mod_physx
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(common::DemoUtils) // optional
        .add_plugin(PhysXPlugin::default())
        .add_startup_system(spawn_scene)
        .add_startup_system(spawn_camera_and_light)
        .run();
}

pub fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut physics: ResMut<Physics>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
    mut px_materials: ResMut<Assets<bpx::Material>>,
) {
    // plane
    commands.spawn((
        RigidBody::Static,
        bpx::Shape {
            geometry: px_geometries.add(bpx::Geometry::halfspace(Vec3::Y)),
            material: px_materials.add(bpx::Material::new(&mut physics, 0., 0., 1.)),
            ..default()
        },
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(1000.0).into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        }
    ));

    // high restitution
    commands.spawn((
        RigidBody::Dynamic,
        bpx::Shape {
            geometry: px_geometries.add(bpx::Geometry::ball(0.5)),
            material: px_materials.add(bpx::Material::new(&mut physics, 0., 0., 1.)),
            ..default()
        },
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.5, ..default() } )),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(-2.0, 5.0, 0.0),
            ..default()
        }
    ));

    // small restitution
    commands.spawn((
        RigidBody::Dynamic,
        bpx::Shape {
            geometry: px_geometries.add(bpx::Geometry::ball(0.5)),
            material: px_materials.add(bpx::Material::new(&mut physics, 0., 0., 0.1)),
            ..default()
        },
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.5, ..default() } )),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(2.0, 5.0, 0.0),
            ..default()
        }
    ));
}

fn spawn_camera_and_light(mut commands: Commands) {
    commands
        .spawn(SpatialBundle::from_transform(Transform::from_xyz(0., 2.5, 0.)))
        .with_children(|builder| {
            builder.spawn(Camera3dBundle {
                transform: Transform::from_xyz(0.0, 2.5, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            });
        })
        .insert(Name::new("Camera"));

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
        ..default()
    }).insert(Name::new("Light"));
}
