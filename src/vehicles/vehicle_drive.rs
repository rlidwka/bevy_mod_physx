use physx::traits::Class;
use super::VehicleWheels;

//use physx_sys::{
    //PxVehicleDrive_getBinaryMetaData_mut,
    //PxVehicleDrive_new_alloc,
    //PxVehicleDrive_getConcreteTypeName,
//}

impl<T> VehicleDrive for T where T: Class<physx_sys::PxVehicleDrive> + VehicleWheels {}

pub trait VehicleDrive: Class<physx_sys::PxVehicleDrive> + VehicleWheels {}
