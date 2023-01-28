use crate::{
    DeriveClassForNewType,
    owner::Owner,
    traits::Class,
};

use physx_sys::{
    PxVehicleDriveTankRawInputData_new_alloc,
    PxVehicleDriveTankRawInputData_delete,
    PxVehicleDriveTankRawInputData_getDriveModel,
    PxVehicleDriveTankRawInputData_setDigitalAccel_mut,
    PxVehicleDriveTankRawInputData_setDigitalLeftThrust_mut,
    PxVehicleDriveTankRawInputData_setDigitalRightThrust_mut,
    PxVehicleDriveTankRawInputData_setDigitalLeftBrake_mut,
    PxVehicleDriveTankRawInputData_setDigitalRightBrake_mut,
    PxVehicleDriveTankRawInputData_getDigitalAccel,
    PxVehicleDriveTankRawInputData_getDigitalLeftThrust,
    PxVehicleDriveTankRawInputData_getDigitalRightThrust,
    PxVehicleDriveTankRawInputData_getDigitalLeftBrake,
    PxVehicleDriveTankRawInputData_getDigitalRightBrake,
    PxVehicleDriveTankRawInputData_setAnalogAccel_mut,
    PxVehicleDriveTankRawInputData_setAnalogLeftThrust_mut,
    PxVehicleDriveTankRawInputData_setAnalogRightThrust_mut,
    PxVehicleDriveTankRawInputData_setAnalogLeftBrake_mut,
    PxVehicleDriveTankRawInputData_setAnalogRightBrake_mut,
    PxVehicleDriveTankRawInputData_getAnalogAccel,
    PxVehicleDriveTankRawInputData_getAnalogLeftThrust,
    PxVehicleDriveTankRawInputData_getAnalogRightThrust,
    PxVehicleDriveTankRawInputData_getAnalogLeftBrake,
    PxVehicleDriveTankRawInputData_getAnalogRightBrake,
    PxVehicleDriveTankRawInputData_setGearUp_mut,
    PxVehicleDriveTankRawInputData_setGearDown_mut,
    PxVehicleDriveTankRawInputData_getGearUp,
    PxVehicleDriveTankRawInputData_getGearDown,
};

use super::VehicleDriveTankControlModel;

#[repr(transparent)]
#[derive(Clone)]
pub struct PxVehicleDriveTankRawInputData {
    obj: physx_sys::PxVehicleDriveTankRawInputData,
}

impl Drop for PxVehicleDriveTankRawInputData {
    fn drop(&mut self) {
        unsafe { PxVehicleDriveTankRawInputData_delete(self.as_mut_ptr()) }
    }
}

DeriveClassForNewType!(PxVehicleDriveTankRawInputData: PxVehicleDriveTankRawInputData);

impl<T> VehicleDriveTankRawInputData for T where T: Class<physx_sys::PxVehicleDriveTankRawInputData> {}

pub trait VehicleDriveTankRawInputData: Class<physx_sys::PxVehicleDriveTankRawInputData> + Sized {
    fn new() -> Option<Owner<Self>> {
        Self::new_with_mode(VehicleDriveTankControlModel::Standard)
    }

    fn new_with_mode(mode: VehicleDriveTankControlModel) -> Option<Owner<Self>> {
        unsafe {
            VehicleDriveTankRawInputData::from_raw(
                PxVehicleDriveTankRawInputData_new_alloc(mode.into())
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
        ptr: *mut physx_sys::PxVehicleDriveTankRawInputData,
    ) -> Option<Owner<Self>> {
        Owner::from_raw(ptr as *mut Self)
    }

    /// Return the drive model (eDRIVE_MODEL_SPECIAL or eDRIVE_MODEL_STANDARD).
    fn get_drive_model(&self) -> VehicleDriveTankControlModel {
        unsafe { PxVehicleDriveTankRawInputData_getDriveModel(self.as_ptr()).into() }
    }

    /// Set if the accel button has been pressed on the keyboard.
    fn set_digital_accel(&mut self, accel_key_pressed: bool) {
        unsafe { PxVehicleDriveTankRawInputData_setDigitalAccel_mut(self.as_mut_ptr(), accel_key_pressed) }
    }

    /// Set if the left thrust button has been pressed on the keyboard.
    fn set_digital_left_thrust(&mut self, left_thrust_pressed: bool) {
        unsafe { PxVehicleDriveTankRawInputData_setDigitalLeftThrust_mut(self.as_mut_ptr(), left_thrust_pressed) }
    }

    /// Set if the right thrust button has been pressed on the keyboard.
    fn set_digital_right_thrust(&mut self, right_thrust_pressed: bool) {
        unsafe { PxVehicleDriveTankRawInputData_setDigitalRightThrust_mut(self.as_mut_ptr(), right_thrust_pressed) }
    }

    /// Set if the left brake button has been pressed on the keyboard.
    fn set_digital_left_brake(&mut self, left_brake_pressed: bool) {
        unsafe { PxVehicleDriveTankRawInputData_setDigitalLeftBrake_mut(self.as_mut_ptr(), left_brake_pressed) }
    }

    /// Set if the right brake button has been pressed on the keyboard.
    fn set_digital_right_brake(&mut self, right_brake_pressed: bool) {
        unsafe { PxVehicleDriveTankRawInputData_setDigitalRightBrake_mut(self.as_mut_ptr(), right_brake_pressed) }
    }

    /// Return if the accel button has been pressed on the keyboard.
    fn get_digital_accel(&self) -> bool {
        unsafe { PxVehicleDriveTankRawInputData_getDigitalAccel(self.as_ptr()) }
    }

    /// Return if the left thrust button has been pressed on the keyboard.
    fn get_digital_left_thrust(&self) -> bool {
        unsafe { PxVehicleDriveTankRawInputData_getDigitalLeftThrust(self.as_ptr()) }
    }

    /// Return if the right thrust button has been pressed on the keyboard.
    fn get_digital_right_thrust(&self) -> bool {
        unsafe { PxVehicleDriveTankRawInputData_getDigitalRightThrust(self.as_ptr()) }
    }

    /// Return if the left brake button has been pressed on the keyboard.
    fn get_digital_left_brake(&self) -> bool {
        unsafe { PxVehicleDriveTankRawInputData_getDigitalLeftBrake(self.as_ptr()) }
    }

    /// Return if the right brake button has been pressed on the keyboard.
    fn get_digital_right_brake(&self) -> bool {
        unsafe { PxVehicleDriveTankRawInputData_getDigitalRightBrake(self.as_ptr()) }
    }

    /// Set the analog accel value from the gamepad.
    fn set_analog_accel(&mut self, accel: f32) {
        unsafe { PxVehicleDriveTankRawInputData_setAnalogAccel_mut(self.as_mut_ptr(), accel) }
    }

    /// Set the analog left thrust value from the gamepad.
    fn set_analog_left_thrust(&mut self, left_thrust: f32) {
        unsafe { PxVehicleDriveTankRawInputData_setAnalogLeftThrust_mut(self.as_mut_ptr(), left_thrust) }
    }

    /// Set the analog right thrust value from the gamepad.
    fn set_analog_right_thrust(&mut self, right_thrust: f32) {
        unsafe { PxVehicleDriveTankRawInputData_setAnalogRightThrust_mut(self.as_mut_ptr(), right_thrust) }
    }

    /// Set the analog left brake value from the gamepad.
    fn set_analog_left_brake(&mut self, left_brake: f32) {
        unsafe { PxVehicleDriveTankRawInputData_setAnalogLeftBrake_mut(self.as_mut_ptr(), left_brake) }
    }

    /// Set the analog right brake value from the gamepad.
    fn set_analog_right_brake(&mut self, right_brake: f32) {
        unsafe { PxVehicleDriveTankRawInputData_setAnalogRightBrake_mut(self.as_mut_ptr(), right_brake) }
    }

    /// Return the analog accel value from the gamepad.
    fn get_analog_accel(&self) -> f32 {
        unsafe { PxVehicleDriveTankRawInputData_getAnalogAccel(self.as_ptr()) }
    }

    /// Return the analog left thrust value from the gamepad.
    fn get_analog_left_thrust(&self) -> f32 {
        unsafe { PxVehicleDriveTankRawInputData_getAnalogLeftThrust(self.as_ptr()) }
    }

    /// Return the analog right thrust value from the gamepad.
    fn get_analog_right_thrust(&self) -> f32 {
        unsafe { PxVehicleDriveTankRawInputData_getAnalogRightThrust(self.as_ptr()) }
    }

    /// Return the analog left brake value from the gamepad.
    fn get_analog_left_brake(&self) -> f32 {
        unsafe { PxVehicleDriveTankRawInputData_getAnalogLeftBrake(self.as_ptr()) }
    }

    /// Return the analog right brake value from the gamepad.
    fn get_analog_right_brake(&self) -> f32 {
        unsafe { PxVehicleDriveTankRawInputData_getAnalogRightBrake(self.as_ptr()) }
    }

    /// Record if the gear-up button has been pressed on keyboard or gamepad.
    fn set_gear_up(&mut self, gear_up_key_pressed: bool) {
        unsafe { PxVehicleDriveTankRawInputData_setGearUp_mut(self.as_mut_ptr(), gear_up_key_pressed) }
    }

    /// Record if the gear-down button has been pressed on keyboard or gamepad.
    fn set_gear_down(&mut self, gear_down_key_pressed: bool) {
        unsafe { PxVehicleDriveTankRawInputData_setGearDown_mut(self.as_mut_ptr(), gear_down_key_pressed) }
    }

    /// Return if the gear-up button has been pressed on keyboard or gamepad.
    fn get_gear_up(&self) -> bool {
        unsafe { PxVehicleDriveTankRawInputData_getGearUp(self.as_ptr()) }
    }

    /// Return if the gear-down button has been pressed on keyboard or gamepad.
    fn get_gear_down(&self) -> bool {
        unsafe { PxVehicleDriveTankRawInputData_getGearDown(self.as_ptr()) }
    }
}
