use physx::{
    traits::Class,
    DeriveClassForNewType,
};

use physx_sys::{
    PxVehicleAckermannGeometryData,
    PxVehicleDifferential4WData,
    //PxVehicleDriveSimData4W_new,
    PxVehicleDriveSimData4W_getDiffData,
    PxVehicleDriveSimData4W_getAckermannGeometryData,
    PxVehicleDriveSimData4W_setDiffData_mut,
    PxVehicleDriveSimData4W_setAckermannGeometryData_mut,
    //PxVehicleDriveSimData4W_new_1,
    //PxVehicleDriveSimData4W_getBinaryMetaData_mut,
    //PxVehicleDriveSimData4W_delete,
};

use super::{
    VehicleAckermannGeometryData,
    VehicleDifferential4WData,
    VehicleDriveSimData,
};

#[repr(transparent)]
#[derive(Clone)]
pub struct PxVehicleDriveSimData4W {
    obj: physx_sys::PxVehicleDriveSimData4W,
}

DeriveClassForNewType!(PxVehicleDriveSimData4W: PxVehicleDriveSimData4W);

impl<T> VehicleDriveSimData4W for T where T: Class<physx_sys::PxVehicleDriveSimData4W> + VehicleDriveSimData {}

pub trait VehicleDriveSimData4W: Class<physx_sys::PxVehicleDriveSimData4W> + VehicleDriveSimData {
    /// Return the data describing the differential.
    fn get_diff_data(&self) -> VehicleDifferential4WData {
        unsafe { (*PxVehicleDriveSimData4W_getDiffData(self.as_ptr())).into() }
    }

    /// Return the data describing the Ackermann steer-correction.
    fn get_ackermann_geometry_data(&self) -> VehicleAckermannGeometryData {
        unsafe { (*PxVehicleDriveSimData4W_getAckermannGeometryData(self.as_ptr())).into() }
    }

    /// Set the data describing the differential.
    fn set_diff_data(&mut self, diff: VehicleDifferential4WData) {
        let diff: PxVehicleDifferential4WData = diff.into();
        unsafe { PxVehicleDriveSimData4W_setDiffData_mut(self.as_mut_ptr(), &diff as *const _) }
    }

    /// Set the data describing the Ackermann steer-correction.
    fn set_ackermann_geometry_data(&mut self, diff: VehicleAckermannGeometryData) {
        let diff: PxVehicleAckermannGeometryData = diff.into();
        unsafe { PxVehicleDriveSimData4W_setAckermannGeometryData_mut(self.as_mut_ptr(), &diff as *const _) }
    }
}
