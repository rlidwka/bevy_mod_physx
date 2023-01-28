use crate::{
    DeriveClassForNewType,
    traits::Class,
};

use physx_sys::{
    PxVehicleDifferentialNWData_new,
    PxVehicleDifferentialNWData_setDrivenWheel_mut,
    PxVehicleDifferentialNWData_getIsDrivenWheel,
    //PxVehicleDifferentialNWData_new_1,
    PxVehicleDifferentialNWData_getDrivenWheelStatus,
    PxVehicleDifferentialNWData_setDrivenWheelStatus_mut,
};

#[repr(transparent)]
#[derive(Clone)]
pub struct VehicleDifferentialNWData {
    pub(crate) obj: physx_sys::PxVehicleDifferentialNWData,
}

impl Default for VehicleDifferentialNWData {
    fn default() -> Self {
        Self { obj: unsafe { PxVehicleDifferentialNWData_new() } }
    }
}

DeriveClassForNewType!(VehicleDifferentialNWData: PxVehicleDifferentialNWData);

impl VehicleDifferentialNWData {
    /// Set a specific wheel to be driven or non-driven by the differential.
    pub fn set_driven_wheel(&mut self, wheel_id: u32, driven_state: bool) {
        unsafe { PxVehicleDifferentialNWData_setDrivenWheel_mut(self.as_mut_ptr(), wheel_id, driven_state) }
    }

    /// Test if a specific wheel has been configured as a driven or non-driven wheel.
    pub fn get_is_driven_wheel(&self, wheel_id: u32) -> bool {
        unsafe { PxVehicleDifferentialNWData_getIsDrivenWheel(self.as_ptr(), wheel_id) }
    }

    pub fn get_driven_wheel_status(&self) -> u32 {
        unsafe { PxVehicleDifferentialNWData_getDrivenWheelStatus(self.as_ptr()) }
    }

    pub fn set_driven_wheel_status(&mut self, status: u32) {
        unsafe { PxVehicleDifferentialNWData_setDrivenWheelStatus_mut(self.as_mut_ptr(), status) }
    }
}
