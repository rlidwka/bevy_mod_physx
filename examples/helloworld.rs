mod flying_camera;

use bevy::prelude::*;
use flying_camera::*;

use bevy_physx::prelude::*;
use bevy_physx::prelude as bpx;
use bevy_physx::components::{BPxActor, BPxVelocity, BPxShape, BPxMassProperties};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(Msaa::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                present_mode: bevy::window::PresentMode::Immediate,
                ..default()
            },
            ..default()
        }))
        .add_plugin(bevy_inspector_egui::quick::WorldInspectorPlugin)
        .add_system(bevy::window::close_on_esc)
        .add_plugin(PhysXPlugin::default())
        .add_plugin(FlyingCameraPlugin)
        .add_startup_system(spawn_light)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_plane)
        .add_startup_system(spawn_stacks)
        .add_startup_system(spawn_dynamic)
        .run();
}

fn spawn_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 20000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 1000.0, 10.0),
        ..default()
    })
    .insert(Name::new("Light"));
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(FlyingCameraBundle {
        flying_camera: FlyingCamera {
            distance: 60.,
            ..default()
        },
        ..default()
    })
    .insert(Name::new("Camera"));
}

fn spawn_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut physics: ResMut<Physics>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
    mut px_materials: ResMut<Assets<bpx::Material>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 500.0 }));
    let material = materials.add(Color::rgb(0.3, 0.5, 0.3).into());
    let px_geometry = px_geometries.add(bpx::Geometry::halfspace());
    let px_material = px_materials.add(bpx::Material::new(&mut physics, 0.5, 0.5, 0.6));

    commands.spawn_empty()
        .insert(PbrBundle {
            mesh,
            material,
            ..default()
        })
        .with_children(|builder| {
            builder.spawn_empty()
                .insert(TransformBundle::from_transform(
                    // physx default plane is rotated compared to bevy plane, we undo that
                    Transform::from_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
                ))
                .insert(BPxActor::Static)
                .insert(BPxShape {
                    geometry: px_geometry,
                    material: px_material,
                    ..default()
                });
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

    let mesh = meshes.add(Mesh::from(shape::Cube { size: WIDTH }));
    let material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());

    let px_geometry = px_geometries.add(bpx::Geometry::cuboid(WIDTH, WIDTH, WIDTH));

    for i in 0..5 {
        commands.spawn(SpatialBundle::from_transform(Transform::from_xyz(0., 0., -1.25 * i as f32)))
            .with_children(|builder| {
                for i in 0..SIZE {
                    for j in 0..SIZE-i {
                        let transform = Transform::from_translation(Vec3::new(
                            ((j * 2) as f32 - (SIZE - i) as f32) / 2. * WIDTH,
                            (i * 2 + 1) as f32 / 2. * WIDTH,
                            0.,
                        ));

                        builder.spawn_empty()
                            .insert(PbrBundle {
                                mesh: mesh.clone(),
                                material: material.clone(),
                                transform,
                                ..default()
                            })
                            .insert(BPxActor::Dynamic)
                            .insert(BPxMassProperties::density(10.))
                            .insert(BPxShape {
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

    let mesh = meshes.add(Mesh::from(shape::UVSphere { radius: 1.25, ..default() }));
    let material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());

    let px_geometry = px_geometries.add(bpx::Geometry::ball(RADIUS));
    let px_material = px_materials.add(bpx::Material::new(&mut physics, 0.5, 0.5, 0.6));

    let transform = Transform::from_translation(Vec3::new(0., 5., 12.5));

    commands.spawn_empty()
        .insert(PbrBundle {
            mesh,
            material,
            transform,
            ..default()
        })
        .insert(BPxActor::Dynamic)
        .insert(BPxMassProperties::density(10.))
        .insert(BPxShape {
            material: px_material,
            geometry: px_geometry,
            ..default()
        })
        .insert(BPxVelocity::linear(Vec3::new(0., -6.25, -12.5)))
        .insert(Name::new("Ball"));
}
