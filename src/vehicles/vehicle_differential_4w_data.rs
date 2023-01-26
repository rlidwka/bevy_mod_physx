use physx_sys::{
    PxVehicleDifferential4WData,
    PxVehicleDifferential4WData_new,
    //PxVehicleDifferential4WData_new_1,
};

#[derive(Debug, Clone)]
pub struct VehicleDifferential4WData {
    /// Ratio of torque split between front and rear (>0.5 means more to front, <0.5 means more to rear).
    pub front_rear_split: f32,
    /// Ratio of torque split between front-left and front-right (>0.5 means more to front-left, <0.5 means more to front-right).
    pub front_left_right_split: f32,
    /// Ratio of torque split between rear-left and rear-right (>0.5 means more to rear-left, <0.5 means more to rear-right).
    pub rear_left_right_split: f32,
    /// Maximum allowed ratio of average front wheel rotation speed and rear wheel rotation speeds The differential will divert more torque to the slower wheels when the bias is exceeded.
    pub centre_bias: f32,
    /// Maximum allowed ratio of front-left and front-right wheel rotation speeds. The differential will divert more torque to the slower wheel when the bias is exceeded.
    pub front_bias: f32,
    /// Maximum allowed ratio of rear-left and rear-right wheel rotation speeds. The differential will divert more torque to the slower wheel when the bias is exceeded.
    pub rear_bias: f32,
    /// Type of differential.
    pub diff_type: VehicleDifferential4WType,
}

impl From<PxVehicleDifferential4WData> for VehicleDifferential4WData {
    fn from(value: PxVehicleDifferential4WData) -> Self {
        Self {
            front_rear_split: value.mFrontRearSplit,
            front_left_right_split: value.mFrontLeftRightSplit,
            rear_left_right_split: value.mRearLeftRightSplit,
            centre_bias: value.mCentreBias,
            front_bias: value.mFrontBias,
            rear_bias: value.mRearBias,
            diff_type: value.mType.into(),
        }
    }
}

impl From<VehicleDifferential4WData> for PxVehicleDifferential4WData {
    fn from(value: VehicleDifferential4WData) -> Self {
        let mut result = unsafe { PxVehicleDifferential4WData_new() };
        result.mFrontRearSplit = value.front_rear_split;
        result.mFrontLeftRightSplit = value.front_left_right_split;
        result.mRearLeftRightSplit = value.rear_left_right_split;
        result.mCentreBias = value.centre_bias;
        result.mFrontBias = value.front_bias;
        result.mRearBias = value.rear_bias;
        result.mType = value.diff_type.into();
        result
    }
}

impl Default for VehicleDifferential4WData {
    fn default() -> Self {
        unsafe { PxVehicleDifferential4WData_new() }.into()
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum VehicleDifferential4WType {
    LS4WD = 0,
    LSFrontWD = 1,
    LSRearWD = 2,
    Open4WD = 3,
    OpenFrontWD = 4,
    OpenRearWD = 5,
    //NbDiffTypes = 6,
}

impl VehicleDifferential4WType {
    pub const NB_DIFF_TYPES: u32 = 6;
}

impl From<VehicleDifferential4WType> for physx_sys::PxVehicleDifferential4WDataEnum::Enum {
    fn from(value: VehicleDifferential4WType) -> Self {
        value as u32
    }
}

impl From<physx_sys::PxVehicleDifferential4WDataEnum::Enum> for VehicleDifferential4WType {
    fn from(ty: physx_sys::PxVehicleDifferential4WDataEnum::Enum) -> Self {
        match ty {
            0 => Self::LS4WD,
            1 => Self::LSFrontWD,
            2 => Self::LSRearWD,
            3 => Self::Open4WD,
            4 => Self::OpenFrontWD,
            5 => Self::OpenRearWD,
            _ => panic!("invalid enum variant"),
        }
    }
}
