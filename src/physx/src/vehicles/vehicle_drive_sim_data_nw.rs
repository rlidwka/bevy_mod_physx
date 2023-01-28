use crate::{
    DeriveClassForNewType,
    traits::Class,
};

use physx_sys::{
    PxVehicleDriveSimDataNW_new,
    PxVehicleDriveSimDataNW_getDiffData,
    PxVehicleDriveSimDataNW_setDiffData_mut,
    //PxVehicleDriveSimDataNW_new_1,
    //PxVehicleDriveSimDataNW_getBinaryMetaData_mut,
    //PxVehicleDriveSimDataNW_delete,
};

use super::{
    VehicleDifferentialNWData,
    VehicleDriveSimData,
};

#[repr(transparent)]
#[derive(Clone)]
pub struct PxVehicleDriveSimDataNW {
    obj: physx_sys::PxVehicleDriveSimDataNW,
}

impl Default for PxVehicleDriveSimDataNW {
    fn default() -> Self {
        Self { obj: unsafe { PxVehicleDriveSimDataNW_new() } }
    }
}

DeriveClassForNewType!(PxVehicleDriveSimDataNW: PxVehicleDriveSimDataNW, PxVehicleDriveSimData);

impl<T> VehicleDriveSimDataNW for T where T: Class<physx_sys::PxVehicleDriveSimDataNW> + VehicleDriveSimData {}

pub trait VehicleDriveSimDataNW: Class<physx_sys::PxVehicleDriveSimDataNW> + VehicleDriveSimData {
    /// Return the data describing the differential of a vehicle with up to PX_MAX_NB_WHEELS driven wheels.
    fn get_diff_data(&self) -> VehicleDifferentialNWData {
        let obj = unsafe { *PxVehicleDriveSimDataNW_getDiffData(self.as_ptr()) };
        VehicleDifferentialNWData { obj }
    }

    /// Set the data describing the differential of a vehicle with up to PX_MAX_NB_WHEELS driven wheels. The differential data describes the set of wheels that are driven by the differential.
    fn set_diff_data(&mut self, diff: VehicleDifferentialNWData) {
        unsafe { PxVehicleDriveSimDataNW_setDiffData_mut(self.as_mut_ptr(), diff.as_ptr()) }
    }
}
