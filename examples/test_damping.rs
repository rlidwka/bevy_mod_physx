mod common;

use bevy::prelude::*;
use bevy_mod_physx::prelude::{self as bpx, *};

fn main() {
    // ported from https://github.com/MasterOfMarkets/bevy_mod_physx
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins.set(
            PhysicsCore::new().with_pvd()
        ))
        .add_plugins(common::DemoUtils) // optional
        .add_systems(Startup, (
            spawn_scene,
            spawn_camera_and_light,
        ))
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
    let primitive = Plane3d::default();
    commands.spawn((
        RigidBody::Static,
        bpx::Shape {
            geometry: px_geometries.add(primitive),
            material: px_materials.add(bpx::Material::new(&mut physics, 0., 0., 1.)),
            ..default()
        },
        PbrBundle {
            mesh: meshes.add(primitive.mesh().size(1000., 1000.)),
            material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
            ..default()
        }
    ));

    // no damping
    let primitive = Sphere::new(0.5);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(primitive),
            material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(-2.0, 7.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        bpx::Shape {
            geometry: px_geometries.add(primitive),
            material: px_materials.add(bpx::Material::new(&mut physics, 0., 0., 1.)),
            ..default()
        },
        Damping { linear: 0., angular: 0. },
    ));

    // high damping
    let primitive = Sphere::new(0.5);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(primitive),
            material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(2.0, 7.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        bpx::Shape {
            geometry: px_geometries.add(primitive),
            material: px_materials.add(bpx::Material::new(&mut physics, 0., 0., 1.)),
            ..default()
        },
        Damping { linear: 1., angular: 1. },
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
