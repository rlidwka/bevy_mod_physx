use crate::{
    DeriveClassForNewType,
    owner::Owner,
    traits::Class,
};

use physx_sys::{
    PxVehicleDriveNWRawInputData_new_alloc,
    PxVehicleDriveNWRawInputData_delete,
};

use super::VehicleDrive4WRawInputData;

#[repr(transparent)]
pub struct PxVehicleDriveNWRawInputData {
    obj: physx_sys::PxVehicleDriveNWRawInputData,
}

impl Drop for PxVehicleDriveNWRawInputData {
    fn drop(&mut self) {
        unsafe { PxVehicleDriveNWRawInputData_delete(self.as_mut_ptr()) }
    }
}

DeriveClassForNewType!(PxVehicleDriveNWRawInputData: PxVehicleDriveNWRawInputData, PxVehicleDrive4WRawInputData);

impl<T> VehicleDriveNWRawInputData for T where T: Class<physx_sys::PxVehicleDriveNWRawInputData> + VehicleDrive4WRawInputData {}

pub trait VehicleDriveNWRawInputData: Class<physx_sys::PxVehicleDriveNWRawInputData> + VehicleDrive4WRawInputData {
    fn new() -> Option<Owner<Self>> {
        unsafe {
            VehicleDriveNWRawInputData::from_raw(
                PxVehicleDriveNWRawInputData_new_alloc()
            )
        }
    }

    /// Create a new Owner wrapper around a raw pointer.
    /// # Safety
    /// Owner's own the pointer they wrap, using the pointer after dropping the Owner,
    /// or creating multiple Owners from the same pointer will cause UB.  Use `into_ptr` to
    /// retrieve the pointer and consume the Owner without dropping the pointee.
    /// Initializes user data.
    unsafe fn from_raw(
        ptr: *mut physx_sys::PxVehicleDriveNWRawInputData,
    ) -> Option<Owner<Self>> {
        Owner::from_raw(ptr as *mut Self)
    }
}
