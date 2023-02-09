use crate::{
    DeriveClassForNewType,
    owner::Owner,
    physics::Physics,
    rigid_dynamic::RigidDynamic,
    traits::Class,
};

use physx_sys::{
    PxVehicleDriveTank_allocate_mut,
    PxVehicleDriveTank_free_mut,
    PxVehicleDriveTank_setup_mut,
    PxVehicleDriveTank_create_mut,
    PxVehicleDriveTank_setDriveModel_mut,
    PxVehicleDriveTank_getDriveModel,
    PxVehicleDriveTank_setToRestState_mut,
    //PxVehicleDriveTank_new_alloc,
    //PxVehicleDriveTank_createObject_mut,
    //PxVehicleDriveTank_getBinaryMetaData_mut,
    //PxVehicleDriveTank_getConcreteTypeName,
    //PxVehicleDriveTank_isKindOf,
};

use super::{
    PxVehicleDriveDynData,
    PxVehicleDriveSimData,
    VehicleDrive,
    VehicleDriveSimData,
    VehicleWheelsDynData,
    VehicleWheelsSimData,
};

#[repr(transparent)]
pub struct PxVehicleDriveTank {
    obj: physx_sys::PxVehicleDriveTank,
}

unsafe impl Send for PxVehicleDriveTank {}
unsafe impl Sync for PxVehicleDriveTank {}

impl PxVehicleDriveTank {
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

impl Drop for PxVehicleDriveTank {
    fn drop(&mut self) {
        unsafe { PxVehicleDriveTank_free_mut(self.as_mut_ptr()) }
    }
}

DeriveClassForNewType!(PxVehicleDriveTank: PxVehicleDriveTank, PxVehicleDrive, PxVehicleWheels, PxBase);

impl<T> VehicleDriveTank for T where T: Class<physx_sys::PxVehicleDriveTank> + VehicleDrive {}

pub trait VehicleDriveTank: Class<physx_sys::PxVehicleDriveTank> + VehicleDrive {
    /// Allocate and set up a tank using simulation data for the wheels and drive model.
    fn new(
        physics: &mut impl Physics,
        veh_actor: &mut impl RigidDynamic,
        wheels_data: &VehicleWheelsSimData,
        drive_data: &impl VehicleDriveSimData,
        nb_driven_wheels: u32,
    ) -> Option<Owner<Self>> {
        unsafe {
            VehicleDriveTank::from_raw(
                PxVehicleDriveTank_create_mut(
                    physics.as_mut_ptr(),
                    veh_actor.as_mut_ptr(),
                    wheels_data.as_ptr(),
                    drive_data.as_ptr(),
                    nb_driven_wheels,
                )
            )
        }
    }

    /// Allocate a PxVehicleTankDrive instance for a tank with nbWheels.
    fn allocate(nb_wheels: u32) -> Option<Owner<Self>> {
        unsafe {
            VehicleDriveTank::from_raw(PxVehicleDriveTank_allocate_mut(nb_wheels))
        }
    }

    /// Set up a tank using simulation data for the wheels and drive model.
    fn setup(
        &mut self,
        physics: &mut impl Physics,
        veh_actor: &mut impl RigidDynamic,
        wheels_data: &VehicleWheelsSimData,
        drive_data: &impl VehicleDriveSimData,
        nb_driven_wheels: u32,
    ) {
        unsafe { PxVehicleDriveTank_setup_mut(self.as_mut_ptr(), physics.as_mut_ptr(), veh_actor.as_mut_ptr(), wheels_data.as_ptr(), drive_data.as_ptr(), nb_driven_wheels) }
    }

    /// Create a new Owner wrapper around a raw pointer.
    /// # Safety
    /// Owner's own the pointer they wrap, using the pointer after dropping the Owner,
    /// or creating multiple Owners from the same pointer will cause UB.  Use `into_ptr` to
    /// retrieve the pointer and consume the Owner without dropping the pointee.
    /// Initializes user data.
    unsafe fn from_raw(
        ptr: *mut physx_sys::PxVehicleDriveTank,
    ) -> Option<Owner<Self>> {
        Owner::from_raw(ptr as *mut Self)
    }

    /// Set the control model used by the tank.
    fn set_drive_model(&mut self, drive_model: VehicleDriveTankControlModel) {
        unsafe { PxVehicleDriveTank_setDriveModel_mut(self.as_mut_ptr(), drive_model.into()) }
    }

    /// Return the control model used by the tank.
    fn get_drive_model(&self) -> VehicleDriveTankControlModel {
        unsafe { PxVehicleDriveTank_getDriveModel(self.as_ptr()).into() }
    }

    /// Set a vehicle to its rest state. Aside from the rigid body transform, this will set the vehicle and rigid body to the state they were in immediately after setup or create.
    fn set_to_rest_state(&mut self) {
        unsafe { PxVehicleDriveTank_setToRestState_mut(self.as_mut_ptr()) }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum VehicleDriveTankControl {
    AnalogInputAccel = 0,
    AnalogInputBrakeLeft = 1,
    AnalogInputBrakeRight = 2,
    AnalogInputThrustLeft = 3,
    AnalogInputThrustRight = 4,
}

impl VehicleDriveTankControl {
    pub const MAX_NB_ANALOG_INPUTS: u32 = 5;
}

impl From<VehicleDriveTankControl> for physx_sys::PxVehicleDriveTankControl::Enum {
    fn from(value: VehicleDriveTankControl) -> Self {
        value as u32
    }
}

impl From<physx_sys::PxVehicleDriveTankControl::Enum> for VehicleDriveTankControl {
    fn from(ty: physx_sys::PxVehicleDriveTankControl::Enum) -> Self {
        match ty {
            0 => Self::AnalogInputAccel,
            1 => Self::AnalogInputBrakeLeft,
            2 => Self::AnalogInputBrakeRight,
            3 => Self::AnalogInputThrustLeft,
            4 => Self::AnalogInputThrustRight,
            _ => panic!("invalid enum variant"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum VehicleDriveTankControlModel {
    Standard = 0,
    Special = 1,
}

impl From<VehicleDriveTankControlModel> for physx_sys::PxVehicleDriveTankControlModel::Enum {
    fn from(value: VehicleDriveTankControlModel) -> Self {
        value as u32
    }
}

impl From<physx_sys::PxVehicleDriveTankControlModel::Enum> for VehicleDriveTankControlModel {
    fn from(ty: physx_sys::PxVehicleDriveTankControlModel::Enum) -> Self {
        match ty {
            0 => Self::Standard,
            1 => Self::Special,
            _ => panic!("invalid enum variant"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum VehicleDriveTankWheelOrder {
    FrontLeft0 = 0,
    FrontRight0 = 1,
    FrontLeft1 = 2,
    FrontRight1 = 3,
    FrontLeft2 = 4,
    FrontRight2 = 5,
    FrontLeft3 = 6,
    FrontRight3 = 7,
    FrontLeft4 = 8,
    FrontRight4 = 9,
    FrontLeft5 = 10,
    FrontRight5 = 11,
    FrontLeft6 = 12,
    FrontRight6 = 13,
    FrontLeft7 = 14,
    FrontRight7 = 15,
    FrontLeft8 = 16,
    FrontRight8 = 17,
    FrontLeft9 = 18,
    FrontRight9 = 19,
}

impl From<VehicleDriveTankWheelOrder> for physx_sys::PxVehicleDriveTankWheelOrder::Enum {
    fn from(value: VehicleDriveTankWheelOrder) -> Self {
        value as u32
    }
}

impl From<physx_sys::PxVehicleDriveTankWheelOrder::Enum> for VehicleDriveTankWheelOrder {
    fn from(ty: physx_sys::PxVehicleDriveTankWheelOrder::Enum) -> Self {
        match ty {
            0 => Self::FrontLeft0,
            1 => Self::FrontRight0,
            2 => Self::FrontLeft1,
            3 => Self::FrontRight1,
            4 => Self::FrontLeft2,
            5 => Self::FrontRight2,
            6 => Self::FrontLeft3,
            7 => Self::FrontRight3,
            8 => Self::FrontLeft4,
            9 => Self::FrontRight4,
            10 => Self::FrontLeft5,
            11 => Self::FrontRight5,
            12 => Self::FrontLeft6,
            13 => Self::FrontRight6,
            14 => Self::FrontLeft7,
            15 => Self::FrontRight7,
            16 => Self::FrontLeft8,
            17 => Self::FrontRight8,
            18 => Self::FrontLeft9,
            19 => Self::FrontRight9,
            _ => panic!("invalid enum variant"),
        }
    }
}
