use physx_sys::{
    PxVehicleGearsData,
    PxVehicleGearsData_new,
    //PxVehicleGearsData_new_1,
    //PxVehicleGearsData_getGearRatio,
    //PxVehicleGearsData_setGearRatio_mut,
};

#[derive(Debug, Clone)]
pub struct VehicleGearsData {
    /// Gear ratios.
    pub ratios: [f32; VehicleGearsRatio::GEARS_RATIO_COUNT as usize],
    /// Gear ratio applied is mRatios[currentGear]*finalRatio.
    pub final_ratio: f32,
    /// Number of gears (including reverse and neutral).
    pub nb_ratios: u32,
    /// Time it takes to switch gear.
    pub switch_time: f32,
}

impl VehicleGearsData {
    pub fn get_gear_ratio(&self, gear: VehicleGearsRatio) -> f32 {
        self.ratios.get(gear as usize).copied().unwrap_or_default()
    }

    pub fn set_gear_ratio(&mut self, gear: VehicleGearsRatio, ratio: f32) {
        if let Some(x) = self.ratios.get_mut(gear as usize) {
            *x = ratio; // avoid crash on the GearRatioCount enum variant
        }
    }
}

impl From<PxVehicleGearsData> for VehicleGearsData {
    fn from(value: PxVehicleGearsData) -> Self {
        Self {
            ratios: value.mRatios,
            final_ratio: value.mFinalRatio,
            nb_ratios: value.mNbRatios,
            switch_time: value.mSwitchTime,
        }
    }
}

impl From<VehicleGearsData> for PxVehicleGearsData {
    fn from(value: VehicleGearsData) -> Self {
        let mut result = unsafe { PxVehicleGearsData_new() };
        result.mRatios = value.ratios;
        result.mFinalRatio = value.final_ratio;
        result.mNbRatios = value.nb_ratios;
        result.mSwitchTime = value.switch_time;
        result
    }
}

impl Default for VehicleGearsData {
    fn default() -> Self {
        unsafe { PxVehicleGearsData_new() }.into()
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum VehicleGearsRatio {
    Reverse = 0,
    Neutral = 1,
    First = 2,
    Second = 3,
    Third = 4,
    Fourth = 5,
    Fifth = 6,
    Sixth = 7,
    Seventh = 8,
    Eighth = 9,
    Ninth = 10,
    Tenth = 11,
    Eleventh = 12,
    Twelfth = 13,
    Thirteenth = 14,
    Fourteenth = 15,
    Fifteenth = 16,
    Sixteenth = 17,
    Seventeenth = 18,
    Eighteenth = 19,
    Nineteenth = 20,
    Twentieth = 21,
    TwentyFirst = 22,
    TwentySecond = 23,
    TwentyThird = 24,
    TwentyFourth = 25,
    TwentyFifth = 26,
    TwentySixth = 27,
    TwentySeventh = 28,
    TwentyEighth = 29,
    TwentyNinth = 30,
    Thirtieth = 31,
    //GearsRatioCount = 32,
}

impl VehicleGearsRatio {
    pub const GEARS_RATIO_COUNT: u32 = 32;
}

impl From<VehicleGearsRatio> for physx_sys::PxVehicleGearsDataEnum::Enum {
    fn from(value: VehicleGearsRatio) -> Self {
        value as u32
    }
}

impl From<physx_sys::PxVehicleGearsDataEnum::Enum> for VehicleGearsRatio {
    fn from(ty: physx_sys::PxVehicleGearsDataEnum::Enum) -> Self {
        match ty {
            0 => Self::Reverse,
            1 => Self::Neutral,
            2 => Self::First,
            3 => Self::Second,
            4 => Self::Third,
            5 => Self::Fourth,
            6 => Self::Fifth,
            7 => Self::Sixth,
            8 => Self::Seventh,
            9 => Self::Eighth,
            10 => Self::Ninth,
            11 => Self::Tenth,
            12 => Self::Eleventh,
            13 => Self::Twelfth,
            14 => Self::Thirteenth,
            15 => Self::Fourteenth,
            16 => Self::Fifteenth,
            17 => Self::Sixteenth,
            18 => Self::Seventeenth,
            19 => Self::Eighteenth,
            20 => Self::Nineteenth,
            21 => Self::Twentieth,
            22 => Self::TwentyFirst,
            23 => Self::TwentySecond,
            24 => Self::TwentyThird,
            25 => Self::TwentyFourth,
            26 => Self::TwentyFifth,
            27 => Self::TwentySixth,
            28 => Self::TwentySeventh,
            29 => Self::TwentyEighth,
            30 => Self::TwentyNinth,
            31 => Self::Thirtieth,
            _ => panic!("invalid enum variant"),
        }
    }
}
