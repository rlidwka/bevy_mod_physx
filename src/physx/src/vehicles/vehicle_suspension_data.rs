use physx_sys::{
    PxVehicleSuspensionData,
    PxVehicleSuspensionData_new,
    //PxVehicleSuspensionData_getRecipMaxCompression,
    //PxVehicleSuspensionData_getRecipMaxDroop,
    //PxVehicleSuspensionData_setMassAndPreserveNaturalFrequency_mut,
};

#[derive(Debug, Clone)]
pub struct VehicleSuspensionData {
    /// Spring strength of suspension unit.
    pub spring_strength: f32,
    /// Spring damper rate of suspension unit.
    pub spring_damper_rate: f32,
    /// Maximum compression allowed by suspension spring.
    pub max_compression: f32,
    /// Maximum elongation allowed by suspension spring.
    pub max_droop: f32,
    /// Mass of vehicle that is supported by suspension spring.
    pub sprung_mass: f32,
    /// Camber angle (in radians) of wheel when the suspension is at its rest position.
    pub camber_at_rest: f32,
    /// Camber angle (in radians) of wheel when the suspension is at maximum compression.
    pub camber_at_max_compression: f32,
    /// Camber angle (in radians) of wheel when the suspension is at maximum droop.
    pub camber_at_max_droop: f32,
}

impl From<PxVehicleSuspensionData> for VehicleSuspensionData {
    fn from(value: PxVehicleSuspensionData) -> Self {
        Self {
            spring_strength: value.mSpringStrength,
            spring_damper_rate: value.mSpringDamperRate,
            max_compression: value.mMaxCompression,
            max_droop: value.mMaxDroop,
            sprung_mass: value.mSprungMass,
            camber_at_rest: value.mCamberAtRest,
            camber_at_max_compression: value.mCamberAtMaxCompression,
            camber_at_max_droop: value.mCamberAtMaxDroop,
        }
    }
}

impl From<VehicleSuspensionData> for PxVehicleSuspensionData {
    fn from(value: VehicleSuspensionData) -> Self {
        let mut result = unsafe { PxVehicleSuspensionData_new() };
        result.mSpringStrength = value.spring_strength;
        result.mSpringDamperRate = value.spring_damper_rate;
        result.mMaxCompression = value.max_compression;
        result.mMaxDroop = value.max_droop;
        result.mSprungMass = value.sprung_mass;
        result.mCamberAtRest = value.camber_at_rest;
        result.mCamberAtMaxCompression = value.camber_at_max_compression;
        result.mCamberAtMaxDroop = value.camber_at_max_droop;
        result
    }
}

impl Default for VehicleSuspensionData {
    fn default() -> Self {
        unsafe { PxVehicleSuspensionData_new() }.into()
    }
}
