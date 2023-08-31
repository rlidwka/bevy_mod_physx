use physx_sys::{
    PxVehicleEngineData,
    PxFixedSizeLookupTable_eMAX_NB_ENGINE_TORQUE_CURVE_ENTRIES_,
    PxVehicleEngineData_new,
    //PxVehicleEngineData_getRecipMOI,
    //PxVehicleEngineData_getRecipMaxOmega,
    //PxVehicleEngineData_new_1,
    //PxVehicleEngineData_delete,
};

#[derive(Debug, Clone)]
pub struct VehicleEngineDataTorqueCurve {
    pub data_pairs: [(f32, f32); 8],
    pub nb_data_pairs: u32,
}

impl From<PxFixedSizeLookupTable_eMAX_NB_ENGINE_TORQUE_CURVE_ENTRIES_> for VehicleEngineDataTorqueCurve {
    fn from(value: PxFixedSizeLookupTable_eMAX_NB_ENGINE_TORQUE_CURVE_ENTRIES_) -> Self {
        Self {
            // SAFETY: [(f32, f32); X] are the same bytes as [f32; 2*X]
            data_pairs: unsafe { std::mem::transmute(value.mDataPairs) },
            nb_data_pairs: value.mNbDataPairs,
        }
    }
}

impl From<VehicleEngineDataTorqueCurve> for PxFixedSizeLookupTable_eMAX_NB_ENGINE_TORQUE_CURVE_ENTRIES_ {
    fn from(value: VehicleEngineDataTorqueCurve) -> Self {
        Self {
            // SAFETY: [(f32, f32); X] are the same bytes as [f32; 2*X]
            mDataPairs: unsafe { std::mem::transmute(value.data_pairs) },
            mNbDataPairs: value.nb_data_pairs,
            mPad: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VehicleEngineData {
    /// Graph of normalized torque (torque/mPeakTorque) against normalized engine speed ( engineRotationSpeed / mMaxOmega ).
    pub torque_curve: VehicleEngineDataTorqueCurve,
    /// Moment of inertia of the engine around the axis of rotation.
    pub moi: f32,
    /// Maximum torque available to apply to the engine when the accelerator pedal is at maximum.
    pub peak_torque: f32,
    /// Maximum rotation speed of the engine.
    pub max_omega: f32,
    /// Damping rate of engine when full throttle is applied.
    pub damping_rate_full_throttle: f32,
    /// Damping rate of engine when full throttle is applied.
    pub damping_rate_zero_throttle_clutch_engaged: f32,
    /// Damping rate of engine when full throttle is applied.
    pub damping_rate_zero_throttle_clutch_disengaged: f32,
}

impl From<PxVehicleEngineData> for VehicleEngineData {
    fn from(value: PxVehicleEngineData) -> Self {
        Self {
            torque_curve: value.mTorqueCurve.into(),
            moi: value.mMOI,
            peak_torque: value.mPeakTorque,
            max_omega: value.mMaxOmega,
            damping_rate_full_throttle: value.mDampingRateFullThrottle,
            damping_rate_zero_throttle_clutch_engaged: value.mDampingRateZeroThrottleClutchEngaged,
            damping_rate_zero_throttle_clutch_disengaged: value.mDampingRateZeroThrottleClutchDisengaged,
        }
    }
}

impl From<VehicleEngineData> for PxVehicleEngineData {
    fn from(value: VehicleEngineData) -> Self {
        let mut result = unsafe { PxVehicleEngineData_new() };
        result.mTorqueCurve = value.torque_curve.into();
        result.mMOI = value.moi;
        result.mPeakTorque = value.peak_torque;
        result.mMaxOmega = value.max_omega;
        result.mDampingRateFullThrottle = value.damping_rate_full_throttle;
        result.mDampingRateZeroThrottleClutchEngaged = value.damping_rate_zero_throttle_clutch_engaged;
        result.mDampingRateZeroThrottleClutchDisengaged = value.damping_rate_zero_throttle_clutch_disengaged;
        result
    }
}

impl Default for VehicleEngineData {
    fn default() -> Self {
        unsafe { PxVehicleEngineData_new() }.into()
    }
}
