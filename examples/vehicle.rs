mod flying_camera;

use bevy::prelude::*;
use flying_camera::*;

use bevy_physx::BPxPlugin;
use bevy_physx::assets::{BPxMaterial, BPxGeometry};
use bevy_physx::components::{BPxActor, BPxShape, BPxVehicle, BPxVehicleWheel, BPxVehicleWheelData, BPxVehicleSuspensionData, BPxMassProperties};
use bevy_physx::resources::{BPxPhysics, BPxCooking, BPxVehicleFrictionPairs};
use physx_sys::PxVehicleDrivableSurfaceType;

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
        .add_plugin(BPxPlugin::default())
        .add_plugin(FlyingCameraPlugin)
        .add_startup_system(spawn_light)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_plane)
        .add_startup_system(spawn_vehicle)
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
    mut physics: ResMut<BPxPhysics>,
    mut px_geometries: ResMut<Assets<BPxGeometry>>,
    mut px_materials: ResMut<Assets<BPxMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 500.0 }));
    let material = materials.add(Color::rgb(0.3, 0.5, 0.3).into());
    let px_geometry = px_geometries.add(BPxGeometry::halfspace());
    let px_material = px_materials.add(BPxMaterial::new(&mut physics, 0.5, 0.5, 0.6));

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
                });
        })
        .insert(Name::new("Plane"));
}

fn spawn_vehicle(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut physics: ResMut<BPxPhysics>,
    mut cooking: ResMut<BPxCooking>,
    mut friction_pairs: ResMut<BPxVehicleFrictionPairs>,
    mut px_geometries: ResMut<Assets<BPxGeometry>>,
    mut px_materials: ResMut<Assets<BPxMaterial>>,
) {
    const HULL_VERTICES : [Vec3; 18] = [
        Vec3::new(-0.92657, 1.44990, -2.83907),
        Vec3::new( 0.92657, 1.44990, -2.83907),
        Vec3::new(-0.73964, 1.85914,  0.41819),
        Vec3::new( 0.73964, 1.85914,  0.41819),
        Vec3::new(-0.96609, 1.11038,  2.57245),
        Vec3::new( 0.96609, 1.11038,  2.57245),
        Vec3::new(-0.62205, 1.01440,  2.84896),
        Vec3::new( 0.62205, 1.01440,  2.84896),
        Vec3::new(-0.92108, 0.63051, -2.74199),
        Vec3::new( 0.92108, 0.63051, -2.74199),
        Vec3::new(-0.65192, 0.46546,  2.74609),
        Vec3::new( 0.65192, 0.46546,  2.74609),
        Vec3::new(-0.98115, 0.46546,  2.48097),
        Vec3::new( 0.98115, 0.46546,  2.48097),
        Vec3::new(-0.90621, 0.38511, -1.06282),
        Vec3::new( 0.90621, 0.38511, -1.06282),
        Vec3::new(-0.90621, 0.34191,  1.26607),
        Vec3::new( 0.90621, 0.34191,  1.26607),
    ];

    pub const WHEEL_MASS: f32 = 30.;
    pub const WHEEL_HALF_WIDTH: f32 = 0.17;
    pub const WHEEL_RADIUS: f32 = 0.49;
    pub const WHEEL_SEGMENTS: usize = 16;

    const WHEEL_OFFSETS : [Vec3; 4] = [
        Vec3::new( 0.888138, 0.44912,  1.98057),
        Vec3::new(-0.888138, 0.44912,  1.98057),
        Vec3::new( 0.888138, 0.44912, -1.76053),
        Vec3::new(-0.888138, 0.44912, -1.76053),
    ];

    let hull_geometry = px_geometries.add(
        BPxGeometry::convex_mesh(&mut physics, &mut cooking, &HULL_VERTICES)
    );
    let wheel_geometry = px_geometries.add(
        BPxGeometry::cylinder(&mut physics, &mut cooking, WHEEL_HALF_WIDTH, WHEEL_RADIUS, WHEEL_SEGMENTS)
    );
    let material = px_materials.add(BPxMaterial::new(&mut physics, 0.5, 0.5, 0.6));

    friction_pairs.setup(&[ px_materials.get(&material).unwrap() ], &[ PxVehicleDrivableSurfaceType { mType: 0 } ]);
    friction_pairs.set_type_pair_friction(0, 0, 1000.);

    commands.spawn_empty()
        .insert(SceneBundle {
            scene: assets.load("cybertruck/hull.glb#Scene0"),
            ..default()
        })
        .insert(BPxActor::Dynamic)
        .insert(BPxMassProperties::mass_with_center(2800., Vec3::new(0., 0.7, 0.)))
        .insert(BPxVehicle)
        .insert(BPxShape {
            material: material.clone(),
            geometry: hull_geometry.clone(),
        })
        .with_children(|builder| {
            for wheel_idx in 0..4 {
                builder.spawn_empty()
                    .insert(SceneBundle {
                        scene: assets.load("cybertruck/wheel.glb#Scene0"),
                        transform: Transform {
                            translation: WHEEL_OFFSETS[wheel_idx],
                            rotation: Quat::from_rotation_z(if wheel_idx % 2 == 1 {
                                std::f32::consts::PI
                            } else {
                                0.
                            }),
                            scale: Vec3::ONE,
                        },
                        ..default()
                    })
                    .insert(BPxVehicleWheel {
                        wheel_data: BPxVehicleWheelData {
                            mass: WHEEL_MASS,
                            radius: WHEEL_RADIUS,
                            width: WHEEL_HALF_WIDTH * 2.,
                            moi: 0.5 * WHEEL_MASS * WHEEL_RADIUS * WHEEL_RADIUS,
                            ..default()
                        },
                        suspension_data: BPxVehicleSuspensionData {
                            max_compression: 0.3,
                            max_droop: 0.1,
                            spring_strength: 35000.,
                            spring_damper_rate: 4500.,
                            ..default()
                        },
                        susp_force_app_point_offset: WHEEL_OFFSETS[wheel_idx] - Vec3::Y * 0.3,
                        tire_force_app_point_offset: WHEEL_OFFSETS[wheel_idx] - Vec3::Y * 0.3,
                        ..default()
                    })
                    .insert(BPxShape {
                        material: material.clone(),
                        geometry: wheel_geometry.clone(),
                    });
            }
        })
        .insert(Name::new("Vehicle"));
}
