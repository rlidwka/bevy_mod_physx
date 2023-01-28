use physx_sys::{
    PxVehicleAntiRollBarData,
    PxVehicleAntiRollBarData_new,
};

#[derive(Debug, Clone)]
pub struct VehicleAntiRollBarData {
    pub wheel0: u32,
    pub wheel1: u32,
    pub stiffness: f32,
}

impl From<PxVehicleAntiRollBarData> for VehicleAntiRollBarData {
    fn from(value: PxVehicleAntiRollBarData) -> Self {
        Self {
            wheel0: value.mWheel0,
            wheel1: value.mWheel1,
            stiffness: value.mStiffness,
        }
    }
}

impl From<VehicleAntiRollBarData> for PxVehicleAntiRollBarData {
    fn from(value: VehicleAntiRollBarData) -> Self {
        let mut result = unsafe { PxVehicleAntiRollBarData_new() };
        result.mWheel0 = value.wheel0;
        result.mWheel1 = value.wheel1;
        result.mStiffness = value.stiffness;
        result
    }
}

impl Default for VehicleAntiRollBarData {
    fn default() -> Self {
        unsafe { PxVehicleAntiRollBarData_new() }.into()
    }
}
