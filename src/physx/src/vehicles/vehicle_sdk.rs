use physx_sys::{phys_PxVehicleSetBasisVectors, phys_PxVehicleSetUpdateMode, phys_PxVehicleSetSweepHitRejectionAngles, phys_PxVehicleSetMaxHitActorAcceleration};

use crate::math::PxVec3;
use crate::traits::Class;

/// This number is the maximum number of wheels allowed for a vehicle.
pub const PX_MAX_NB_WHEELS: usize = 20; // maybe autogenerate this in physx-sys?

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum VehicleTypes {
    Drive4W = 0,
    DriveNW = 1,
    DriveTank = 2,
    NoDrive = 3,
    User1 = 4,
    User2 = 5,
    User3 = 6,
    //MaxNBVehicleTypes = 7,
}

impl VehicleTypes {
    pub const MAX_NB_VEHICLE_TYPES: u32 = 7;
}

impl From<VehicleTypes> for physx_sys::PxVehicleTypes::Enum {
    fn from(value: VehicleTypes) -> Self {
        value as u32
    }
}

impl From<physx_sys::PxVehicleTypes::Enum> for VehicleTypes {
    fn from(ty: physx_sys::PxVehicleTypes::Enum) -> Self {
        match ty {
            0 => Self::Drive4W,
            1 => Self::DriveNW,
            2 => Self::DriveTank,
            3 => Self::NoDrive,
            4 => Self::User1,
            5 => Self::User2,
            6 => Self::User3,
            _ => panic!("invalid enum variant"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum VehicleUpdateMode {
    VelocityChange = 0,
    Acceleration = 1,
}

impl From<VehicleUpdateMode> for physx_sys::PxVehicleUpdateMode::Enum {
    fn from(value: VehicleUpdateMode) -> Self {
        value as u32
    }
}

impl From<physx_sys::PxVehicleUpdateMode::Enum> for VehicleUpdateMode {
    fn from(ty: physx_sys::PxVehicleUpdateMode::Enum) -> Self {
        match ty {
            0 => Self::VelocityChange,
            1 => Self::Acceleration,
            _ => panic!("invalid enum variant"),
        }
    }
}

/// Set the basis vectors of the vehicle simulation.
pub fn vehicle_set_basis_vectors(up: PxVec3, forward: PxVec3) {
    unsafe {
        phys_PxVehicleSetBasisVectors(up.as_ptr(), forward.as_ptr());
    }
}

/// Set the effect of PxVehicleUpdates to be either to modify each vehicle's rigid body actor.
pub fn vehicle_set_update_mode(vehicle_update_mode: VehicleUpdateMode) {
    unsafe {
        phys_PxVehicleSetUpdateMode(vehicle_update_mode.into());
    }
}

/// Set threshold angles that are used to determine if a wheel hit is to be resolved by vehicle suspension or by rigid body collision.
pub fn vehicle_set_sweep_hit_rejection_angles(point_reject_angle: f32, normal_reject_angle: f32) {
    unsafe {
        phys_PxVehicleSetSweepHitRejectionAngles(point_reject_angle, normal_reject_angle);
    }
}

/// Determine the maximum acceleration experienced by PxRigidDynamic instances that are found to be in contact with a wheel.
pub fn vehicle_set_max_hit_actor_acceleration(max_hit_actor_acceleration: f32) {
    unsafe {
        phys_PxVehicleSetMaxHitActorAcceleration(max_hit_actor_acceleration);
    }
}
