use physx_sys::{
    PxVehicleTireLoadFilterData,
    PxVehicleTireLoadFilterData_new,
    //PxVehicleTireLoadFilterData_getDenominator,
    //PxVehicleTireLoadFilterData_new_1,
};

#[derive(Debug, Clone)]
pub struct VehicleTireLoadFilterData {
    /// Graph point (mMinNormalisedLoad,mMinFilteredNormalisedLoad).
    pub min_normalised_load: f32,
    /// Graph point (mMinNormalisedLoad,mMinFilteredNormalisedLoad).
    pub min_filtered_normalised_load: f32,
    /// Graph point (mMaxNormalisedLoad,mMaxFilteredNormalisedLoad).
    pub max_normalised_load: f32,
    /// Graph point (mMaxNormalisedLoad,mMaxFilteredNormalisedLoad).
    pub max_filtered_normalised_load: f32,
}

impl From<PxVehicleTireLoadFilterData> for VehicleTireLoadFilterData {
    fn from(value: PxVehicleTireLoadFilterData) -> Self {
        Self {
            min_normalised_load: value.mMinNormalisedLoad,
            min_filtered_normalised_load: value.mMinFilteredNormalisedLoad,
            max_normalised_load: value.mMaxNormalisedLoad,
            max_filtered_normalised_load: value.mMaxFilteredNormalisedLoad,
        }
    }
}

impl From<VehicleTireLoadFilterData> for PxVehicleTireLoadFilterData {
    fn from(value: VehicleTireLoadFilterData) -> Self {
        let mut result = unsafe { PxVehicleTireLoadFilterData_new() };
        result.mMinNormalisedLoad = value.min_normalised_load;
        result.mMinFilteredNormalisedLoad = value.min_filtered_normalised_load;
        result.mMaxNormalisedLoad = value.max_normalised_load;
        result.mMaxFilteredNormalisedLoad = value.max_filtered_normalised_load;
        result
    }
}

impl Default for VehicleTireLoadFilterData {
    fn default() -> Self {
        unsafe { PxVehicleTireLoadFilterData_new() }.into()
    }
}
