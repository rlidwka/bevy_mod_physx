use physx_sys::{
    PxVehicleAckermannGeometryData,
    PxVehicleAckermannGeometryData_new,
    //PxVehicleAckermannGeometryData_new_1,
};

#[derive(Debug, Clone)]
pub struct VehicleAckermannGeometryData {
    /// Accuracy of Ackermann steer calculation.
    pub accuracy: f32,
    /// Distance between center-point of the two front wheels.
    pub front_width: f32,
    /// Distance between center-point of the two rear wheels.
    pub rear_width: f32,
    /// Distance between center of front axle and center of rear axle.
    pub axle_separation: f32,
}

impl From<PxVehicleAckermannGeometryData> for VehicleAckermannGeometryData {
    fn from(value: PxVehicleAckermannGeometryData) -> Self {
        Self {
            accuracy: value.mAccuracy,
            front_width: value.mFrontWidth,
            rear_width: value.mRearWidth,
            axle_separation: value.mAxleSeparation,
        }
    }
}

impl From<VehicleAckermannGeometryData> for PxVehicleAckermannGeometryData {
    fn from(value: VehicleAckermannGeometryData) -> Self {
        let mut result = unsafe { PxVehicleAckermannGeometryData_new() };
        result.mAccuracy = value.accuracy;
        result.mFrontWidth = value.front_width;
        result.mRearWidth = value.rear_width;
        result.mAxleSeparation = value.axle_separation;
        result
    }
}

impl Default for VehicleAckermannGeometryData {
    fn default() -> Self {
        unsafe { PxVehicleAckermannGeometryData_new() }.into()
    }
}
