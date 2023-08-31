use crate::{
    base::Base,
    traits::Class,
    rigid_dynamic::PxRigidDynamic,
    shape::Shape,
};

use physx_sys::{
    PxVehicleWheels_getVehicleType,
    PxVehicleWheels_getRigidDynamicActor_mut,
    PxVehicleWheels_getRigidDynamicActor,
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

use super::VehicleTypes;

impl<T> VehicleWheels for T where T: Class<physx_sys::PxVehicleWheels> + Base {}

pub trait VehicleWheels: Class<physx_sys::PxVehicleWheels> + Base {
    /// Return the type of vehicle.
    fn get_vehicle_type(&self) -> VehicleTypes {
        unsafe { PxVehicleWheels_getVehicleType(self.as_ptr()).into() }
    }

    /// Get PxRigidDynamic instance that is the vehicle's physx representation.
    ///
    /// SAFETY: RigidDynamic's user data type and shape must match the one you used to create vehicle
    unsafe fn get_rigid_dynamic_actor<D, G: Shape>(&self) -> &PxRigidDynamic<D, G> {
        // TODO: not sure how to make this safe, maybe return () as user data?
        std::mem::transmute(PxVehicleWheels_getRigidDynamicActor(self.as_ptr()))
    }

    /// Get PxRigidDynamic instance that is the vehicle's physx representation.
    ///
    /// SAFETY: RigidDynamic's user data type and shape must match the one you used to create vehicle
    unsafe fn get_rigid_dynamic_actor_mut<D, G: Shape>(&mut self) -> &mut PxRigidDynamic<D, G> {
        // TODO: not sure how to make this safe, maybe return () as user data?
        std::mem::transmute(PxVehicleWheels_getRigidDynamicActor_mut(self.as_mut_ptr()))
    }

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
