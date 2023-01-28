use crate::{
    DeriveClassForNewType,
    traits::Class,
};

use physx_sys::{
    PxVehicleDriveDynData_setToRestState_mut,
    PxVehicleDriveDynData_setAnalogInput_mut,
    PxVehicleDriveDynData_getAnalogInput,
    PxVehicleDriveDynData_setGearUp_mut,
    PxVehicleDriveDynData_setGearDown_mut,
    PxVehicleDriveDynData_getGearUp,
    PxVehicleDriveDynData_getGearDown,
    PxVehicleDriveDynData_setUseAutoGears_mut,
    PxVehicleDriveDynData_getUseAutoGears,
    PxVehicleDriveDynData_toggleAutoGears_mut,
    PxVehicleDriveDynData_setCurrentGear_mut,
    PxVehicleDriveDynData_getCurrentGear,
    PxVehicleDriveDynData_setTargetGear_mut,
    PxVehicleDriveDynData_getTargetGear,
    PxVehicleDriveDynData_startGearChange_mut,
    PxVehicleDriveDynData_forceGearChange_mut,
    PxVehicleDriveDynData_setEngineRotationSpeed_mut,
    PxVehicleDriveDynData_getEngineRotationSpeed,
    PxVehicleDriveDynData_getGearSwitchTime,
    PxVehicleDriveDynData_getAutoBoxSwitchTime,
    PxVehicleDriveDynData_new,
    //PxVehicleDriveDynData_new_1,
    PxVehicleDriveDynData_getNbAnalogInput,
    PxVehicleDriveDynData_setGearChange_mut,
    PxVehicleDriveDynData_getGearChange,
    PxVehicleDriveDynData_setGearSwitchTime_mut,
    PxVehicleDriveDynData_setAutoBoxSwitchTime_mut,
};

use super::VehicleGearsRatio;

#[repr(transparent)]
#[derive(Clone)]
pub struct PxVehicleDriveDynData {
    obj: physx_sys::PxVehicleDriveDynData,
}

impl Default for PxVehicleDriveDynData {
    fn default() -> Self {
        Self { obj: unsafe { PxVehicleDriveDynData_new() } }
    }
}

DeriveClassForNewType!(PxVehicleDriveDynData: PxVehicleDriveDynData);

impl<T> VehicleDriveDynData for T where T: Class<physx_sys::PxVehicleDriveDynData> {}

pub trait VehicleDriveDynData: Class<physx_sys::PxVehicleDriveDynData> {
    const MAX_NB_ANALOG_INPUTS: usize = 16;

    /// Set all dynamics data to zero to bring the vehicle to rest.
    fn set_to_rest_state(&mut self) {
        unsafe { PxVehicleDriveDynData_setToRestState_mut(self.as_mut_ptr()) }
    }

    /// Set an analog control value to drive the vehicle.
    fn set_analog_input(&mut self, ctype: impl VehicleDriveControlType, analog_val: f32) {
        unsafe { PxVehicleDriveDynData_setAnalogInput_mut(self.as_mut_ptr(), ctype.into(), analog_val) }
    }

    /// Get the analog control value that has been applied to the vehicle.
    fn get_analog_input(&self, ctype: impl VehicleDriveControlType) -> f32 {
        unsafe { PxVehicleDriveDynData_getAnalogInput(self.as_ptr(), ctype.into()) }
    }

    /// Inform the vehicle that the gear-up button has been pressed.
    fn set_gear_up(&mut self, digital_val: bool) {
        unsafe { PxVehicleDriveDynData_setGearUp_mut(self.as_mut_ptr(), digital_val) }
    }

    /// Set that the gear-down button has been pressed.
    fn set_gear_down(&mut self, digital_val: bool) {
        unsafe { PxVehicleDriveDynData_setGearDown_mut(self.as_mut_ptr(), digital_val) }
    }

    /// Check if the gear-up button has been pressed.
    fn get_gear_up(&mut self) -> bool {
        unsafe { PxVehicleDriveDynData_getGearUp(self.as_ptr()) }
    }

    /// Check if the gear-down button has been pressed.
    fn get_gear_down(&mut self) -> bool {
        unsafe { PxVehicleDriveDynData_getGearDown(self.as_ptr()) }
    }

    /// Set the flag that will be used to select auto-gears If useAutoGears is true the auto-box will be active.
    fn set_use_auto_gears(&mut self, use_auto_gears: bool) {
        unsafe { PxVehicleDriveDynData_setUseAutoGears_mut(self.as_mut_ptr(), use_auto_gears) }
    }

    /// Get the flag status that is used to select auto-gears.
    fn get_use_auto_gears(&mut self) -> bool {
        unsafe { PxVehicleDriveDynData_getUseAutoGears(self.as_ptr()) }
    }

    /// Toggle the auto-gears flag If useAutoGears is true the auto-box will be active.
    fn toggle_auto_gears(&mut self) {
        unsafe { PxVehicleDriveDynData_toggleAutoGears_mut(self.as_mut_ptr()) }
    }

    /// Set the current gear.
    fn set_current_gear(&mut self, current_gear: VehicleGearsRatio) {
        unsafe { PxVehicleDriveDynData_setCurrentGear_mut(self.as_mut_ptr(), current_gear.into()) }
    }

    /// Get the current gear.
    fn get_current_gear(&self) -> VehicleGearsRatio {
        unsafe { PxVehicleDriveDynData_getCurrentGear(self.as_ptr()).into() }
    }

    /// Set the target gear.
    fn set_target_gear(&mut self, target_gear: VehicleGearsRatio) {
        unsafe { PxVehicleDriveDynData_setTargetGear_mut(self.as_mut_ptr(), target_gear.into()) }
    }

    /// Get the target gear.
    fn get_target_gear(&self) -> VehicleGearsRatio {
        unsafe { PxVehicleDriveDynData_getTargetGear(self.as_ptr()).into() }
    }

    /// Start a gear change to a target gear.
    fn start_gear_change(&mut self, target_gear: VehicleGearsRatio) {
        unsafe { PxVehicleDriveDynData_startGearChange_mut(self.as_mut_ptr(), target_gear.into()) }
    }

    /// Force an immediate gear change to a target gear.
    fn force_gear_change(&mut self, target_gear: VehicleGearsRatio) {
        unsafe { PxVehicleDriveDynData_forceGearChange_mut(self.as_mut_ptr(), target_gear.into()) }
    }

    /// Set the rotation speed of the engine (radians per second).
    fn set_engine_rotation_speed(&mut self, speed: f32) {
        unsafe { PxVehicleDriveDynData_setEngineRotationSpeed_mut(self.as_mut_ptr(), speed) }
    }

    /// Return the rotation speed of the engine (radians per second).
    fn get_engine_rotation_speed(&self) -> f32 {
        unsafe { PxVehicleDriveDynData_getEngineRotationSpeed(self.as_ptr()) }
    }

    /// Return the time that has passed since the current gear change was initiated.
    fn get_gear_switch_time(&self) -> f32 {
        unsafe { PxVehicleDriveDynData_getGearSwitchTime(self.as_ptr()) }
    }

    /// Return the time that has passed since the autobox last initiated a gear change.
    fn get_autobox_switch_time(&self) -> f32 {
        unsafe { PxVehicleDriveDynData_getAutoBoxSwitchTime(self.as_ptr()) }
    }

    fn get_nb_analog_input(&self) -> u32 {
        unsafe { PxVehicleDriveDynData_getNbAnalogInput(self.as_ptr()) }
    }

    fn set_gear_change(&mut self, gear_change: VehicleGearsRatio) {
        unsafe { PxVehicleDriveDynData_setGearChange_mut(self.as_mut_ptr(), gear_change.into()) }
    }

    fn get_gear_change(&self) -> VehicleGearsRatio {
        unsafe { PxVehicleDriveDynData_getGearChange(self.as_ptr()).into() }
    }

    fn set_gear_switch_time(&mut self, switch_time: f32) {
        unsafe { PxVehicleDriveDynData_setGearSwitchTime_mut(self.as_mut_ptr(), switch_time) }
    }

    fn set_autobox_switch_time(&mut self, autobox_switch_time: f32) {
        unsafe { PxVehicleDriveDynData_setAutoBoxSwitchTime_mut(self.as_mut_ptr(), autobox_switch_time) }
    }
}

pub trait VehicleDriveControlType : Into<u32> {}
