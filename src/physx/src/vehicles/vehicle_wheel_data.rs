use physx_sys::{
    PxVehicleWheelData,
    PxVehicleWheelData_new,
    //PxVehicleWheelData_getRecipRadius,
    //PxVehicleWheelData_getRecipMOI,
};

#[derive(Debug, Clone)]
pub struct VehicleWheelData {
    /// Radius of unit that includes metal wheel plus rubber tire.
    pub radius: f32,
    /// Maximum width of unit that includes wheel plus tire.
    pub width: f32,
    /// Mass of unit that includes wheel plus tire.
    pub mass: f32,
    /// Moment of inertia of unit that includes wheel plus tire about the rolling axis.
    pub moi: f32,
    /// Damping rate applied to wheel.
    pub damping_rate: f32,
    /// Max brake torque that can be applied to wheel.
    pub max_brake_torque: f32,
    /// Max handbrake torque that can be applied to wheel.
    pub max_hand_brake_torque: f32,
    /// Max steer angle that can be achieved by the wheel.
    pub max_steer: f32,
    /// Wheel toe angle. This value is ignored by PxVehicleDriveTank and PxVehicleNoDrive.
    pub toe_angle: f32,
}

impl From<PxVehicleWheelData> for VehicleWheelData {
    fn from(value: PxVehicleWheelData) -> Self {
        Self {
            radius: value.mRadius,
            width: value.mWidth,
            mass: value.mMass,
            moi: value.mMOI,
            damping_rate: value.mDampingRate,
            max_brake_torque: value.mMaxBrakeTorque,
            max_hand_brake_torque: value.mMaxBrakeTorque,
            max_steer: value.mMaxSteer,
            toe_angle: value.mToeAngle,
        }
    }
}

impl From<VehicleWheelData> for PxVehicleWheelData {
    fn from(value: VehicleWheelData) -> Self {
        let mut result = unsafe { PxVehicleWheelData_new() };
        result.mRadius = value.radius;
        result.mWidth = value.width;
        result.mMass = value.mass;
        result.mMOI = value.moi;
        result.mDampingRate = value.damping_rate;
        result.mMaxBrakeTorque = value.max_brake_torque;
        result.mMaxHandBrakeTorque = value.max_hand_brake_torque;
        result.mMaxSteer = value.max_steer;
        result.mToeAngle = value.toe_angle;
        result
    }
}

impl Default for VehicleWheelData {
    fn default() -> Self {
        unsafe { PxVehicleWheelData_new() }.into()
    }
}
