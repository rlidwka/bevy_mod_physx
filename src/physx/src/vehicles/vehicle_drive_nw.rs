use crate::{
    DeriveClassForNewType,
    owner::Owner,
    physics::Physics,
    rigid_dynamic::RigidDynamic,
    traits::Class,
};

use physx_sys::{
    PxVehicleDriveNW_allocate_mut,
    PxVehicleDriveNW_free_mut,
    PxVehicleDriveNW_setup_mut,
    PxVehicleDriveNW_create_mut,
    PxVehicleDriveNW_setToRestState_mut,
    //PxVehicleDriveNW_new_alloc,
    //PxVehicleDriveNW_new_alloc_1,
    //PxVehicleDriveNW_createObject_mut,
    //PxVehicleDriveNW_getBinaryMetaData_mut,
    //PxVehicleDriveNW_getConcreteTypeName,
    //PxVehicleDriveNW_isKindOf,
};

use super::{
    PxVehicleDriveDynData,
    PxVehicleDriveSimData,
    VehicleDrive,
    VehicleDriveControlType,
    VehicleDriveSimDataNW,
    VehicleWheelsDynData,
    VehicleWheelsSimData,
};

#[repr(transparent)]
pub struct PxVehicleDriveNW {
    obj: physx_sys::PxVehicleDriveNW,
}

unsafe impl Send for PxVehicleDriveNW {}
unsafe impl Sync for PxVehicleDriveNW {}

impl PxVehicleDriveNW {
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

impl Drop for PxVehicleDriveNW {
    fn drop(&mut self) {
        unsafe { PxVehicleDriveNW_free_mut(self.as_mut_ptr()) }
    }
}

DeriveClassForNewType!(PxVehicleDriveNW: PxVehicleDriveNW, PxVehicleDrive, PxVehicleWheels, PxBase);

impl<T> VehicleDriveNW for T where T: Class<physx_sys::PxVehicleDriveNW> + VehicleDrive {}

pub trait VehicleDriveNW: Class<physx_sys::PxVehicleDriveNW> + VehicleDrive {
    /// Allocate and set up a vehicle using simulation data for the wheels and drive model.
    fn new(
        physics: &mut impl Physics,
        veh_actor: &mut impl RigidDynamic,
        wheels_data: &VehicleWheelsSimData,
        drive_data: &impl VehicleDriveSimDataNW,
        nb_wheels: u32,
    ) -> Option<Owner<Self>> {
        unsafe {
            VehicleDriveNW::from_raw(
                PxVehicleDriveNW_create_mut(
                    physics.as_mut_ptr(),
                    veh_actor.as_mut_ptr(),
                    wheels_data.as_ptr(),
                    drive_data.as_ptr(),
                    nb_wheels,
                )
            )
        }
    }

    /// Allocate a PxVehicleDriveNW instance for a NWDrive vehicle with nbWheels.
    fn allocate(nb_wheels: u32) -> Option<Owner<Self>> {
        unsafe {
            VehicleDriveNW::from_raw(PxVehicleDriveNW_allocate_mut(nb_wheels))
        }
    }

    /// Set up a vehicle using simulation data for the wheels and drive model.
    fn setup(
        &mut self,
        physics: &mut impl Physics,
        veh_actor: &mut impl RigidDynamic,
        wheels_data: &VehicleWheelsSimData,
        drive_data: &impl VehicleDriveSimDataNW,
        nb_wheels: u32,
    ) {
        unsafe { PxVehicleDriveNW_setup_mut(self.as_mut_ptr(), physics.as_mut_ptr(), veh_actor.as_mut_ptr(), wheels_data.as_ptr(), drive_data.as_ptr(), nb_wheels) }
    }

    /// Create a new Owner wrapper around a raw pointer.
    /// # Safety
    /// Owner's own the pointer they wrap, using the pointer after dropping the Owner,
    /// or creating multiple Owners from the same pointer will cause UB.  Use `into_ptr` to
    /// retrieve the pointer and consume the Owner without dropping the pointee.
    /// Initializes user data.
    unsafe fn from_raw(
        ptr: *mut physx_sys::PxVehicleDriveNW,
    ) -> Option<Owner<Self>> {
        Owner::from_raw(ptr as *mut Self)
    }

    /// Set a vehicle to its rest state. Aside from the rigid body transform, this will set the vehicle and rigid body to the state they were in immediately after setup or create.
    fn set_to_rest_state(&mut self) {
        unsafe { PxVehicleDriveNW_setToRestState_mut(self.as_mut_ptr()) }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum VehicleDriveNWControl {
    AnalogInputAccel = 0,
    AnalogInputBrake = 1,
    AnalogInputHandBrake = 2,
    AnalogInputSteerLeft = 3,
    AnalogInputSteerRight = 4,
}

impl VehicleDriveControlType for VehicleDriveNWControl {}

impl VehicleDriveNWControl {
    pub const MAX_NB_ANALOG_INPUTS: u32 = 5;
}

impl From<VehicleDriveNWControl> for physx_sys::PxVehicleDriveNWControl::Enum {
    fn from(value: VehicleDriveNWControl) -> Self {
        value as u32
    }
}

impl From<physx_sys::PxVehicleDriveNWControl::Enum> for VehicleDriveNWControl {
    fn from(ty: physx_sys::PxVehicleDriveNWControl::Enum) -> Self {
        match ty {
            0 => Self::AnalogInputAccel,
            1 => Self::AnalogInputBrake,
            2 => Self::AnalogInputHandBrake,
            3 => Self::AnalogInputSteerLeft,
            4 => Self::AnalogInputSteerRight,
            _ => panic!("invalid enum variant"),
        }
    }
}
