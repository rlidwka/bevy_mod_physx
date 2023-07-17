use std::collections::HashMap;

use bevy::prelude::*;
use physx::prelude::*;
use physx::vehicles::*;

use crate::core::scene::SceneRwLock;
use crate::prelude as bpx;
use crate::types::PxRigidDynamic;

#[derive(Component)]
pub enum Vehicle {
    NoDrive {
        wheels: Vec<Entity>,
        wheels_sim_data: Owner<VehicleWheelsSimData>,
    },
    Drive4W {
        wheels: Vec<Entity>,
        wheels_sim_data: Owner<VehicleWheelsSimData>,
        drive_sim_data: Box<PxVehicleDriveSimData4W>,
    },
    DriveNW {
        wheels: Vec<Entity>,
        wheels_sim_data: Owner<VehicleWheelsSimData>,
        drive_sim_data: Box<PxVehicleDriveSimDataNW>,
    },
    DriveTank {
        wheels: Vec<Entity>,
        wheels_sim_data: Owner<VehicleWheelsSimData>,
        drive_sim_data: Box<PxVehicleDriveSimData>,
    },
}

#[derive(Component)]
pub enum VehicleHandle {
    NoDrive(SceneRwLock<Owner<PxVehicleNoDrive>>),
    Drive4W(SceneRwLock<Owner<PxVehicleDrive4W>>),
    DriveNW(SceneRwLock<Owner<PxVehicleDriveNW>>),
    DriveTank(SceneRwLock<Owner<PxVehicleDriveTank>>),
}

impl VehicleHandle {
    pub fn new(vehicle_desc: &mut Vehicle, physics: &mut bpx::Physics, actor: &mut PxRigidDynamic) -> Self {
        let (wheels, wheels_sim_data) = match vehicle_desc {
            Vehicle::NoDrive { wheels, wheels_sim_data } => (wheels, wheels_sim_data),
            Vehicle::Drive4W { wheels, wheels_sim_data, .. } => (wheels, wheels_sim_data),
            Vehicle::DriveNW { wheels, wheels_sim_data, .. } => (wheels, wheels_sim_data),
            Vehicle::DriveTank { wheels, wheels_sim_data, .. } => (wheels, wheels_sim_data),
        };

        let mut shape_mapping = HashMap::new();
        for (idx, shape) in actor.get_shapes().into_iter().enumerate() {
            shape_mapping.insert(*shape.get_user_data(), idx as i32);
        }

        for (wheel_id, entity) in wheels.iter().enumerate() {
            wheels_sim_data.set_wheel_shape_mapping(
                wheel_id as u32,
                *shape_mapping.get(entity)
                    .expect("Wheel entity is not a valid shape. Vehicle must be a dynamic actor with all shapes as its direct descendants."),
            );
        }

        match vehicle_desc {
            Vehicle::NoDrive { wheels: _, wheels_sim_data } => {
                Self::NoDrive(
                    SceneRwLock::new(VehicleNoDrive::new(physics.physics_mut(), actor, wheels_sim_data).unwrap())
                )
            }
            Vehicle::Drive4W { wheels, wheels_sim_data, drive_sim_data } => {
                Self::Drive4W(
                    SceneRwLock::new(VehicleDrive4W::new(physics.physics_mut(), actor, wheels_sim_data, drive_sim_data.as_ref(), wheels.len() as u32 - 4).unwrap())
                )
            }
            Vehicle::DriveNW { wheels, wheels_sim_data, drive_sim_data } => {
                Self::DriveNW(
                    SceneRwLock::new(VehicleDriveNW::new(physics.physics_mut(), actor, wheels_sim_data, drive_sim_data.as_ref(), wheels.len() as u32).unwrap())
                )
            }
            Vehicle::DriveTank { wheels, wheels_sim_data, drive_sim_data } => {
                Self::DriveTank(
                    SceneRwLock::new(VehicleDriveTank::new(physics.physics_mut(), actor, wheels_sim_data, drive_sim_data.as_ref(), wheels.len() as u32).unwrap())
                )
            }
        }
    }
}
