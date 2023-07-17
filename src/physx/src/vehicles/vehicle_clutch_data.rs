use physx_sys::{
    PxVehicleClutchData,
    PxVehicleClutchData_new,
    //PxVehicleClutchData_new_1,
};

#[derive(Debug, Clone)]
pub struct VehicleClutchData {
    /// Strength of clutch.
    pub strength: f32,
    /// The engine and wheel rotation speeds that are coupled through the clutch can be updated by choosing one of two modes: eESTIMATE and eBEST_POSSIBLE.
    pub accuracy_mode: u32,
    /// Tune the mathematical accuracy and computational cost of the computed estimate to the wheel and engine rotation speeds if eESTIMATE is chosen.
    pub estimate_iterations: u32,
}

impl From<PxVehicleClutchData> for VehicleClutchData {
    fn from(value: PxVehicleClutchData) -> Self {
        Self {
            strength: value.mStrength,
            accuracy_mode: value.mAccuracyMode,
            estimate_iterations: value.mEstimateIterations,
        }
    }
}

impl From<VehicleClutchData> for PxVehicleClutchData {
    fn from(value: VehicleClutchData) -> Self {
        let mut result = unsafe { PxVehicleClutchData_new() };
        result.mStrength = value.strength;
        result.mAccuracyMode = value.accuracy_mode;
        result.mEstimateIterations = value.estimate_iterations;
        result
    }
}

impl Default for VehicleClutchData {
    fn default() -> Self {
        unsafe { PxVehicleClutchData_new() }.into()
    }
}
