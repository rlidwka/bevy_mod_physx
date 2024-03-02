mod common;

use bevy::prelude::*;
use bevy_mod_physx::prelude::{self as bpx, *};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins.set(
            PhysicsCore::new().with_pvd()
        ))
        .add_plugins(common::DemoUtils) // optional
        .add_systems(Startup, (
            spawn_plane,
            spawn_stacks,
            spawn_dynamic,
            spawn_camera_and_light,
        ))
        .run();
}

fn spawn_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut physics: ResMut<Physics>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
    mut px_materials: ResMut<Assets<bpx::Material>>,
) {
    let primitive = Plane3d::default();
    let mesh = meshes.add(primitive.mesh().size(500., 500.));
    let material = materials.add(Color::rgb(0.3, 0.5, 0.3));
    let px_geometry = px_geometries.add(primitive);
    let px_material = px_materials.add(bpx::Material::new(&mut physics, 0.5, 0.5, 0.6));

    commands.spawn_empty()
        .insert(PbrBundle {
            mesh,
            material,
            ..default()
        })
        .insert(bpx::RigidBody::Static)
        .insert(bpx::Shape {
            geometry: px_geometry,
            material: px_material,
            ..default()
        })
        .insert(Name::new("Plane"));
}

fn spawn_stacks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
) {
    const WIDTH: f32 = 0.5;
    const SIZE: usize = 10;

    let primitive = Cuboid::from_size(Vec3::splat(WIDTH));
    let mesh = meshes.add(primitive);
    let material = materials.add(Color::rgb(0.8, 0.7, 0.6));

    let px_geometry = px_geometries.add(primitive);

    for i in 0..5 {
        commands.spawn(SpatialBundle::from_transform(Transform::from_xyz(0., 0., -1.25 * i as f32)))
            .with_children(|builder| {
                for i in 0..SIZE {
                    for j in 0..SIZE-i {
                        let transform = Transform::from_xyz(
                            ((j * 2) as f32 - (SIZE - i) as f32) / 2. * WIDTH,
                            (i * 2 + 1) as f32 / 2. * WIDTH,
                            0.,
                        );

                        builder.spawn_empty()
                            .insert(PbrBundle {
                                mesh: mesh.clone(),
                                material: material.clone(),
                                transform,
                                ..default()
                            })
                            .insert(bpx::RigidBody::Dynamic)
                            .insert(MassProperties::density(10.))
                            .insert(bpx::Shape {
                                geometry: px_geometry.clone(),
                                ..default()
                            });
                    }
                }
            })
            .insert(Name::new(format!("Stack {i}")));
    }
}

fn spawn_dynamic(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut physics: ResMut<Physics>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
    mut px_materials: ResMut<Assets<bpx::Material>>,
) {
    const RADIUS: f32 = 1.25;

    let primitive = Sphere::new(RADIUS);
    let mesh = meshes.add(primitive);
    let material = materials.add(Color::rgb(0.8, 0.7, 0.6));

    let px_geometry = px_geometries.add(primitive);
    let px_material = px_materials.add(bpx::Material::new(&mut physics, 0.5, 0.5, 0.6));

    let transform = Transform::from_xyz(0., 5., 12.5);

    commands.spawn_empty()
        .insert(PbrBundle {
            mesh,
            material,
            transform,
            ..default()
        })
        .insert(bpx::RigidBody::Dynamic)
        .insert(MassProperties::density(10.))
        .insert(bpx::Shape {
            material: px_material,
            geometry: px_geometry,
            ..default()
        })
        .insert(Velocity::linear(Vec3::new(0., -6.25, -12.5)))
        .insert(Name::new("Ball"));
}

fn spawn_camera_and_light(mut commands: Commands) {
    commands
        .spawn(SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)))
        .with_children(|builder| {
            builder.spawn(Camera3dBundle {
                transform: Transform::from_xyz(-32.5, 13.6, 18.8).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            });
        })
        .insert(Name::new("Camera"));

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.2, -0.2, 0.)),
        ..default()
    })
    .insert(Name::new("Light"));
}
