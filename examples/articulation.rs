mod flying_camera;

use bevy::pbr::DirectionalLightShadowMap;
use bevy::prelude::*;
use flying_camera::*;

use bevy_physx::prelude::*;
use bevy_physx::prelude as bpx;
use physx::prelude::*;
use physx_sys::PxSolverType;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(Msaa::default())
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_inspector_egui::quick::WorldInspectorPlugin::default())
        .add_system(bevy::window::close_on_esc)
        .add_plugin(PhysXPlugin {
            scene: bpx::SceneDescriptor {
                solver_type: PxSolverType::Tgs,
                ..default()
            },
            ..default()
        })
        .add_plugin(PhysXDebugRenderPlugin)
        .add_plugin(FlyingCameraPlugin)
        .add_startup_system(spawn_light)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_long_chain)
        .add_startup_system(spawn_obstacle)
        .add_startup_system(spawn_plane)
        .run();
}

fn spawn_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 15000.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            rotation: Quat::from_euler(EulerRot::XYZ, -1.2, -0.2, 0.),
            ..default()
        },
        ..default()
    })
    .insert(Name::new("Light"));
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(FlyingCameraBundle {
        flying_camera: FlyingCamera {
            distance: 40.,
            ..default()
        },
        ..default()
    })
    .insert(Name::new("Camera"));
}

fn spawn_long_chain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
) {
    const SCALE: f32 = 0.25;
    const RADIUS: f32 = 0.5 * SCALE;
    const HALF_HEIGHT: f32 = 1. * SCALE;
    const SEGMENTS: usize = 40;

    // change this for another kind of rope
    const OVERLAPPING_LINKS: bool = true;

    let mut position = Vec3::new(0., 24., 0.);
    let mesh = meshes.add(Mesh::from(shape::Capsule { radius: RADIUS, depth: HALF_HEIGHT + RADIUS * 2., ..default() }));
    let material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());

    let px_geometry = px_geometries.add(bpx::Geometry::capsule(HALF_HEIGHT, RADIUS));
    let mut parent_link = None;

    for _ in 0..SEGMENTS {
        let mut builder = commands.spawn_empty();
        builder
            .insert(bpx::RigidBody::ArticulationLink)
            .insert(SpatialBundle::from_transform(Transform::from_translation(position)))
            .insert(Damping {
                linear: 0.1,
                angular: 0.1,
            })
            .insert(MaxVelocity {
                linear: 100.,
                angular: 30.,
            })
            .insert(bpx::Shape {
                geometry: px_geometry.clone(),
                ..default()
            })
            .insert(MassProperties::mass(1.))
            .with_children(|builder| {
                builder.spawn(PbrBundle {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    transform: Transform::from_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                    ..default()
                });
            });

        if let Some(parent_link) = parent_link {
            let parent_pose;
            let child_pose;

            if OVERLAPPING_LINKS {
                parent_pose = Transform::from_translation(Vec3::new(HALF_HEIGHT, 0., 0.));
                child_pose = Transform::from_translation(Vec3::new(-HALF_HEIGHT, 0., 0.));
            } else {
                parent_pose = Transform::from_translation(Vec3::new(RADIUS + HALF_HEIGHT, 0., 0.));
                child_pose = Transform::from_translation(Vec3::new(-RADIUS - HALF_HEIGHT, 0., 0.));
            }

            // need to configure inbound joint
            builder.insert(ArticulationJoint {
                parent: parent_link,
                parent_pose,
                child_pose,
                joint_type: ArticulationJointType::Spherical,
                motion_swing1: ArticulationMotion::Free,
                motion_swing2: ArticulationMotion::Free,
                motion_twist: ArticulationMotion::Free,
                friction_coefficient: 1.,
                ..default()
            });
        } else {
            // articulation root
            builder.insert(ArticulationRoot {
                fix_base: true,
                ..default()
            });
        }

        let id = builder.id();

        if OVERLAPPING_LINKS {
            position.x += RADIUS + HALF_HEIGHT * 2.;
        } else {
            position.x += (RADIUS + HALF_HEIGHT) * 2.;
        }

        parent_link = Some(id);
    }
}

fn spawn_obstacle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut physics: ResMut<bpx::Physics>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
    mut px_materials: ResMut<Assets<bpx::Material>>,
) {
    const HALF_X: f32 = 1.;
    const HALF_Y: f32 = 0.1;
    const HALF_Z: f32 = 2.;

    let mesh = meshes.add(Mesh::from(
        shape::Box { min_x: -HALF_X, max_x: HALF_X, min_y: -HALF_Y, max_y: HALF_Y, min_z: -HALF_Z, max_z: HALF_Z }
    ));
    let material = materials.add(Color::rgb(0.8, 0.2, 0.3).into());

    let px_geometry = px_geometries.add(bpx::Geometry::cuboid(HALF_X, HALF_Y, HALF_Z));
    let px_material = px_materials.add(bpx::Material::new(&mut physics, 0.5, 0.5, 0.6));
    let transform = Transform::from_translation(Vec3::new(10., 21., 0.));

    commands.spawn_empty()
        .insert(PbrBundle {
            mesh,
            material,
            transform,
            ..default()
        })
        .insert(bpx::RigidBody::Static)
        .insert(bpx::Shape {
            material: px_material,
            geometry: px_geometry,
            ..default()
        })
        .insert(Name::new("Obstacle"));
}

fn spawn_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut physics: ResMut<bpx::Physics>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
    mut px_materials: ResMut<Assets<bpx::Material>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 500.0, subdivisions: 4 }));
    let material = materials.add(Color::rgb(0.3, 0.5, 0.3).into());
    let px_geometry = px_geometries.add(bpx::Geometry::halfspace(Vec3::Y));
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
