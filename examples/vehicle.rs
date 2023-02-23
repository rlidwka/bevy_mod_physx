mod flying_camera;
use flying_camera::*;

use bevy::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGrid};
use bevy_physx::prelude::*;
use bevy_physx::prelude as bpx;
use bevy_physx::components::FilterData;
use physx::prelude::*;
use physx::vehicles::*;
use physx_sys::{PxVehicleDriveTankRawInputData};

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

pub const WHEEL_MASS: f32 = 30.;
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
struct PlayerControlledNoDrive;

#[derive(Component, Default)]
struct PlayerControlledDriveTank {
    initialized: bool,
    smoothing: Option<VehicleKeySmoothingData>,
    input: Option<Owner<PxVehicleDriveTankRawInputData>>,
}

#[derive(Component, Default)]
struct PlayerControlledDriveNW {
    initialized: bool,
    steer_table: Option<VehicleSteerVsForwardSpeedTable>,
    smoothing: Option<VehicleKeySmoothingData>,
    input: Option<Owner<PxVehicleDriveNWRawInputData>>,
}

#[derive(Component, Default)]
struct PlayerControlledDrive4W {
    initialized: bool,
    steer_table: Option<VehicleSteerVsForwardSpeedTable>,
    smoothing: Option<VehicleKeySmoothingData>,
    input: Option<Owner<PxVehicleDrive4WRawInputData>>,
}

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
            scene: bpx::SceneDescriptor {
                gravity: GRAVITY_FORCE,
                ..default()
            },
            ..default()
        })
        .add_plugin(FlyingCameraPlugin)
        .add_startup_system(spawn_light)
        .add_startup_system(spawn_plane)
        .add_startup_system(spawn_vehicle)
        .add_system(apply_vehicle_nodrive_controls)
        .add_system(apply_vehicle_tank_controls)
        .add_system(apply_vehicle_drive_nw_controls)
        .add_system(apply_vehicle_drive_4w_controls)
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
    mut physics: ResMut<bpx::Physics>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
    mut px_materials: ResMut<Assets<bpx::Material>>,
) {
    let px_geometry = px_geometries.add(bpx::Geometry::halfspace());
    let px_material = px_materials.add(bpx::Material::new(&mut physics, 0.5, 0.5, 0.6));

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
            .insert(bpx::RigidBody::Static)
            .insert(bpx::Shape {
                geometry: px_geometry,
                material: px_material,
                query_filter_data: FilterData::new(0, 0, 0, DRIVABLE_SURFACE),
                simulation_filter_data: FilterData::new(COLLISION_FLAG_GROUND, COLLISION_FLAG_GROUND_AGAINST, 0, 0),
            });
    })
    .insert(Name::new("Plane"));
}

fn create_wheels_sim_data() -> Owner<VehicleWheelsSimData> {
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
            max_steer: if idx < 2 { 0.5 } else { 0. },
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

#[allow(unused)]
fn create_drive_nw_sim_data() -> Box<PxVehicleDriveSimDataNW> {
    let mut diff = VehicleDifferentialNWData::default();
    diff.set_driven_wheel(0, true);
    diff.set_driven_wheel(1, true);
    diff.set_driven_wheel(2, true);
    diff.set_driven_wheel(3, true);

    let mut drive_sim_data = PxVehicleDriveSimDataNW::default();
    drive_sim_data.set_diff_data(diff);
    drive_sim_data.set_engine_data(VehicleEngineData {
        peak_torque: 500.,
        max_omega: 600.,
        ..default()
    });
    drive_sim_data.set_gears_data(VehicleGearsData {
        switch_time: 0.1,
        ..default()
    });

    Box::new(drive_sim_data)
}

#[allow(unused)]
fn create_drive_4w_sim_data() -> Box<PxVehicleDriveSimData4W> {
    let mut drive_sim_data = PxVehicleDriveSimData4W::default();

    drive_sim_data.set_diff_data(VehicleDifferential4WData {
        diff_type: VehicleDifferential4WType::OpenRearWD,
        ..default()
    });
    drive_sim_data.set_engine_data(VehicleEngineData {
        peak_torque: 500.,
        max_omega: 600.,
        ..default()
    });
    drive_sim_data.set_gears_data(VehicleGearsData {
        switch_time: 0.1,
        ..default()
    });
    drive_sim_data.set_clutch_data(VehicleClutchData {
        strength: 10.,
        ..default()
    });
    drive_sim_data.set_ackermann_geometry_data(VehicleAckermannGeometryData {
        front_width: (WHEEL_OFFSETS[1] - WHEEL_OFFSETS[0]).x.abs(),
        rear_width: (WHEEL_OFFSETS[3] - WHEEL_OFFSETS[2]).x.abs(),
        axle_separation: (WHEEL_OFFSETS[2] - WHEEL_OFFSETS[0]).z.abs(),
        ..default()
    });

    Box::new(drive_sim_data)
}

fn spawn_vehicle(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut physics: ResMut<bpx::Physics>,
    cooking: Res<Cooking>,
    mut simulation: ResMut<VehicleSimulation>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
    mut px_materials: ResMut<Assets<bpx::Material>>,
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
        bpx::Geometry::convex_mesh(&mut physics, &cooking, &HULL_VERTICES).unwrap()
    );
    let wheel_geometry = px_geometries.add(
        bpx::Geometry::cylinder(&mut physics, &cooking, WHEEL_HALF_WIDTH, WHEEL_RADIUS, WHEEL_SEGMENTS).unwrap()
    );
    let material = px_materials.add(bpx::Material::new(&mut physics, 0.5, 0.5, 0.6));

    let mut friction_pairs = VehicleDrivableSurfaceToTireFrictionPairs::new(
        1, 1, &[ &***(px_materials.get(&material).unwrap()) ], &[ VehicleDrivableSurfaceType(0) ]
    ).unwrap();
    friction_pairs.set_type_pair_friction(0, 0, 1000.);
    simulation.set_friction_pairs(friction_pairs);
    //simulation.set_collision_method(VehicleSimulationMethod::Raycast);

    let mut wheels = vec![];

    for wheel_idx in 0..WHEEL_COUNT as u32 {
        wheels.push(
            commands.spawn_empty()
                .insert(SpatialBundle::from_transform(Transform::from_translation(WHEEL_OFFSETS[wheel_idx as usize])))
                .insert(bpx::Shape {
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

    commands.spawn_empty()
        .insert(SceneBundle {
            scene: assets.load("cybertruck/hull.glb#Scene0"),
            ..default()
        })
        .insert(bpx::RigidBody::Dynamic)
        .insert(MassProperties::mass_with_center(HULL_MASS, CENTER_OF_MASS))

        ////////////////////////////////////////////////////////
        // uncomment this to test vehicle without a drive
        /*.insert(Vehicle::NoDrive {
            wheels: wheels.clone(),
            wheels_sim_data: create_wheels_sim_data(),
        })
        .insert(PlayerControlledNoDrive)*/
        ////////////////////////////////////////////////////////

        ////////////////////////////////////////////////////////
        // uncomment this to test a tank
        /*.insert(Vehicle::DriveTank {
            wheels: wheels.clone(),
            wheels_sim_data: create_wheels_sim_data(),
            drive_sim_data: Box::default(),
        })
        .insert(PlayerControlledDriveTank::default())*/
        ////////////////////////////////////////////////////////

        ////////////////////////////////////////////////////////
        // uncomment this to test vehicle with N wheels
        /*.insert(Vehicle::DriveNW {
            wheels: wheels.clone(),
            wheels_sim_data: create_wheels_sim_data(),
            drive_sim_data: create_drive_nw_sim_data(),
        })
        .insert(PlayerControlledDriveNW::default())*/
        ////////////////////////////////////////////////////////

        ////////////////////////////////////////////////////////
        // uncomment this to test vehicle with 4 wheels
        .insert(Vehicle::Drive4W {
            wheels: wheels.clone(),
            wheels_sim_data: create_wheels_sim_data(),
            drive_sim_data: create_drive_4w_sim_data(),
        })
        .insert(PlayerControlledDrive4W::default())
        ////////////////////////////////////////////////////////

        .insert(bpx::Shape {
            material,
            geometry: hull_geometry,
            ..default()
        })
        .insert(Name::new("Vehicle"))
        .insert_children(0, &wheels)
        .add_child(camera);
}

fn apply_vehicle_nodrive_controls(
    mut player_query: Query<&mut VehicleHandle, With<PlayerControlledNoDrive>>,
    keys: Res<Input<KeyCode>>,
) {
    let Ok(mut vehicle) = player_query.get_single_mut() else { return; };
    let VehicleHandle::NoDrive(vehicle) = vehicle.as_mut() else { return; };

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
    mut player_query: Query<(&mut VehicleHandle, &mut PlayerControlledDriveTank)>,
    simtime: Res<SimTime>,
    keys: Res<Input<KeyCode>>,
) {
    let Ok((mut vehicle, mut controls)) = player_query.get_single_mut() else { return; };
    let VehicleHandle::DriveTank(vehicle) = vehicle.as_mut() else { return; };

    if !controls.initialized {
        controls.initialized = true;
        vehicle.drive_dyn_data_mut().set_current_gear(VehicleGearsRatio::First);
        vehicle.drive_dyn_data_mut().set_use_auto_gears(true);

        let mut smoothing = VehicleKeySmoothingData::new();
        smoothing.set_rise_rates(&[6., 6., 6., 2.5, 2.5]);
        smoothing.set_fall_rates(&[10., 10., 10., 5., 5.]);
        controls.smoothing = Some(smoothing);

        let input: Owner<PxVehicleDriveTankRawInputData> = VehicleDriveTankRawInputData::new().unwrap();
        controls.input = Some(input);
    }

    for timestep in simtime.ticks() {
        let input = controls.input.as_mut().unwrap();

        // WARNING: tank requires different drive settings, and with default drive
        // controls are really wonky, need to adjust settings
        input.set_digital_accel(false);
        input.set_digital_left_brake(false);
        input.set_digital_left_thrust(false);
        input.set_digital_right_brake(false);
        input.set_digital_right_thrust(false);

        if keys.pressed(KeyCode::W) {
            input.set_digital_accel(true);
            input.set_digital_left_thrust(true);
            input.set_digital_right_thrust(true);
        }

        if keys.pressed(KeyCode::S) {
            input.set_digital_left_brake(true);
            input.set_digital_right_brake(true);
        }

        if keys.pressed(KeyCode::A) {
            input.set_digital_accel(true);
            input.set_digital_right_brake(true);
            input.set_digital_left_thrust(true);
        }

        if keys.pressed(KeyCode::D) {
            input.set_digital_accel(true);
            input.set_digital_left_brake(true);
            input.set_digital_right_thrust(true);
        }

        let smoothing = controls.smoothing.as_ref().unwrap();
        let input = controls.input.as_ref().unwrap();

        vehicle.smooth_digital_raw_inputs_and_set_analog_inputs(smoothing, input, timestep)
    }
}

fn apply_vehicle_drive_nw_controls(
    mut player_query: Query<(&mut VehicleHandle, &mut PlayerControlledDriveNW)>,
    simtime: Res<SimTime>,
    keys: Res<Input<KeyCode>>,
) {
    let Ok((mut vehicle, mut controls)) = player_query.get_single_mut() else { return; };
    let VehicleHandle::DriveNW(vehicle) = vehicle.as_mut() else { return; };

    if !controls.initialized {
        controls.initialized = true;
        vehicle.drive_dyn_data_mut().set_current_gear(VehicleGearsRatio::First);
        vehicle.drive_dyn_data_mut().set_use_auto_gears(true);

        let mut smoothing = VehicleKeySmoothingData::new();
        smoothing.set_rise_rates(&[6., 6., 6., 2.5, 2.5]);
        smoothing.set_fall_rates(&[10., 10., 10., 5., 5.]);
        controls.smoothing = Some(smoothing);

        let mut steer_table = VehicleSteerVsForwardSpeedTable::new();
        steer_table.set_data(&[
            (0., 0.75),
            (5., 0.75),
            (30., 0.125),
            (120., 0.1),
        ]);
        controls.steer_table = Some(steer_table);

        let input: Owner<PxVehicleDriveNWRawInputData> = VehicleDriveNWRawInputData::new().unwrap();
        controls.input = Some(input);
    }

    for timestep in simtime.ticks() {
        let input = controls.input.as_mut().unwrap();

        input.set_digital_accel(keys.pressed(KeyCode::W));
        input.set_digital_brake(keys.pressed(KeyCode::S));
        input.set_digital_steer_left(keys.pressed(KeyCode::A));
        input.set_digital_steer_right(keys.pressed(KeyCode::D));

        let smoothing = controls.smoothing.as_ref().unwrap();
        let input = controls.input.as_ref().unwrap();
        let steer_table = controls.steer_table.as_ref().unwrap();

        vehicle.smooth_digital_raw_inputs_and_set_analog_inputs(steer_table, smoothing, input, timestep, false);
    }
}

fn apply_vehicle_drive_4w_controls(
    mut player_query: Query<(&mut VehicleHandle, &mut PlayerControlledDrive4W)>,
    simtime: Res<SimTime>,
    keys: Res<Input<KeyCode>>,
) {
    let Ok((mut vehicle, mut controls)) = player_query.get_single_mut() else { return; };
    let VehicleHandle::Drive4W(vehicle) = vehicle.as_mut() else { return; };

    if !controls.initialized {
        controls.initialized = true;
        vehicle.drive_dyn_data_mut().set_current_gear(VehicleGearsRatio::First);
        vehicle.drive_dyn_data_mut().set_use_auto_gears(true);

        let mut smoothing = VehicleKeySmoothingData::new();
        smoothing.set_rise_rates(&[6., 6., 6., 2.5, 2.5]);
        smoothing.set_fall_rates(&[10., 10., 10., 5., 5.]);
        controls.smoothing = Some(smoothing);

        let mut steer_table = VehicleSteerVsForwardSpeedTable::new();
        steer_table.set_data(&[
            (0., 0.75),
            (5., 0.75),
            (30., 0.125),
            (120., 0.1),
        ]);
        controls.steer_table = Some(steer_table);

        let input: Owner<PxVehicleDrive4WRawInputData> = VehicleDrive4WRawInputData::new().unwrap();
        controls.input = Some(input);
    }

    for timestep in simtime.ticks() {
        let input = controls.input.as_mut().unwrap();

        input.set_digital_accel(keys.pressed(KeyCode::W));
        input.set_digital_brake(keys.pressed(KeyCode::S));
        input.set_digital_steer_right(keys.pressed(KeyCode::A));
        input.set_digital_steer_left(keys.pressed(KeyCode::D));

        let smoothing = controls.smoothing.as_ref().unwrap();
        let input = controls.input.as_ref().unwrap();
        let steer_table = controls.steer_table.as_ref().unwrap();

        vehicle.smooth_digital_raw_inputs_and_set_analog_inputs(steer_table, smoothing, input, timestep, false);
    }
}
