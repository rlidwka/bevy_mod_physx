mod flying_camera;
use flying_camera::*;

use bevy::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGrid};
use bevy_physx::prelude::*;
use bevy_physx::assets::{BPxMaterial, BPxGeometry};
use bevy_physx::components::{BPxActor, BPxShape, BPxMassProperties, BPxFilterData, BPxVehicleNoDrive, BPxVehicleDriveTank};
use bevy_physx::resources::{BPxPhysics, BPxCooking, BPxVehicleFrictionPairs};
use bevy_physx::vehicles::{VehicleNoDrive, VehicleWheelsSimData, vehicle_compute_sprung_masses, VehicleUtilGravityDirection, VehicleWheelData, VehicleTireData, VehicleSuspensionData, VehicleDriveSimData, Owner, PxVehicleDriveSimData, VehicleDriveDynData, VehicleGearsRatio};
use physx_sys::PxVehicleDrivableSurfaceType;

const DRIVABLE_SURFACE: u32 = 0xffff0000;
//const UNDRIVABLE_SURFACE: u32 = 0x0000ffff;

const COLLISION_FLAG_GROUND: u32 = 1 << 0;
//const COLLISION_FLAG_WHEEL: u32 = 1 << 1;
const COLLISION_FLAG_CHASSIS: u32 = 1 << 2;
const COLLISION_FLAG_OBSTACLE: u32 = 1 << 3;
const COLLISION_FLAG_DRIVABLE_OBSTACLE: u32 = 1 << 4;

const COLLISION_FLAG_GROUND_AGAINST: u32 = COLLISION_FLAG_CHASSIS | COLLISION_FLAG_OBSTACLE | COLLISION_FLAG_DRIVABLE_OBSTACLE;
//const COLLISION_FLAG_WHEEL_AGAINST: u32 = COLLISION_FLAG_WHEEL | COLLISION_FLAG_CHASSIS | COLLISION_FLAG_OBSTACLE;
//const COLLISION_FLAG_CHASSIS_AGAINST: u32 = COLLISION_FLAG_GROUND | COLLISION_FLAG_WHEEL | COLLISION_FLAG_CHASSIS | COLLISION_FLAG_OBSTACLE | COLLISION_FLAG_DRIVABLE_OBSTACLE;
//const COLLISION_FLAG_OBSTACLE_AGAINST: u32 = COLLISION_FLAG_GROUND | COLLISION_FLAG_WHEEL | COLLISION_FLAG_CHASSIS | COLLISION_FLAG_OBSTACLE | COLLISION_FLAG_DRIVABLE_OBSTACLE;
//const COLLISION_FLAG_DRIVABLE_OBSTACLE_AGAINST: u32 = COLLISION_FLAG_GROUND | COLLISION_FLAG_CHASSIS | COLLISION_FLAG_OBSTACLE | COLLISION_FLAG_DRIVABLE_OBSTACLE;

pub const GRAVITY_FORCE: Vec3 = Vec3::new(0., -9.81, 0.);
pub const HULL_MASS: f32 = 2800.;
pub const CENTER_OF_MASS: Vec3 = Vec3::new(0., 0.7, 0.);
pub const HULL_VERTICES : [Vec3; 18] = [
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

pub const WHEEL_MASS: f32 = 250.;
pub const WHEEL_HALF_WIDTH: f32 = 0.17;
pub const WHEEL_RADIUS: f32 = 0.49;
pub const WHEEL_SEGMENTS: usize = 24;
pub const WHEEL_COUNT: usize = 4;
pub const WHEEL_OFFSETS : [Vec3; WHEEL_COUNT] = [
    Vec3::new( 0.888138, 0.44912,  1.98057),
    Vec3::new(-0.888138, 0.44912,  1.98057),
    Vec3::new( 0.888138, 0.44912, -1.76053),
    Vec3::new(-0.888138, 0.44912, -1.76053),
];

#[derive(Component)]
struct PlayerControlled;

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
        .add_plugin(InfiniteGridPlugin)
        .add_plugin(PhysXPlugin {
            gravity: GRAVITY_FORCE,
            ..default()
        })
        .add_plugin(FlyingCameraPlugin)
        .add_startup_system(spawn_light)
        .add_startup_system(spawn_plane)
        .add_startup_system(spawn_vehicle)
        .add_system(apply_vehicle_nodrive_controls)
        .add_system(apply_vehicle_tank_controls)
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

fn spawn_plane(
    mut commands: Commands,
    mut physics: ResMut<BPxPhysics>,
    mut px_geometries: ResMut<Assets<BPxGeometry>>,
    mut px_materials: ResMut<Assets<BPxMaterial>>,
) {
    let px_geometry = px_geometries.add(BPxGeometry::halfspace());
    let px_material = px_materials.add(BPxMaterial::new(&mut physics, 0.5, 0.5, 0.6));

    commands.spawn(InfiniteGridBundle {
        grid: InfiniteGrid {
            fadeout_distance: 10000.,
            ..default()
        },
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
                query_filter_data: BPxFilterData::new(0, 0, 0, DRIVABLE_SURFACE),
                simulation_filter_data: BPxFilterData::new(COLLISION_FLAG_GROUND, COLLISION_FLAG_GROUND_AGAINST, 0, 0),
            });
    })
    .insert(Name::new("Plane"));
}

fn create_wheels_sim_data(wheels: [Entity; WHEEL_COUNT]) -> Owner<VehicleWheelsSimData> {
    let mut wheels_sim_data = VehicleWheelsSimData::new(WHEEL_COUNT as u32).unwrap();
    let cmass_offsets = WHEEL_OFFSETS.iter().map(|v| *v - CENTER_OF_MASS).collect::<Vec<_>>();

    let suspension_spring_masses = vehicle_compute_sprung_masses(
        &cmass_offsets.iter().map(|v| v.to_physx()).collect::<Vec<_>>(),
        CENTER_OF_MASS.to_physx(),
        HULL_MASS,
        VehicleUtilGravityDirection::Y,
    );

    for idx in 0..WHEEL_COUNT as u32 {
        wheels_sim_data.set_wheel_data(idx, VehicleWheelData {
            mass: WHEEL_MASS,
            radius: WHEEL_RADIUS,
            width: WHEEL_HALF_WIDTH * 2.,
            moi: 0.5 * WHEEL_MASS * WHEEL_RADIUS * WHEEL_RADIUS,
            ..default()
        });

        wheels_sim_data.set_tire_data(idx, VehicleTireData {
            tire_type: 0,
            ..default()
        });

        wheels_sim_data.set_suspension_data(idx, VehicleSuspensionData {
            max_compression: 0.01,
            max_droop: 0.03,
            spring_strength: 35000.,
            spring_damper_rate: 4500.,
            sprung_mass: suspension_spring_masses[idx as usize],
            ..default()
        });

        wheels_sim_data.set_susp_travel_direction(idx, Vec3::new(0., -1., 0.).to_physx());
        wheels_sim_data.set_wheel_centre_offset(idx, cmass_offsets[idx as usize].to_physx());
        wheels_sim_data.set_susp_force_app_point_offset(idx, (cmass_offsets[idx as usize] - Vec3::Y * 0.3).to_physx());
        wheels_sim_data.set_tire_force_app_point_offset(idx, (cmass_offsets[idx as usize] - Vec3::Y * 0.3).to_physx());
    }

    wheels_sim_data
}

fn spawn_vehicle(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut physics: ResMut<BPxPhysics>,
    cooking: Res<BPxCooking>,
    mut friction_pairs: ResMut<BPxVehicleFrictionPairs>,
    mut px_geometries: ResMut<Assets<BPxGeometry>>,
    mut px_materials: ResMut<Assets<BPxMaterial>>,
) {
    let camera = commands.spawn(FlyingCameraBundle {
        flying_camera: FlyingCamera {
            distance: 60.,
            ..default()
        },
        ..default()
    })
    .insert(Name::new("Camera"))
    .id();

    let hull_geometry = px_geometries.add(
        BPxGeometry::convex_mesh(&mut physics, &cooking, &HULL_VERTICES)
    );
    let wheel_geometry = px_geometries.add(
        BPxGeometry::cylinder(&mut physics, &cooking, WHEEL_HALF_WIDTH, WHEEL_RADIUS, WHEEL_SEGMENTS)
    );
    let material = px_materials.add(BPxMaterial::new(&mut physics, 0.5, 0.5, 0.6));

    friction_pairs.setup(&[ px_materials.get(&material).unwrap() ], &[ PxVehicleDrivableSurfaceType { mType: 0 } ]);
    friction_pairs.set_type_pair_friction(0, 0, 1000.);

    let mut wheels = vec![];

    for wheel_idx in 0..WHEEL_COUNT as u32 {
        wheels.push(
            commands.spawn_empty()
                .insert(SpatialBundle::from_transform(Transform::from_translation(WHEEL_OFFSETS[wheel_idx as usize])))
                .insert(BPxShape {
                    material: material.clone(),
                    geometry: wheel_geometry.clone(),
                    ..default()
                })
                .with_children(|builder| {
                    builder.spawn(SceneBundle {
                        scene: assets.load("cybertruck/wheel.glb#Scene0"),
                        transform: Transform::from_rotation(Quat::from_rotation_z(if wheel_idx % 2 == 1 {
                            std::f32::consts::PI
                        } else {
                            0.
                        })),
                        ..default()
                    });
                })
                .id()
        );
    }

    let wheels_sim_data = create_wheels_sim_data(wheels[..].try_into().unwrap());
    let drive_sim_data = PxVehicleDriveSimData::default();
    let vehicle = BPxVehicleDriveTank::new(wheels.to_vec(), wheels_sim_data, Box::new(drive_sim_data));

    commands.spawn_empty()
        .insert(SceneBundle {
            scene: assets.load("cybertruck/hull.glb#Scene0"),
            ..default()
        })
        .insert(PlayerControlled)
        .insert(BPxActor::Dynamic)
        .insert(BPxMassProperties::mass_with_center(HULL_MASS, CENTER_OF_MASS))
        .insert(vehicle)
        .insert(BPxShape {
            material,
            geometry: hull_geometry,
            ..default()
        })
        .insert(Name::new("Vehicle"))
        .insert_children(0, &wheels)
        .add_child(camera);
}

fn apply_vehicle_nodrive_controls(
    mut player_query: Query<&mut BPxVehicleNoDrive, With<PlayerControlled>>,
    keys: Res<Input<KeyCode>>,
) {
    let Ok(mut vehicle) = player_query.get_single_mut() else { return; };
    let Some(vehicle) = vehicle.vehicle_mut() else { return; };

    if keys.just_pressed(KeyCode::W) {
        vehicle.set_drive_torque(2, 4000.);
        vehicle.set_drive_torque(3, 4000.);
    }

    if keys.just_released(KeyCode::W) {
        vehicle.set_drive_torque(2, 0.);
        vehicle.set_drive_torque(3, 0.);
    }

    if keys.just_pressed(KeyCode::S) {
        vehicle.set_brake_torque(2, 15000.);
        vehicle.set_brake_torque(3, 15000.);
    }

    if keys.just_released(KeyCode::S) {
        vehicle.set_brake_torque(2, 0.);
        vehicle.set_brake_torque(3, 0.);
    }

    if keys.just_pressed(KeyCode::A) {
        vehicle.set_steer_angle(0, 0.5);
        vehicle.set_steer_angle(1, 0.5);
    }

    if keys.just_released(KeyCode::A) {
        vehicle.set_steer_angle(0, 0.);
        vehicle.set_steer_angle(1, 0.);
    }

    if keys.just_pressed(KeyCode::D) {
        vehicle.set_steer_angle(0, -0.5);
        vehicle.set_steer_angle(1, -0.5);
    }

    if keys.just_released(KeyCode::D) {
        vehicle.set_steer_angle(0, 0.);
        vehicle.set_steer_angle(1, 0.);
    }
}

fn apply_vehicle_tank_controls(
    mut player_query: Query<&mut BPxVehicleDriveTank, With<PlayerControlled>>,
    keys: Res<Input<KeyCode>>,
) {
    let Ok(mut vehicle) = player_query.get_single_mut() else { return; };
    let Some(vehicle) = vehicle.vehicle_mut() else { return; };

    if keys.just_pressed(KeyCode::W) {
        /*vehicle.drive_dyn_data_mut().set_use_auto_gears(true);

        if vehicle.drive_dyn_data_mut().get_current_gear() < VehicleGearsRatio::First {
            vehicle.drive_dyn_data_mut().set_current_gear(VehicleGearsRatio::First);
        }*/

        vehicle.drive_dyn_data_mut().set_gear_up(true);
        vehicle.drive_dyn_data_mut().set_engine_rotation_speed(1000000.);
    }
}
