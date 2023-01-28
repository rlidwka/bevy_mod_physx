use crate::{
    DeriveClassForNewType,
    traits::Class,
};

use physx_sys::{
    PxVehicleAutoBoxData,
    PxVehicleClutchData,
    PxVehicleEngineData,
    PxVehicleGearsData,
    PxVehicleDriveSimData_getEngineData,
    PxVehicleDriveSimData_setEngineData_mut,
    PxVehicleDriveSimData_getGearsData,
    PxVehicleDriveSimData_setGearsData_mut,
    PxVehicleDriveSimData_getClutchData,
    PxVehicleDriveSimData_setClutchData_mut,
    PxVehicleDriveSimData_getAutoBoxData,
    PxVehicleDriveSimData_setAutoBoxData_mut,
    PxVehicleDriveSimData_new,
    //PxVehicleDriveSimData_new_1,
    //PxVehicleDriveSimData_getBinaryMetaData_mut,
    //PxVehicleDriveSimData_delete,
};

use super::{
    VehicleAutoBoxData,
    VehicleClutchData,
    VehicleEngineData,
    VehicleGearsData,
};

#[repr(transparent)]
#[derive(Clone)]
pub struct PxVehicleDriveSimData {
    obj: physx_sys::PxVehicleDriveSimData,
}

impl Default for PxVehicleDriveSimData {
    fn default() -> Self {
        Self { obj: unsafe { PxVehicleDriveSimData_new() } }
    }
}

DeriveClassForNewType!(PxVehicleDriveSimData: PxVehicleDriveSimData);

impl<T> VehicleDriveSimData for T where T: Class<physx_sys::PxVehicleDriveSimData> {}

pub trait VehicleDriveSimData: Class<physx_sys::PxVehicleDriveSimData> + Sized {
    /// Return the engine data.
    fn get_engine_data(&self) -> VehicleEngineData {
        unsafe { (*PxVehicleDriveSimData_getEngineData(self.as_ptr())).into() }
    }

    /// Set the engine data.
    fn set_engine_data(&mut self, engine: VehicleEngineData) {
        let engine: PxVehicleEngineData = engine.into();
        unsafe { PxVehicleDriveSimData_setEngineData_mut(self.as_mut_ptr(), &engine as *const _) }
    }

    /// Return the gears data.
    fn get_gears_data(&self) -> VehicleGearsData {
        unsafe { (*PxVehicleDriveSimData_getGearsData(self.as_ptr())).into() }
    }

    /// Set the gears data.
    fn set_gears_data(&mut self, gears: VehicleGearsData) {
        let gears: PxVehicleGearsData = gears.into();
        unsafe { PxVehicleDriveSimData_setGearsData_mut(self.as_mut_ptr(), &gears as *const _) }
    }

    /// Return the clutch data.
    fn get_clutch_data(&self) -> VehicleClutchData {
        unsafe { (*PxVehicleDriveSimData_getClutchData(self.as_ptr())).into() }
    }

    /// Set the clutch data.
    fn set_clutch_data(&mut self, clutch: VehicleClutchData) {
        let clutch: PxVehicleClutchData = clutch.into();
        unsafe { PxVehicleDriveSimData_setClutchData_mut(self.as_mut_ptr(), &clutch as *const _) }
    }

    /// Return the autobox data.
    fn get_autobox_data(&self) -> VehicleAutoBoxData {
        unsafe { (*PxVehicleDriveSimData_getAutoBoxData(self.as_ptr())).into() }
    }

    /// Set the autobox data.
    fn set_autobox_data(&mut self, autobox: VehicleAutoBoxData) {
        let autobox: PxVehicleAutoBoxData = autobox.into();
        unsafe { PxVehicleDriveSimData_setAutoBoxData_mut(self.as_mut_ptr(), &autobox as *const _) }
    }
}
