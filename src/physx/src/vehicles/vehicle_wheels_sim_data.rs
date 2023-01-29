use crate::{
    DeriveClassForNewType,
    math::PxVec3,
    owner::Owner,
    traits::Class,
};

use physx_sys::{
    PxVehicleAntiRollBarData,
    PxVehicleSuspensionData,
    PxVehicleTireData,
    PxVehicleTireLoadFilterData,
    PxVehicleWheelData,
    PxVehicleWheelsSimData_allocate_mut,
    PxVehicleWheelsSimData_setChassisMass_mut,
    PxVehicleWheelsSimData_free_mut,
    PxVehicleWheelsSimData_copy_mut,
    PxVehicleWheelsSimData_getNbWheels,
    PxVehicleWheelsSimData_getSuspensionData,
    PxVehicleWheelsSimData_getWheelData,
    PxVehicleWheelsSimData_getTireData,
    PxVehicleWheelsSimData_getSuspTravelDirection,
    PxVehicleWheelsSimData_getSuspForceAppPointOffset,
    PxVehicleWheelsSimData_getTireForceAppPointOffset,
    PxVehicleWheelsSimData_getWheelCentreOffset,
    PxVehicleWheelsSimData_getWheelShapeMapping,
    //PxVehicleWheelsSimData_getSceneQueryFilterData,
    PxVehicleWheelsSimData_getNbAntiRollBars,
    PxVehicleWheelsSimData_getAntiRollBarData,
    PxVehicleWheelsSimData_getTireLoadFilterData,
    PxVehicleWheelsSimData_setSuspensionData_mut,
    PxVehicleWheelsSimData_setWheelData_mut,
    PxVehicleWheelsSimData_setTireData_mut,
    PxVehicleWheelsSimData_setSuspTravelDirection_mut,
    PxVehicleWheelsSimData_setSuspForceAppPointOffset_mut,
    PxVehicleWheelsSimData_setTireForceAppPointOffset_mut,
    PxVehicleWheelsSimData_setWheelCentreOffset_mut,
    PxVehicleWheelsSimData_setWheelShapeMapping_mut,
    //PxVehicleWheelsSimData_setSceneQueryFilterData_mut,
    PxVehicleWheelsSimData_setTireLoadFilterData_mut,
    PxVehicleWheelsSimData_addAntiRollBarData_mut,
    PxVehicleWheelsSimData_disableWheel_mut,
    PxVehicleWheelsSimData_enableWheel_mut,
    PxVehicleWheelsSimData_getIsWheelDisabled,
    PxVehicleWheelsSimData_setSubStepCount_mut,
    PxVehicleWheelsSimData_setMinLongSlipDenominator_mut,
    //PxVehicleWheelsSimData_setFlags_mut,
    //PxVehicleWheelsSimData_getFlags,
    //PxVehicleWheelsSimData_new_alloc,
    //PxVehicleWheelsSimData_getBinaryMetaData_mut,
    PxVehicleWheelsSimData_getNbWheels4,
    PxVehicleWheelsSimData_getNbSuspensionData,
    PxVehicleWheelsSimData_getNbWheelData,
    PxVehicleWheelsSimData_getNbSuspTravelDirection,
    PxVehicleWheelsSimData_getNbTireData,
    PxVehicleWheelsSimData_getNbSuspForceAppPointOffset,
    PxVehicleWheelsSimData_getNbTireForceAppPointOffset,
    PxVehicleWheelsSimData_getNbWheelCentreOffset,
    PxVehicleWheelsSimData_getNbWheelShapeMapping,
    //PxVehicleWheelsSimData_getNbSceneQueryFilterData,
    PxVehicleWheelsSimData_getMinLongSlipDenominator,
    PxVehicleWheelsSimData_setThresholdLongSpeed_mut,
    PxVehicleWheelsSimData_getThresholdLongSpeed,
    PxVehicleWheelsSimData_setLowForwardSpeedSubStepCount_mut,
    PxVehicleWheelsSimData_getLowForwardSpeedSubStepCount,
    PxVehicleWheelsSimData_setHighForwardSpeedSubStepCount_mut,
    PxVehicleWheelsSimData_getHighForwardSpeedSubStepCount,
    PxVehicleWheelsSimData_setWheelEnabledState_mut,
    PxVehicleWheelsSimData_getWheelEnabledState,
    PxVehicleWheelsSimData_getNbWheelEnabledState,
    PxVehicleWheelsSimData_getNbAntiRollBars4,
    PxVehicleWheelsSimData_getNbAntiRollBarData,
    PxVehicleWheelsSimData_setAntiRollBarData_mut,
    //PxVehicleWheelsSimData_new_alloc_1,
    //PxVehicleWheelsSimData_delete,
};

use super::{
    VehicleAntiRollBarData,
    VehicleSuspensionData,
    VehicleTireData,
    VehicleTireLoadFilterData,
    VehicleWheelData,
};

#[repr(transparent)]
pub struct VehicleWheelsSimData {
    obj: physx_sys::PxVehicleWheelsSimData,
}

DeriveClassForNewType!(VehicleWheelsSimData: PxVehicleWheelsSimData);

impl VehicleWheelsSimData {
    /// Create a PxVehicleWheelsSimData instance with nbWheels.
    pub fn new(nb_wheels: u32) -> Option<Owner<Self>> {
        unsafe {
            VehicleWheelsSimData::from_raw(
                PxVehicleWheelsSimData_allocate_mut(nb_wheels)
            )
        }
    }

    /// Create a new Owner wrapper around a raw pointer.
    /// # Safety
    /// Owner's own the pointer they wrap, using the pointer after dropping the Owner,
    /// or creating multiple Owners from the same pointer will cause UB.  Use `into_ptr` to
    /// retrieve the pointer and consume the Owner without dropping the pointee.
    /// Initializes user data.
    unsafe fn from_raw(
        ptr: *mut physx_sys::PxVehicleWheelsSimData,
    ) -> Option<Owner<Self>> {
        Owner::from_raw(ptr as *mut Self)
    }

    /// Setup with mass information that can be applied to the default values of the suspensions, wheels, and tires set in their respective constructors.
    pub fn set_chassis_mass(&mut self, chassis_mass: f32) {
        unsafe { PxVehicleWheelsSimData_setChassisMass_mut(self.as_mut_ptr(), chassis_mass) }
    }

    /// Copy the data of a single wheel unit (wheel, suspension, tire) from srcWheel of src to trgWheel.
    pub fn copy(&mut self, src: &Self, src_wheel: u32, trg_wheel: u32) {
        unsafe { PxVehicleWheelsSimData_copy_mut(self.as_mut_ptr(), src.as_ptr(), src_wheel, trg_wheel) }
    }

    /// Return the number of wheels.
    pub fn get_nb_wheels(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getNbWheels(self.as_ptr()) }
    }

    /// Return the suspension data of the idth wheel.
    pub fn get_suspension_data(&self, id: u32) -> VehicleSuspensionData {
        unsafe { (*PxVehicleWheelsSimData_getSuspensionData(self.as_ptr(), id)).into() }
    }

    /// Return the wheel data of the idth wheel.
    pub fn get_wheel_data(&self, id: u32) -> VehicleWheelData {
        unsafe { (*PxVehicleWheelsSimData_getWheelData(self.as_ptr(), id)).into() }
    }

    /// Return the tire data of the idth wheel.
    pub fn get_tire_data(&self, id: u32) -> VehicleTireData {
        unsafe { (*PxVehicleWheelsSimData_getTireData(self.as_ptr(), id)).into() }
    }

    /// Return the direction of travel of the suspension of the idth wheel.
    pub fn get_susp_travel_direction(&self, id: u32) -> PxVec3 {
        unsafe { (*PxVehicleWheelsSimData_getSuspTravelDirection(self.as_ptr(), id)).into() }
    }

    /// Return the application point of the suspension force of the suspension of the idth wheel as an offset from the rigid body center of mass.
    pub fn get_susp_force_app_point_offset(&self, id: u32) -> PxVec3 {
        unsafe { (*PxVehicleWheelsSimData_getSuspForceAppPointOffset(self.as_ptr(), id)).into() }
    }

    /// Return the application point of the tire force of the tire of the idth wheel as an offset from the rigid body center of mass.
    pub fn get_tire_force_app_point_offset(&self, id: u32) -> PxVec3 {
        unsafe { (*PxVehicleWheelsSimData_getTireForceAppPointOffset(self.as_ptr(), id)).into() }
    }

    /// Return the offset from the rigid body centre of mass to the centre of the idth wheel.
    pub fn get_wheel_centre_offset(&self, id: u32) -> PxVec3 {
        unsafe { (*PxVehicleWheelsSimData_getWheelCentreOffset(self.as_ptr(), id)).into() }
    }

    /// Return the wheel mapping for the ith wheel.
    pub fn get_wheel_shape_mapping(&self, wheel_id: u32) -> i32 {
        unsafe { PxVehicleWheelsSimData_getWheelShapeMapping(self.as_ptr(), wheel_id) }
    }

    /// Return the scene query filter data used by the specified suspension line.
    /*pub fn get_scene_query_filter_data(&self, susp_id: u32) -> PxFilterData {
        unsafe { PxVehicleWheelsSimData_getSceneQueryFilterData(self.as_ptr(), wheel_id) }
    }*/

    /// Return the number of unique anti-roll bars that have been added with addAntiRollBarData.
    pub fn get_nb_anti_roll_bars(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getNbAntiRollBars(self.as_ptr()) }
    }

    /// Return the number of unique anti-roll bars that have been added with addAntiRollBarData.
    pub fn get_anti_roll_bar_data(&self, anti_roll_id: u32) -> VehicleAntiRollBarData {
        unsafe { (*PxVehicleWheelsSimData_getAntiRollBarData(self.as_ptr(), anti_roll_id)).into() }
    }

    /// Return the data that describes the filtering of the tire load to produce smoother handling at large time-steps.
    pub fn get_tire_load_filter_data(&self) -> VehicleTireLoadFilterData {
        unsafe { (*PxVehicleWheelsSimData_getTireLoadFilterData(self.as_ptr())).into() }
    }

    /// Set the suspension data of the idth wheel.
    pub fn set_suspension_data(&mut self, id: u32, susp: VehicleSuspensionData) {
        let susp: PxVehicleSuspensionData = susp.into();
        unsafe { PxVehicleWheelsSimData_setSuspensionData_mut(self.as_mut_ptr(), id, &susp as *const _) }
    }

    /// Set the wheel data of the idth wheel.
    pub fn set_wheel_data(&mut self, id: u32, wheel: VehicleWheelData) {
        let wheel: PxVehicleWheelData = wheel.into();
        unsafe { PxVehicleWheelsSimData_setWheelData_mut(self.as_mut_ptr(), id, &wheel as *const _) }
    }

    /// Set the tire data of the idth wheel.
    pub fn set_tire_data(&mut self, id: u32, tire: VehicleTireData) {
        let tire: PxVehicleTireData = tire.into();
        unsafe { PxVehicleWheelsSimData_setTireData_mut(self.as_mut_ptr(), id, &tire as *const _) }
    }

    /// Set the direction of travel of the suspension of the idth wheel.
    pub fn set_susp_travel_direction(&mut self, id: u32, dir: PxVec3) {
        unsafe { PxVehicleWheelsSimData_setSuspTravelDirection_mut(self.as_mut_ptr(), id, dir.as_ptr()) }
    }

    /// Set the application point of the suspension force of the suspension of the idth wheel.
    pub fn set_susp_force_app_point_offset(&mut self, id: u32, offset: PxVec3) {
        unsafe { PxVehicleWheelsSimData_setSuspForceAppPointOffset_mut(self.as_mut_ptr(), id, offset.as_ptr()) }
    }

    /// Set the application point of the tire force of the tire of the idth wheel.
    pub fn set_tire_force_app_point_offset(&mut self, id: u32, offset: PxVec3) {
        unsafe { PxVehicleWheelsSimData_setTireForceAppPointOffset_mut(self.as_mut_ptr(), id, offset.as_ptr()) }
    }

    /// Set the offset from the rigid body centre of mass to the centre of the idth wheel.
    pub fn set_wheel_centre_offset(&mut self, id: u32, offset: PxVec3) {
        unsafe { PxVehicleWheelsSimData_setWheelCentreOffset_mut(self.as_mut_ptr(), id, offset.as_ptr()) }
    }

    /// Set mapping between wheel id and position of corresponding wheel shape in the list of actor shapes.
    pub fn set_wheel_shape_mapping(&mut self, wheel_id: u32, shape_id: i32) {
        unsafe { PxVehicleWheelsSimData_setWheelShapeMapping_mut(self.as_mut_ptr(), wheel_id, shape_id) }
    }

    /// Set the scene query filter data that will be used for raycasts along the travel direction of the specified suspension. The default value is PxFilterData(0,0,0,0).
    /*pub fn set_scene_query_filter_data(&mut self, susp_id: u32, sq_filter_data: PxFilterData) {
        unsafe { PxVehicleWheelsSimData_setSceneQueryFilterData_mut(self.as_mut_ptr(), susp_id, sq_filter_data) }
    }*/

    /// Set the data that describes the filtering of the tire load to produce smoother handling at large timesteps.
    pub fn set_tire_load_filter_data(&mut self, tire_load_filter: VehicleTireLoadFilterData) {
        let tire_load_filter: PxVehicleTireLoadFilterData = tire_load_filter.into();
        unsafe { PxVehicleWheelsSimData_setTireLoadFilterData_mut(self.as_mut_ptr(), &tire_load_filter as *const _) }
    }

    /// Set the anti-roll suspension for a pair of wheels.
    pub fn add_anti_roll_bar_data(&mut self, anti_roll: VehicleAntiRollBarData) -> u32 {
        let anti_roll: PxVehicleAntiRollBarData = anti_roll.into();
        unsafe { PxVehicleWheelsSimData_addAntiRollBarData_mut(self.as_mut_ptr(), &anti_roll as *const _) }
    }

    /// Disable a wheel so that zero suspension forces and zero tire forces are applied to the rigid body from this wheel.
    pub fn disable_wheel(&mut self, wheel: u32) {
        unsafe { PxVehicleWheelsSimData_disableWheel_mut(self.as_mut_ptr(), wheel) }
    }

    /// Enable a wheel so that suspension forces and tire forces are applied to the rigid body. All wheels are enabled by default and remain enabled until they are disabled.
    pub fn enable_wheel(&mut self, wheel: u32) {
        unsafe { PxVehicleWheelsSimData_enableWheel_mut(self.as_mut_ptr(), wheel) }
    }

    /// Test if a wheel has been disabled.
    pub fn get_is_wheel_disabled(&mut self, wheel: u32) -> bool {
        unsafe { PxVehicleWheelsSimData_getIsWheelDisabled(self.as_mut_ptr(), wheel) }
    }

    /// Set the number of vehicle sub-steps that will be performed when the vehicle's longitudinal speed is below and above a threshold longitudinal speed.
    pub fn set_sub_step_count(&mut self, threshold_longitudinal_speed: f32, low_forward_speed_sub_step_count: u32, high_forward_speed_sub_step_count: u32) {
        unsafe { PxVehicleWheelsSimData_setSubStepCount_mut(self.as_mut_ptr(), threshold_longitudinal_speed, low_forward_speed_sub_step_count, high_forward_speed_sub_step_count) }
    }

    /// Set the minimum denominator used in the longitudinal slip calculation.
    pub fn set_min_long_slip_denominator(&mut self, min_long_slip_denominator: f32) {
        unsafe { PxVehicleWheelsSimData_setMinLongSlipDenominator_mut(self.as_mut_ptr(), min_long_slip_denominator) }
    }

    pub fn get_nb_wheels_4(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getNbWheels4(self.as_ptr()) }
    }

    pub fn get_nb_suspension_data(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getNbSuspensionData(self.as_ptr()) }
    }

    pub fn get_nb_wheel_data(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getNbWheelData(self.as_ptr()) }
    }

    pub fn get_nb_susp_travel_direction(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getNbSuspTravelDirection(self.as_ptr()) }
    }

    pub fn get_nb_tire_data(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getNbTireData(self.as_ptr()) }
    }

    pub fn get_nb_susp_force_app_point_offset(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getNbSuspForceAppPointOffset(self.as_ptr()) }
    }

    pub fn get_nb_tire_force_app_point_offset(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getNbTireForceAppPointOffset(self.as_ptr()) }
    }

    pub fn get_nb_wheel_centre_offset(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getNbWheelCentreOffset(self.as_ptr()) }
    }

    pub fn get_nb_wheel_shape_mapping(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getNbWheelShapeMapping(self.as_ptr()) }
    }

    /*pub fn get_nb_scene_query_filter_data(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getNbSceneQueryFilterData(self.as_ptr()) }
    }*/

    pub fn get_min_long_slip_denominator(&self) -> f32 {
        unsafe { PxVehicleWheelsSimData_getMinLongSlipDenominator(self.as_ptr()) }
    }

    pub fn set_threshold_long_speed(&mut self, f: f32) {
        unsafe { PxVehicleWheelsSimData_setThresholdLongSpeed_mut(self.as_mut_ptr(), f) }
    }

    pub fn get_threshold_long_speed(&self) -> f32 {
        unsafe { PxVehicleWheelsSimData_getThresholdLongSpeed(self.as_ptr()) }
    }

    pub fn set_low_forward_speed_sub_step_count(&mut self, f: u32) {
        unsafe { PxVehicleWheelsSimData_setLowForwardSpeedSubStepCount_mut(self.as_mut_ptr(), f) }
    }

    pub fn get_low_forward_speed_sub_step_count(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getLowForwardSpeedSubStepCount(self.as_ptr()) }
    }

    pub fn set_high_forward_speed_sub_step_count(&mut self, f: u32) {
        unsafe { PxVehicleWheelsSimData_setHighForwardSpeedSubStepCount_mut(self.as_mut_ptr(), f) }
    }

    pub fn get_high_forward_speed_sub_step_count(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getHighForwardSpeedSubStepCount(self.as_ptr()) }
    }

    pub fn set_wheel_enabled_state(&mut self, wheel: u32, state: bool) {
        unsafe { PxVehicleWheelsSimData_setWheelEnabledState_mut(self.as_mut_ptr(), wheel, state) }
    }

    pub fn get_wheel_enabled_state(&self, wheel: u32) -> bool {
        unsafe { PxVehicleWheelsSimData_getWheelEnabledState(self.as_ptr(), wheel) }
    }

    pub fn get_nb_wheel_enabled_state(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getNbWheelEnabledState(self.as_ptr()) }
    }

    pub fn get_nb_anti_roll_bars_4(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getNbAntiRollBars4(self.as_ptr()) }
    }

    pub fn get_nb_anti_roll_bar_data(&self) -> u32 {
        unsafe { PxVehicleWheelsSimData_getNbAntiRollBarData(self.as_ptr()) }
    }

    pub fn set_anti_roll_bar_data(&mut self, id: u32, anti_roll: VehicleAntiRollBarData) {
        let anti_roll: PxVehicleAntiRollBarData = anti_roll.into();
        unsafe { PxVehicleWheelsSimData_setAntiRollBarData_mut(self.as_mut_ptr(), id, &anti_roll as *const _) }
    }
}

impl Drop for VehicleWheelsSimData {
    fn drop(&mut self) {
        unsafe { PxVehicleWheelsSimData_free_mut(self.as_mut_ptr()) }
    }
}
