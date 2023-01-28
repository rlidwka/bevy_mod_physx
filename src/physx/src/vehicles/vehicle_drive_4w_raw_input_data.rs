use crate::{
    DeriveClassForNewType,
    owner::Owner,
    traits::Class,
};

use physx_sys::{
    PxVehicleDrive4WRawInputData_new_alloc,
    PxVehicleDrive4WRawInputData_delete,
    PxVehicleDrive4WRawInputData_setDigitalAccel_mut,
    PxVehicleDrive4WRawInputData_setDigitalBrake_mut,
    PxVehicleDrive4WRawInputData_setDigitalHandbrake_mut,
    PxVehicleDrive4WRawInputData_setDigitalSteerLeft_mut,
    PxVehicleDrive4WRawInputData_setDigitalSteerRight_mut,
    PxVehicleDrive4WRawInputData_getDigitalAccel,
    PxVehicleDrive4WRawInputData_getDigitalBrake,
    PxVehicleDrive4WRawInputData_getDigitalHandbrake,
    PxVehicleDrive4WRawInputData_getDigitalSteerLeft,
    PxVehicleDrive4WRawInputData_getDigitalSteerRight,
    PxVehicleDrive4WRawInputData_setAnalogAccel_mut,
    PxVehicleDrive4WRawInputData_setAnalogBrake_mut,
    PxVehicleDrive4WRawInputData_setAnalogHandbrake_mut,
    PxVehicleDrive4WRawInputData_setAnalogSteer_mut,
    PxVehicleDrive4WRawInputData_getAnalogAccel,
    PxVehicleDrive4WRawInputData_getAnalogBrake,
    PxVehicleDrive4WRawInputData_getAnalogHandbrake,
    PxVehicleDrive4WRawInputData_getAnalogSteer,
    PxVehicleDrive4WRawInputData_setGearUp_mut,
    PxVehicleDrive4WRawInputData_setGearDown_mut,
    PxVehicleDrive4WRawInputData_getGearUp,
    PxVehicleDrive4WRawInputData_getGearDown,
};

#[repr(transparent)]
#[derive(Clone)]
pub struct PxVehicleDrive4WRawInputData {
    obj: physx_sys::PxVehicleDrive4WRawInputData,
}

impl Drop for PxVehicleDrive4WRawInputData {
    fn drop(&mut self) {
        unsafe { PxVehicleDrive4WRawInputData_delete(self.as_mut_ptr()) }
    }
}

DeriveClassForNewType!(PxVehicleDrive4WRawInputData: PxVehicleDrive4WRawInputData);

impl<T> VehicleDrive4WRawInputData for T where T: Class<physx_sys::PxVehicleDrive4WRawInputData> {}

pub trait VehicleDrive4WRawInputData: Class<physx_sys::PxVehicleDrive4WRawInputData> + Sized {
    fn new() -> Option<Owner<Self>> {
        unsafe {
            VehicleDrive4WRawInputData::from_raw(
                PxVehicleDrive4WRawInputData_new_alloc()
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
        ptr: *mut physx_sys::PxVehicleDrive4WRawInputData,
    ) -> Option<Owner<Self>> {
        Owner::from_raw(ptr as *mut Self)
    }

    /// Record if the accel button has been pressed on keyboard.
    fn set_digital_accel(&mut self, accel_key_pressed: bool) {
        unsafe { PxVehicleDrive4WRawInputData_setDigitalAccel_mut(self.as_mut_ptr(), accel_key_pressed) }
    }

    /// Record if the brake button has been pressed on keyboard.
    fn set_digital_brake(&mut self, brake_key_pressed: bool) {
        unsafe { PxVehicleDrive4WRawInputData_setDigitalBrake_mut(self.as_mut_ptr(), brake_key_pressed) }
    }

    /// Record if the handbrake button has been pressed on keyboard.
    fn set_digital_handbrake(&mut self, handbrake_key_pressed: bool) {
        unsafe { PxVehicleDrive4WRawInputData_setDigitalHandbrake_mut(self.as_mut_ptr(), handbrake_key_pressed) }
    }

    /// Record if the left steer button has been pressed on keyboard.
    fn set_digital_steer_left(&mut self, steer_left_key_pressed: bool) {
        unsafe { PxVehicleDrive4WRawInputData_setDigitalSteerLeft_mut(self.as_mut_ptr(), steer_left_key_pressed) }
    }

    /// Record if the right steer button has been pressed on keyboard.
    fn set_digital_steer_right(&mut self, steer_right_key_pressed: bool) {
        unsafe { PxVehicleDrive4WRawInputData_setDigitalSteerRight_mut(self.as_mut_ptr(), steer_right_key_pressed) }
    }

    /// Return if the accel button has been pressed on keyboard.
    fn get_digital_accel(&self) -> bool {
        unsafe { PxVehicleDrive4WRawInputData_getDigitalAccel(self.as_ptr()) }
    }

    /// Return if the brake button has been pressed on keyboard.
    fn get_digital_brake(&self) -> bool {
        unsafe { PxVehicleDrive4WRawInputData_getDigitalBrake(self.as_ptr()) }
    }

    /// Return if the handbrake button has been pressed on keyboard.
    fn get_digital_handbrake(&self) -> bool {
        unsafe { PxVehicleDrive4WRawInputData_getDigitalHandbrake(self.as_ptr()) }
    }

    /// Return if the left steer button has been pressed on keyboard.
    fn get_digital_steer_left(&self) -> bool {
        unsafe { PxVehicleDrive4WRawInputData_getDigitalSteerLeft(self.as_ptr()) }
    }

    /// Return if the right steer button has been pressed on keyboard.
    fn get_digital_steer_right(&self) -> bool {
        unsafe { PxVehicleDrive4WRawInputData_getDigitalSteerRight(self.as_ptr()) }
    }

    /// Set the analog accel value from the gamepad.
    fn set_analog_accel(&mut self, accel: f32) {
        unsafe { PxVehicleDrive4WRawInputData_setAnalogAccel_mut(self.as_mut_ptr(), accel) }
    }

    /// Set the analog brake value from the gamepad.
    fn set_analog_brake(&mut self, brake: f32) {
        unsafe { PxVehicleDrive4WRawInputData_setAnalogBrake_mut(self.as_mut_ptr(), brake) }
    }

    /// Set the analog handbrake value from the gamepad.
    fn set_analog_handbrake(&mut self, handbrake: f32) {
        unsafe { PxVehicleDrive4WRawInputData_setAnalogHandbrake_mut(self.as_mut_ptr(), handbrake) }
    }

    /// Set the analog steer value from the gamepad.
    fn set_analog_steer(&mut self, steer: f32) {
        unsafe { PxVehicleDrive4WRawInputData_setAnalogSteer_mut(self.as_mut_ptr(), steer) }
    }

    /// Return the analog accel value from the gamepad.
    fn get_analog_accel(&self) -> f32 {
        unsafe { PxVehicleDrive4WRawInputData_getAnalogAccel(self.as_ptr()) }
    }

    /// Return the analog brake value from the gamepad.
    fn get_analog_brake(&self) -> f32 {
        unsafe { PxVehicleDrive4WRawInputData_getAnalogBrake(self.as_ptr()) }
    }

    /// Return the analog handbrake value from the gamepad.
    fn get_analog_handbrake(&self) -> f32 {
        unsafe { PxVehicleDrive4WRawInputData_getAnalogHandbrake(self.as_ptr()) }
    }

    /// Return the analog steer value from the gamepad.
    fn get_analog_steer(&self) -> f32 {
        unsafe { PxVehicleDrive4WRawInputData_getAnalogSteer(self.as_ptr()) }
    }

    /// Record if the gear-up button has been pressed on keyboard or gamepad.
    fn set_gear_up(&mut self, gear_up_key_pressed: bool) {
        unsafe { PxVehicleDrive4WRawInputData_setGearUp_mut(self.as_mut_ptr(), gear_up_key_pressed) }
    }

    /// Record if the gear-down button has been pressed on keyboard or gamepad.
    fn set_gear_down(&mut self, gear_down_key_pressed: bool) {
        unsafe { PxVehicleDrive4WRawInputData_setGearDown_mut(self.as_mut_ptr(), gear_down_key_pressed) }
    }

    /// Return if the gear-up button has been pressed on keyboard or gamepad.
    fn get_gear_up(&self) -> bool {
        unsafe { PxVehicleDrive4WRawInputData_getGearUp(self.as_ptr()) }
    }

    /// Return if the gear-down button has been pressed on keyboard or gamepad.
    fn get_gear_down(&self) -> bool {
        unsafe { PxVehicleDrive4WRawInputData_getGearDown(self.as_ptr()) }
    }
}
