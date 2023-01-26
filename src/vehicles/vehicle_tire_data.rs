use physx_sys::{
    PxVehicleTireData,
    PxVehicleTireData_new,
    //PxVehicleTireData_getRecipLongitudinalStiffnessPerUnitGravity,
    //PxVehicleTireData_getFrictionVsSlipGraphRecipx1Minusx0,
    //PxVehicleTireData_getFrictionVsSlipGraphRecipx2Minusx1,
};

#[derive(Debug, Clone)]
pub struct VehicleTireData {
    /// Tire lateral stiffness is a graph of tire load that has linear behavior near zero load and flattens at large loads. mLatStiffX describes the minimum normalized load (load/restLoad) that gives a flat lateral stiffness response to load.
    pub lat_stiff_x: f32,
    /// Tire lateral stiffness is a graph of tire load that has linear behavior near zero load and flattens at large loads. mLatStiffY describes the maximum possible value of lateralStiffness/restLoad that occurs when (load/restLoad)>= mLatStiffX.
    pub lat_stiff_y: f32,
    /// Tire Longitudinal stiffness per unit gravitational acceleration.
    pub longitudinal_stiffness_per_unit_gravity: f32,
    /// Tire camber stiffness per unity gravitational acceleration.
    pub camber_stiffness_per_unit_gravity: f32,
    /// Graph of friction vs longitudinal slip with 3 points.
    pub friction_vs_slip_graph: [[f32; 2]; 3],
    /// Tire type denoting slicks, wets, snow, winter, summer, all-terrain, mud etc.
    pub tire_type: u32,
}

impl From<PxVehicleTireData> for VehicleTireData {
    fn from(value: PxVehicleTireData) -> Self {
        Self {
            lat_stiff_x: value.mLatStiffX,
            lat_stiff_y: value.mLatStiffY,
            longitudinal_stiffness_per_unit_gravity: value.mLongitudinalStiffnessPerUnitGravity,
            camber_stiffness_per_unit_gravity: value.mCamberStiffnessPerUnitGravity,
            friction_vs_slip_graph: value.mFrictionVsSlipGraph,
            tire_type: value.mType,
        }
    }
}

impl From<VehicleTireData> for PxVehicleTireData {
    fn from(value: VehicleTireData) -> Self {
        let mut result = unsafe { PxVehicleTireData_new() };
        result.mLatStiffX = value.lat_stiff_x;
        result.mLatStiffY = value.lat_stiff_y;
        result.mLongitudinalStiffnessPerUnitGravity = value.longitudinal_stiffness_per_unit_gravity;
        result.mCamberStiffnessPerUnitGravity = value.camber_stiffness_per_unit_gravity;
        result.mFrictionVsSlipGraph = value.friction_vs_slip_graph;
        result.mType = value.tire_type;
        result
    }
}

impl Default for VehicleTireData {
    fn default() -> Self {
        unsafe { PxVehicleTireData_new() }.into()
    }
}
