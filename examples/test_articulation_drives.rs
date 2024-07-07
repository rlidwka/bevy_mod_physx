mod common;

use bevy::prelude::*;
use bevy_mod_physx::prelude::{self as bpx, *};
use physx::prelude::*;
use physx_sys::PxArticulationDrive;

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
            spawn_camera_and_light
        ))
        .run();
}

#[allow(clippy::redundant_clone)]
pub fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut physics: ResMut<bpx::Physics>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
    mut px_materials: ResMut<Assets<bpx::Material>>,
) {
    // plane
    let primitive = Plane3d::default();
    commands.spawn((
        bpx::RigidBody::Static,
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

    let primitive_root = Cuboid::from_size(Vec3::splat(0.5));
    let bevy_material = materials.add(Color::srgb(0.8, 0.7, 0.6));
    let root_mesh = meshes.add(primitive_root);
    let root_geometry = px_geometries.add(primitive_root);
    let primitive_part = Sphere::new(0.2);
    let part_mesh = meshes.add(primitive_part);
    let part_geometry = px_geometries.add(primitive_part);
    let limited_joint = ArticulationJointMotion::Limited {
        min: -std::f32::consts::FRAC_PI_4,
        max: std::f32::consts::FRAC_PI_4,
    };

    let root = commands.spawn((
        PbrBundle {
            mesh: root_mesh,
            material: bevy_material.clone(),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        bpx::RigidBody::ArticulationLink,
        bpx::Shape {
            geometry: root_geometry,
            ..default()
        },
        MassProperties::density(10000.),
    )).id();

    let drive_cfg = PxArticulationDrive { stiffness: 1000.0, damping: 100.0, maxForce: 100.0, driveType: ArticulationDriveType::Acceleration };
    let part_1 = commands.spawn((
        PbrBundle {
            mesh: part_mesh.clone(),
            material: bevy_material.clone(),
            transform: Transform::from_xyz(0.0, 1.5, 0.0),
            ..default()
        },
        bpx::RigidBody::ArticulationLink,
        bpx::Shape {
            geometry: part_geometry.clone(),
            ..default()
        },
        ArticulationJoint {
            parent: root,
            parent_pose: Transform::from_xyz(0.0, 0.3, 0.0),
            child_pose: Transform::from_xyz(0.0, -0.3, 0.0),
            joint_type: ArticulationJointType::Spherical,
            motion_swing1: limited_joint,
            motion_swing2: limited_joint,
            motion_twist: limited_joint,
            friction_coefficient: 1.,
            ..default()
        },
        ArticulationJointDrives {
            twist: drive_cfg,
            swing1: drive_cfg,
            swing2: drive_cfg,
            ..default()
        },
    )).id();

    let drive_cfg = PxArticulationDrive { stiffness: 100.0, damping: 10.0, maxForce: 100.0, driveType: ArticulationDriveType::Acceleration };
    let _part_2_1 = commands.spawn((
        PbrBundle {
            mesh: part_mesh.clone(),
            material: bevy_material.clone(),
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..default()
        },
        bpx::RigidBody::ArticulationLink,
        bpx::Shape {
            geometry: part_geometry.clone(),
            ..default()
        },
        ArticulationJoint {
            parent: part_1,
            parent_pose: Transform::from_xyz(-0.3, 0.3, 0.0),
            child_pose: Transform::from_xyz(0.0, -0.3, 0.0),
            joint_type: ArticulationJointType::Spherical,
            motion_swing1: limited_joint,
            motion_swing2: limited_joint,
            motion_twist: limited_joint,
            friction_coefficient: 1.,
            ..default()
        },
        ArticulationJointDrives {
            twist: drive_cfg,
            swing1: drive_cfg,
            swing2: drive_cfg,
            ..default()
        },
        MassProperties::density(1000.),
    )).id();

    let drive_cfg = PxArticulationDrive { stiffness: 100.0, damping: 10.0, maxForce: 100.0, driveType: ArticulationDriveType::Acceleration };
    let _part_2_2 = commands.spawn((
        PbrBundle {
            mesh: part_mesh.clone(),
            material: bevy_material.clone(),
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..default()
        },
        bpx::RigidBody::ArticulationLink,
        bpx::Shape {
            geometry: part_geometry.clone(),
            ..default()
        },
        ArticulationJoint {
            parent: part_1,
            parent_pose: Transform::from_xyz(0.3, 0.3, 0.0),
            child_pose: Transform::from_xyz(0.0, -0.3, 0.0),
            joint_type: ArticulationJointType::Spherical,
            motion_swing1: limited_joint,
            motion_swing2: limited_joint,
            motion_twist: limited_joint,
            friction_coefficient: 1.,
            ..default()
        },
        ArticulationJointDrives {
            twist: drive_cfg,
            swing1: drive_cfg,
            swing2: drive_cfg,
            ..default()
        },
        MassProperties::density(0.5),
    )).id();
}

fn spawn_camera_and_light(mut commands: Commands) {
    commands
        .spawn(SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)))
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
