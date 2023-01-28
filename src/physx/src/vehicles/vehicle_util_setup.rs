use crate::math::{PxTransform, PxVec3};
use crate::traits::Class;

use physx_sys::{
    PxVehicleCopyDynamicsMap,
    phys_PxVehicle4WEnable3WDeltaMode,
    phys_PxVehicle4WEnable3WTadpoleMode,
    phys_PxVehicleComputeSprungMasses,
    phys_PxVehicleCopyDynamicsData,
    phys_PxVehicleUpdateCMassLocalPose,
};

use super::{
    VehicleDriveSimData4W,
    VehicleWheels,
    VehicleWheelsDynData,
    VehicleWheelsSimData,
};

/// Reconfigure a PxVehicle4W instance as a three-wheeled car with tadpole config (2 front wheels, 1 rear wheel).
pub fn vehicle_4w_enable_3w_tadpole_mode(
    wheel_sim_data: &mut VehicleWheelsSimData,
    wheel_dyn_data: &mut VehicleWheelsDynData,
    drive_sim_data: &mut impl VehicleDriveSimData4W,
) {
    unsafe {
        phys_PxVehicle4WEnable3WTadpoleMode(wheel_sim_data.as_mut_ptr(), wheel_dyn_data.as_mut_ptr(), drive_sim_data.as_mut_ptr());
    }
}

/// Reconfigure a PxVehicle4W instance as a three-wheeled car with delta config (1 front wheel, 2 rear wheels).
pub fn vehicle_4w_enable_3w_delta_mode(
    wheel_sim_data: &mut VehicleWheelsSimData,
    wheel_dyn_data: &mut VehicleWheelsDynData,
    drive_sim_data: &mut impl VehicleDriveSimData4W,
) {
    unsafe {
        phys_PxVehicle4WEnable3WDeltaMode(wheel_sim_data.as_mut_ptr(), wheel_dyn_data.as_mut_ptr(), drive_sim_data.as_mut_ptr());
    }
}

/// Compute the sprung masses of the suspension springs given (i) the number of sprung masses, (ii) coordinates of the sprung masses, (iii) the center of mass offset of the rigid body, (iv) the total mass of the rigid body, and (v) the direction of gravity (0 for x-axis, 1 for y-axis, 2 for z-axis).
pub fn vehicle_compute_sprung_masses(
    sprung_mass_coordinates: &[PxVec3],
    centre_of_mass: PxVec3,
    total_mass: f32,
    gravity_direction: VehicleUtilGravityDirection,
) -> Vec<f32> {
    let mut sprung_masses = vec![0f32; sprung_mass_coordinates.len()];

    unsafe {
        phys_PxVehicleComputeSprungMasses(
            sprung_mass_coordinates.len() as u32,
            // SAFETY: PxVec3 is repr(transparent) of physx_sys::PxVec3,
            // so &[PxVec3] can be transmuted into &[physx_sys::PxVec3]
            std::mem::transmute(sprung_mass_coordinates.as_ptr()),
            centre_of_mass.as_ptr(),
            total_mass,
            gravity_direction.into(),
            sprung_masses.as_mut_ptr(),
        );
    }

    sprung_masses
}

/// Reconfigure the vehicle to reflect a new center of mass local pose that has been applied to the actor. The function requires (i) the center of mass local pose that was last used to configure the vehicle and the vehicle's actor, (ii) the new center of mass local pose that has been applied to the vehicle's actor and will now be applied to the vehicle, and (iii) the direction of gravity (0 for x-axis, 1 for y-axis, 2 for z-axis).
pub fn vehicle_update_cmass_local_pose(
    old_cmass_local_pose: &PxTransform,
    new_cmass_local_pose: &PxTransform,
    gravity_direction: VehicleUtilGravityDirection,
    vehicle: &mut impl VehicleWheels,
) {
    unsafe {
        phys_PxVehicleUpdateCMassLocalPose(
            old_cmass_local_pose.as_ptr(),
            new_cmass_local_pose.as_ptr(),
            gravity_direction.into(),
            vehicle.as_mut_ptr(),
        );
    }
}

/// Copy dynamics data from src to trg, including wheel rotation speed, wheel rotation angle, engine rotation speed etc.
pub fn vehicle_copy_dynamics_data(wheel_map: &PxVehicleCopyDynamicsMap, src: &impl VehicleWheels, trg: &mut impl VehicleWheels) {
    unsafe {
        phys_PxVehicleCopyDynamicsData(wheel_map, src.as_ptr(), trg.as_mut_ptr());
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum VehicleUtilGravityDirection {
    X = 0,
    Y = 1,
    Z = 2,
}

impl From<VehicleUtilGravityDirection> for u32 {
    fn from(value: VehicleUtilGravityDirection) -> Self {
        value as u32
    }
}
