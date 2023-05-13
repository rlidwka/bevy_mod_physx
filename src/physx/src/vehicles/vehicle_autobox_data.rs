use physx_sys::{
    PxVehicleAutoBoxData,
    PxVehicleAutoBoxData_new,
    //PxVehicleAutoBoxData_setLatency_mut,
    //PxVehicleAutoBoxData_getLatency,
    //PxVehicleAutoBoxData_new_1,
    //PxVehicleAutoBoxData_getUpRatios,
    //PxVehicleAutoBoxData_setUpRatios_mut,
    //PxVehicleAutoBoxData_getDownRatios,
    //PxVehicleAutoBoxData_setDownRatios_mut,
};

use super::VehicleGearsRatio;

#[derive(Debug, Clone)]
pub struct VehicleAutoBoxData {
    /// Value of ( engineRotationSpeed / PxVehicleEngineData::mMaxOmega ) that is high enough to increment gear.
    pub up_ratios: [f32; VehicleGearsRatio::GEARS_RATIO_COUNT as usize],
    /// Value of engineRevs/maxEngineRevs that is low enough to decrement gear.
    pub down_ratios: [f32; VehicleGearsRatio::GEARS_RATIO_COUNT as usize],
}

impl VehicleAutoBoxData {
    pub fn set_latency(&mut self, latency: f32) {
        self.down_ratios[VehicleGearsRatio::Reverse as usize] = latency;
    }

    pub fn get_latency(&self) -> f32 {
        self.down_ratios[VehicleGearsRatio::Reverse as usize]
    }

    pub fn get_up_ratios(&self, gear: VehicleGearsRatio) -> f32 {
        self.up_ratios.get(gear as usize).copied().unwrap_or_default()
    }

    pub fn set_up_ratios(&mut self, gear: VehicleGearsRatio, ratio: f32) {
        if let Some(x) = self.up_ratios.get_mut(gear as usize) {
            *x = ratio; // avoid crash on the GearRatioCount enum variant
        }
    }

    pub fn get_down_ratios(&self, gear: VehicleGearsRatio) -> f32 {
        self.down_ratios.get(gear as usize).copied().unwrap_or_default()
    }

    pub fn set_down_ratios(&mut self, gear: VehicleGearsRatio, ratio: f32) {
        if let Some(x) = self.down_ratios.get_mut(gear as usize) {
            *x = ratio; // avoid crash on the GearRatioCount enum variant
        }
    }
}

impl From<PxVehicleAutoBoxData> for VehicleAutoBoxData {
    fn from(value: PxVehicleAutoBoxData) -> Self {
        Self {
            up_ratios: value.mUpRatios,
            down_ratios: value.mDownRatios,
        }
    }
}

impl From<VehicleAutoBoxData> for PxVehicleAutoBoxData {
    fn from(value: VehicleAutoBoxData) -> Self {
        let mut result = unsafe { PxVehicleAutoBoxData_new() };
        result.mUpRatios = value.up_ratios;
        result.mDownRatios = value.down_ratios;
        result
    }
}

impl Default for VehicleAutoBoxData {
    fn default() -> Self {
        unsafe { PxVehicleAutoBoxData_new() }.into()
    }
}
