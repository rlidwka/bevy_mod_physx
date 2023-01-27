use physx::{
    base::Base,
    traits::Class,
};

use physx_sys::{
    PxVehicleWheels_getVehicleType,
    //PxVehicleWheels_getRigidDynamicActor_mut,
    //PxVehicleWheels_getRigidDynamicActor,
    PxVehicleWheels_computeForwardSpeed,
    PxVehicleWheels_computeSidewaysSpeed,
    //PxVehicleWheels_requiresObjects_mut,
    //PxVehicleWheels_getConcreteTypeName,
    //PxVehicleWheels_isKindOf,
    //PxVehicleWheels_preExportDataReset_mut,
    //PxVehicleWheels_exportExtraData_mut,
    //PxVehicleWheels_importExtraData_mut,
    //PxVehicleWheels_resolveReferences_mut,
    //PxVehicleWheels_getBinaryMetaData_mut,
    PxVehicleWheels_getNbNonDrivenWheels,
    //PxVehicleWheels_new_alloc,
    //PxVehicleWheels_new_alloc_1,
    //PxVehicleWheels_release_mut,
};

impl<T> VehicleWheels for T where T: Class<physx_sys::PxVehicleWheels> + Base {}

pub trait VehicleWheels: Class<physx_sys::PxVehicleWheels> + Base {
    /// Return the type of vehicle.
    fn get_vehicle_type(&self) -> VehicleTypes {
        unsafe { PxVehicleWheels_getVehicleType(self.as_ptr()).into() }
    }

    /// Get PxRigidDynamic instance that is the vehicle's physx representation.
    /*fn get_rigid_dynamic_actor(&self) -> *const PxRigidDynamic {
        // TODO: not sure how to return proper rust object?
        unsafe { PxVehicleWheels_getRigidDynamicActor(self.as_ptr()) }
    }*/

    /// Get PxRigidDynamic instance that is the vehicle's physx representation.
    /*fn get_rigid_dynamic_actor_mut(&mut self) -> *mut PxRigidDynamic {
        // TODO: not sure how to return proper rust object?
        unsafe { PxVehicleWheels_getRigidDynamicActor_mut(self.as_mut_ptr()) }
    }*/

    /// Compute the rigid body velocity component along the forward vector of the rigid body transform.
    fn compute_forward_speed(&self) -> f32 {
        unsafe { PxVehicleWheels_computeForwardSpeed(self.as_ptr()) }
    }

    /// Compute the rigid body velocity component along the right vector of the rigid body transform.
    fn compute_sideways_speed(&self) -> f32 {
        unsafe { PxVehicleWheels_computeSidewaysSpeed(self.as_ptr()) }
    }

    fn get_nb_non_driven_wheels(&self) -> u32 {
        unsafe { PxVehicleWheels_getNbNonDrivenWheels(self.as_ptr()) }
    }
}

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
