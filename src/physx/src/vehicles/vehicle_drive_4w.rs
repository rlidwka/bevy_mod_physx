use crate::{
    DeriveClassForNewType,
    owner::Owner,
    physics::Physics,
    rigid_dynamic::RigidDynamic,
    traits::Class,
};

use physx_sys::{
    PxVehicleDrive4W_allocate_mut,
    PxVehicleDrive4W_free_mut,
    PxVehicleDrive4W_setup_mut,
    PxVehicleDrive4W_create_mut,
    PxVehicleDrive4W_setToRestState_mut,
    //PxVehicleDrive4W_createObject_mut,
    //PxVehicleDrive4W_getBinaryMetaData_mut,
    //PxVehicleDrive4W_new_alloc,
    //PxVehicleDrive4W_getConcreteTypeName,
};

use super::{
    PxVehicleDriveDynData,
    PxVehicleDriveSimData,
    VehicleDrive,
    VehicleDriveSimData4W,
    VehicleWheelsDynData,
    VehicleWheelsSimData,
};

#[repr(transparent)]
pub struct PxVehicleDrive4W {
    obj: physx_sys::PxVehicleDrive4W,
}

unsafe impl Send for PxVehicleDrive4W {}
unsafe impl Sync for PxVehicleDrive4W {}

impl PxVehicleDrive4W {
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

    /// Simulation data that models vehicle components.
    pub fn drive_sim_data(&self) -> &PxVehicleDriveSimData {
        // SAFETY: VehicleDriveSimData is repr(transparent)
        unsafe { std::mem::transmute(&self.obj.mDriveSimData) }
    }

    /// Simulation data that models vehicle components.
    pub fn drive_sim_data_mut(&mut self) -> &mut PxVehicleDriveSimData {
        // SAFETY: VehicleDriveSimData is repr(transparent)
        unsafe { std::mem::transmute(&mut self.obj.mDriveSimData) }
    }

    /// Dynamics data of vehicle instance.
    pub fn drive_dyn_data(&self) -> &PxVehicleDriveDynData {
        // SAFETY: VehicleDriveDynData is repr(transparent)
        unsafe { std::mem::transmute(&self.obj.mDriveDynData) }
    }

    /// Dynamics data of vehicle instance.
    pub fn drive_dyn_data_mut(&mut self) -> &mut PxVehicleDriveDynData {
        // SAFETY: VehicleDriveDynData is repr(transparent)
        unsafe { std::mem::transmute(&mut self.obj.mDriveDynData) }
    }
}

impl Drop for PxVehicleDrive4W {
    fn drop(&mut self) {
        unsafe { PxVehicleDrive4W_free_mut(self.as_mut_ptr()) }
    }
}

DeriveClassForNewType!(PxVehicleDrive4W: PxVehicleDrive4W, PxVehicleDrive, PxVehicleWheels, PxBase);

impl<T> VehicleDrive4W for T where T: Class<physx_sys::PxVehicleDrive4W> + VehicleDrive {}

pub trait VehicleDrive4W: Class<physx_sys::PxVehicleDrive4W> + VehicleDrive {
    /// Allocate and set up a vehicle using simulation data for the wheels and drive model.
    fn new(
        physics: &mut impl Physics,
        veh_actor: &mut impl RigidDynamic,
        wheels_data: &VehicleWheelsSimData,
        drive_data: &impl VehicleDriveSimData4W,
        nb_non_driven_wheels: u32,
    ) -> Option<Owner<Self>> {
        unsafe {
            VehicleDrive4W::from_raw(
                PxVehicleDrive4W_create_mut(
                    physics.as_mut_ptr(),
                    veh_actor.as_mut_ptr(),
                    wheels_data.as_ptr(),
                    drive_data.as_ptr(),
                    nb_non_driven_wheels,
                )
            )
        }
    }

    /// Allocate a PxVehicleDrive4W instance for a 4WDrive vehicle with nbWheels (= 4 + number of un-driven wheels).
    fn allocate(nb_wheels: u32) -> Option<Owner<Self>> {
        unsafe {
            VehicleDrive4W::from_raw(PxVehicleDrive4W_allocate_mut(nb_wheels))
        }
    }

    /// Set up a vehicle using simulation data for the wheels and drive model.
    fn setup(
        &mut self,
        physics: &mut impl Physics,
        veh_actor: &mut impl RigidDynamic,
        wheels_data: &VehicleWheelsSimData,
        drive_data: &impl VehicleDriveSimData4W,
        nb_non_driven_wheels: u32,
    ) {
        unsafe { PxVehicleDrive4W_setup_mut(self.as_mut_ptr(), physics.as_mut_ptr(), veh_actor.as_mut_ptr(), wheels_data.as_ptr(), drive_data.as_ptr(), nb_non_driven_wheels) }
    }

    /// Create a new Owner wrapper around a raw pointer.
    /// # Safety
    /// Owner's own the pointer they wrap, using the pointer after dropping the Owner,
    /// or creating multiple Owners from the same pointer will cause UB.  Use `into_ptr` to
    /// retrieve the pointer and consume the Owner without dropping the pointee.
    /// Initializes user data.
    unsafe fn from_raw(
        ptr: *mut physx_sys::PxVehicleDrive4W,
    ) -> Option<Owner<Self>> {
        Owner::from_raw(ptr as *mut Self)
    }

    /// Set a vehicle to its rest state. Aside from the rigid body transform, this will set the vehicle and rigid body to the state they were in immediately after setup or create.
    fn set_to_rest_state(&mut self) {
        unsafe { PxVehicleDrive4W_setToRestState_mut(self.as_mut_ptr()) }
    }
}
