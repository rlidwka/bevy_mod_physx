use crate::{
    DeriveClassForNewType,
    owner::Owner,
    traits::Class,
    physics::Physics,
    rigid_dynamic::RigidDynamic,
};

use physx_sys::{
    PxVehicleNoDrive_allocate_mut,
    PxVehicleNoDrive_free_mut,
    PxVehicleNoDrive_setup_mut,
    PxVehicleNoDrive_create_mut,
    PxVehicleNoDrive_setToRestState_mut,
    PxVehicleNoDrive_setBrakeTorque_mut,
    PxVehicleNoDrive_setDriveTorque_mut,
    PxVehicleNoDrive_setSteerAngle_mut,
    PxVehicleNoDrive_getBrakeTorque,
    PxVehicleNoDrive_getDriveTorque,
    PxVehicleNoDrive_getSteerAngle,
    //PxVehicleNoDrive_new_alloc,
    //PxVehicleNoDrive_exportExtraData_mut,
    //PxVehicleNoDrive_importExtraData_mut,
    //PxVehicleNoDrive_createObject_mut,
    //PxVehicleNoDrive_getBinaryMetaData_mut,
    //PxVehicleNoDrive_getConcreteTypeName,
    //PxVehicleNoDrive_isKindOf,
    PxVehicleNoDrive_getNbSteerAngle,
    PxVehicleNoDrive_getNbDriveTorque,
    PxVehicleNoDrive_getNbBrakeTorque,
};

use super::{
    VehicleWheels,
    VehicleWheelsDynData,
    VehicleWheelsSimData,
};

#[repr(transparent)]
#[derive(Clone)]
pub struct PxVehicleNoDrive {
    obj: physx_sys::PxVehicleNoDrive,
}

unsafe impl Send for PxVehicleNoDrive {}
unsafe impl Sync for PxVehicleNoDrive {}

impl PxVehicleNoDrive {
    /// Data describing the setup of all the wheels/suspensions/tires.
    pub fn wheels_sim_data(&self) -> &VehicleWheelsSimData {
        // SAFETY: VehicleWheelsSimData is repr(transparent)
        unsafe { std::mem::transmute(&self.obj.mWheelsSimData) }
    }

    /// Data describing the setup of all the wheels/suspensions/tires.
    pub fn wheels_sim_data_mut(&mut self) -> &mut VehicleWheelsSimData {
        // SAFETY: VehicleWheelsSimData is repr(transparent)
        unsafe { std::mem::transmute(&mut self.obj.mWheelsSimData) }
    }

    /// Data describing the dynamic state of all wheels/suspension/tires.
    pub fn wheels_dyn_data(&self) -> &VehicleWheelsDynData {
        // SAFETY: VehicleWheelsDynData is repr(transparent)
        unsafe { std::mem::transmute(&self.obj.mWheelsDynData) }
    }

    /// Data describing the dynamic state of all wheels/suspension/tires.
    pub fn wheels_dyn_data_mut(&mut self) -> &mut VehicleWheelsDynData {
        // SAFETY: VehicleWheelsDynData is repr(transparent)
        unsafe { std::mem::transmute(&mut self.obj.mWheelsDynData) }
    }
}

impl Drop for PxVehicleNoDrive {
    fn drop(&mut self) {
        unsafe { PxVehicleNoDrive_free_mut(self.as_mut_ptr()) }
    }
}

DeriveClassForNewType!(PxVehicleNoDrive: PxVehicleNoDrive, PxVehicleWheels, PxBase);

impl<T> VehicleNoDrive for T where T: Class<physx_sys::PxVehicleNoDrive> + VehicleWheels {}

pub trait VehicleNoDrive: Class<physx_sys::PxVehicleNoDrive> + VehicleWheels {
    /// Allocate and set up a vehicle using simulation data for the wheels.
    fn new(
        physics: &mut impl Physics,
        veh_actor: &mut impl RigidDynamic,
        wheels_data: &VehicleWheelsSimData,
    ) -> Option<Owner<Self>> {
        unsafe {
            VehicleNoDrive::from_raw(
                PxVehicleNoDrive_create_mut(
                    physics.as_mut_ptr(),
                    veh_actor.as_mut_ptr(),
                    wheels_data.as_ptr(),
                )
            )
        }
    }

    /// Allocate a PxVehicleNoDrive instance for a vehicle without drive model and with nbWheels.
    fn allocate(nb_wheels: u32) -> Option<Owner<Self>> {
        unsafe {
            VehicleNoDrive::from_raw(PxVehicleNoDrive_allocate_mut(nb_wheels))
        }
    }

    /// Set up a vehicle using simulation data for the wheels.
    fn setup(
        &mut self,
        physics: &mut impl Physics,
        veh_actor: &mut impl RigidDynamic,
        wheels_data: &VehicleWheelsSimData,
    ) {
        unsafe { PxVehicleNoDrive_setup_mut(self.as_mut_ptr(), physics.as_mut_ptr(), veh_actor.as_mut_ptr(), wheels_data.as_ptr()) }
    }

    /// Create a new Owner wrapper around a raw pointer.
    /// # Safety
    /// Owner's own the pointer they wrap, using the pointer after dropping the Owner,
    /// or creating multiple Owners from the same pointer will cause UB.  Use `into_ptr` to
    /// retrieve the pointer and consume the Owner without dropping the pointee.
    /// Initializes user data.
    unsafe fn from_raw(
        ptr: *mut physx_sys::PxVehicleNoDrive,
    ) -> Option<Owner<Self>> {
        Owner::from_raw(ptr as *mut Self)
    }

    /// Set a vehicle to its rest state. Aside from the rigid body transform, this will set the vehicle and rigid body to the state they were in immediately after setup or create.
    fn set_to_rest_state(&mut self) {
        unsafe { PxVehicleNoDrive_setToRestState_mut(self.as_mut_ptr()) }
    }

    /// Set the brake torque to be applied to a specific wheel.
    fn set_brake_torque(&mut self, id: u32, brake_torque: f32) {
        unsafe { PxVehicleNoDrive_setBrakeTorque_mut(self.as_mut_ptr(), id, brake_torque) }
    }

    /// Set the drive torque to be applied to a specific wheel.
    fn set_drive_torque(&mut self, id: u32, drive_torque: f32) {
        unsafe { PxVehicleNoDrive_setDriveTorque_mut(self.as_mut_ptr(), id, drive_torque) }
    }

    /// Set the steer angle to be applied to a specific wheel.
    fn set_steer_angle(&mut self, id: u32, steer_angle: f32) {
        unsafe { PxVehicleNoDrive_setSteerAngle_mut(self.as_mut_ptr(), id, steer_angle) }
    }

    /// Get the brake torque that has been applied to a specific wheel.
    fn get_brake_torque(&self, id: u32) -> f32 {
        unsafe { PxVehicleNoDrive_getBrakeTorque(self.as_ptr(), id) }
    }

    /// Get the drive torque that has been applied to a specific wheel.
    fn get_drive_torque(&self, id: u32) -> f32 {
        unsafe { PxVehicleNoDrive_getDriveTorque(self.as_ptr(), id) }
    }

    /// Get the steer angle that has been applied to a specific wheel.
    fn get_steer_angle(&self, id: u32) -> f32 {
        unsafe { PxVehicleNoDrive_getSteerAngle(self.as_ptr(), id) }
    }

    fn get_nb_steer_angle(&self) -> u32 {
        unsafe { PxVehicleNoDrive_getNbSteerAngle(self.as_ptr()) }
    }

    fn get_nb_drive_torque(&self) -> u32 {
        unsafe { PxVehicleNoDrive_getNbDriveTorque(self.as_ptr()) }
    }

    fn get_nb_brake_torque(&self) -> u32 {
        unsafe { PxVehicleNoDrive_getNbBrakeTorque(self.as_ptr()) }
    }
}
