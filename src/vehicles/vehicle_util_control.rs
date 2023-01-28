use physx::DeriveClassForNewType;
use physx::traits::Class;
use physx_sys::{PxVehicleKeySmoothingData, PxVehiclePadSmoothingData, phys_PxVehicleDriveTankSmoothDigitalRawInputsAndSetAnalogInputs, phys_PxVehicleDriveTankSmoothAnalogRawInputsAndSetAnalogInputs};

use super::{VehicleDriveDynData, PxVehicleDriveDynData, VehicleDrive4W, VehicleDriveTank, VehicleDriveTankRawInputData, VehicleDriveNW};

#[derive(Clone)]
pub struct VehicleKeySmoothingData {
    obj: PxVehicleKeySmoothingData,
}

DeriveClassForNewType!(VehicleKeySmoothingData: PxVehicleKeySmoothingData);

impl VehicleKeySmoothingData {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_rise_rates(mut self, rise_rates: &[f32]) -> Self {
        self.obj.mRiseRates[..rise_rates.len()].copy_from_slice(rise_rates);
        self
    }

    pub fn set_fall_rates(mut self, fall_rates: &[f32]) -> Self {
        self.obj.mFallRates[..fall_rates.len()].copy_from_slice(fall_rates);
        self
    }
}

impl Default for VehicleKeySmoothingData {
    fn default() -> Self {
        Self {
            obj: PxVehicleKeySmoothingData {
                mRiseRates: [0.; PxVehicleDriveDynData::MAX_NB_ANALOG_INPUTS],
                mFallRates: [0.; PxVehicleDriveDynData::MAX_NB_ANALOG_INPUTS],
            }
        }
    }
}

#[derive(Clone)]
pub struct VehiclePadSmoothingData {
    obj: PxVehiclePadSmoothingData,
}

DeriveClassForNewType!(VehiclePadSmoothingData: PxVehiclePadSmoothingData);

impl VehiclePadSmoothingData {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_rise_rates(mut self, rise_rates: &[f32]) -> Self {
        self.obj.mRiseRates[..rise_rates.len()].copy_from_slice(rise_rates);
        self
    }

    pub fn set_fall_rates(mut self, fall_rates: &[f32]) -> Self {
        self.obj.mFallRates[..fall_rates.len()].copy_from_slice(fall_rates);
        self
    }
}

impl Default for VehiclePadSmoothingData {
    fn default() -> Self {
        Self {
            obj: PxVehiclePadSmoothingData {
                mRiseRates: [0.; PxVehicleDriveDynData::MAX_NB_ANALOG_INPUTS],
                mFallRates: [0.; PxVehicleDriveDynData::MAX_NB_ANALOG_INPUTS],
            }
        }
    }
}

impl<T: VehicleDriveTank> VehicleDriveTankControl for T {}

pub trait VehicleDriveTankControl: VehicleDriveTank {
    fn smooth_digital_raw_inputs_and_set_analog_inputs(
        &mut self,
        key_smoothing: &VehicleKeySmoothingData,
        raw_input_data: &impl VehicleDriveTankRawInputData,
        timestep: f32,
    ) {
        unsafe {
            phys_PxVehicleDriveTankSmoothDigitalRawInputsAndSetAnalogInputs(
                key_smoothing.as_ptr(),
                raw_input_data.as_ptr(),
                timestep,
                self.as_mut_ptr(),
            );
        }
    }

    fn smooth_analog_raw_inputs_and_set_analog_inputs(
        &mut self,
        pad_smoothing: &VehiclePadSmoothingData,
        raw_input_data: &impl VehicleDriveTankRawInputData,
        timestep: f32,
    ) {
        unsafe {
            phys_PxVehicleDriveTankSmoothAnalogRawInputsAndSetAnalogInputs(
                pad_smoothing.as_ptr(),
                raw_input_data.as_ptr(),
                timestep,
                self.as_mut_ptr(),
            );
        }
    }
}
