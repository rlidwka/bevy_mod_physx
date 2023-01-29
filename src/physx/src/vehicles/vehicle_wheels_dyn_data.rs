use crate::{
    DeriveClassForNewType,
    owner::Owner,
    traits::Class,
};

use physx_sys::{
    PxVehicleWheelsDynData_new_alloc,
    //PxVehicleWheelsDynData_delete,
    PxVehicleWheelsDynData_setToRestState_mut,
    //PxVehicleWheelsDynData_setTireForceShaderFunction_mut,
    //PxVehicleWheelsDynData_setTireForceShaderData_mut,
    //PxVehicleWheelsDynData_getTireForceShaderData,
    PxVehicleWheelsDynData_setWheelRotationSpeed_mut,
    PxVehicleWheelsDynData_getWheelRotationSpeed,
    PxVehicleWheelsDynData_setWheelRotationAngle_mut,
    PxVehicleWheelsDynData_getWheelRotationAngle,
    //PxVehicleWheelsDynData_setUserData_mut,
    //PxVehicleWheelsDynData_getUserData,
    PxVehicleWheelsDynData_copy_mut,
    //PxVehicleWheelsDynData_getBinaryMetaData_mut,
    PxVehicleWheelsDynData_getNbWheelRotationSpeed,
    PxVehicleWheelsDynData_getNbWheelRotationAngle,
    //PxVehicleWheelsDynData_getWheel4DynData,
};

#[repr(transparent)]
pub struct VehicleWheelsDynData {
    obj: physx_sys::PxVehicleWheelsDynData,
}

DeriveClassForNewType!(VehicleWheelsDynData: PxVehicleWheelsDynData);

impl VehicleWheelsDynData {
    pub fn new() -> Option<Owner<Self>> {
        unsafe {
            VehicleWheelsDynData::from_raw(
                PxVehicleWheelsDynData_new_alloc()
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
        ptr: *mut physx_sys::PxVehicleWheelsDynData,
    ) -> Option<Owner<Self>> {
        Owner::from_raw(ptr as *mut Self)
    }

    /// Set all wheels to their rest state.
    pub fn set_to_rest_state(&mut self) {
        unsafe { PxVehicleWheelsDynData_setToRestState_mut(self.as_mut_ptr()) }
    }

    /// Set the tire force shader function.
    /*pub fn set_tire_force_shader_function(&mut self, tire_force_shader_fn: ...) {
        unsafe { PxVehicleWheelsDynData_setTireForceShaderFunction_mut(self.as_mut_ptr(), tire_force_shader_fn) }
    }*/

    /// Set the tire force shader function.
    /*pub fn set_tire_force_shader_data(&mut self, tire_id: u32, tire_force_shader_data: ...) {
        unsafe { PxVehicleWheelsDynData_setTireForceShaderData_mut(self.as_mut_ptr(), tire_id, tire_force_shader_data) }
    }*/

    /// Set the tire force shader function.
    /*pub fn get_tire_force_shader_data(&self, tire_id: u32) -> ... {
        unsafe { PxVehicleWheelsDynData_getTireForceShaderData(self.as_ptr(), tire_id) }
    }*/

    /// Set the wheel rotation speed (radians per second) about the rolling axis for the specified wheel.
    pub fn set_wheel_rotation_speed(&mut self, wheel_idx: u32, speed: f32) {
        unsafe { PxVehicleWheelsDynData_setWheelRotationSpeed_mut(self.as_mut_ptr(), wheel_idx, speed) }
    }

    /// Return the rotation speed about the rolling axis of a specified wheel.
    pub fn get_wheel_rotation_speed(&self, wheel_idx: u32) -> f32 {
        unsafe { PxVehicleWheelsDynData_getWheelRotationSpeed(self.as_ptr(), wheel_idx) }
    }

    /// Set the wheel rotation angle (radians) about the rolling axis of the specified wheel.
    pub fn set_wheel_rotation_angle(&mut self, wheel_idx: u32, angle: f32) {
        unsafe { PxVehicleWheelsDynData_setWheelRotationAngle_mut(self.as_mut_ptr(), wheel_idx, angle) }
    }

    /// Return the rotation angle about the rolling axis for the specified wheel.
    pub fn get_wheel_rotation_angle(&self, wheel_idx: u32) -> f32 {
        unsafe { PxVehicleWheelsDynData_getWheelRotationAngle(self.as_ptr(), wheel_idx) }
    }

    /// Copy the dynamics data of a single wheel unit (wheel, suspension, tire) from srcWheel of src to trgWheel.
    pub fn copy(&mut self, src: &Self, src_wheel: u32, trg_wheel: u32) {
        unsafe { PxVehicleWheelsDynData_copy_mut(self.as_mut_ptr(), src.as_ptr(), src_wheel, trg_wheel) }
    }

    pub fn get_nb_wheel_rotation_speed(&self) -> u32 {
        unsafe { PxVehicleWheelsDynData_getNbWheelRotationSpeed(self.as_ptr()) }
    }

    pub fn get_nb_wheel_rotation_angle(&self) -> u32 {
        unsafe { PxVehicleWheelsDynData_getNbWheelRotationAngle(self.as_ptr()) }
    }
}
