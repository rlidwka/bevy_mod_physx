use std::ptr::drop_in_place;
use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use physx::prelude::*;
use physx::traits::{Class, PxFlags};
use physx_sys::{
    PxShape_release_mut, PxPhysics_createShape_mut, PxConvexMeshGeometryFlag, PxConvexMeshGeometryFlags,
    PxMeshGeometryFlags, PxMeshGeometryFlag, PxMeshScale_new, PxVehicleWheelData_new, PxVehicleWheelData,
    PxVehicleTireData, PxVehicleTireData_new, PxVehicleSuspensionData_new, PxVehicleSuspensionData,
    PxVehicleWheels, PxFilterData, PxFilterData_new_2, PxVehicleEngineData_new, PxVehicleEngineData, PxVehicleGearsData_new, PxVehicleGearsData, PxVehicleClutchData_new, PxVehicleClutchData, PxVehicleAutoBoxData, PxVehicleAutoBoxData_new, PxVehicleDifferential4WData_new, PxVehicleAckermannGeometryData, PxVehicleAckermannGeometryData_new, PxVehicleDifferential4WData, PxVehicleDriveSimData_setEngineData_mut, PxVehicleDriveSimData_setGearsData_mut, PxVehicleDriveSimData_setClutchData_mut, PxVehicleDriveSimData_setAutoBoxData_mut, PxVehicleDriveSimData_new, PxVehicleDriveSimData, PxVehicleDriveSimDataNW_new, PxVehicleDriveSimDataNW, PxVehicleDriveSimData4W, PxVehicleDriveSimData4W_new, PxVehicleDriveSimData4W_setAckermannGeometryData_mut, PxVehicleDriveSimData4W_setDiffData_mut,
};
use super::{PxRigidStatic, PxRigidDynamic, PxShape};
use super::assets::{BPxGeometry, BPxMaterial};
use super::resources::BPxPhysics;

#[derive(Component, Clone)]
pub enum BPxActor {
    Dynamic,
    Static,
}

#[derive(Component, Clone, Default)]
pub struct BPxShape {
    pub geometry: Handle<BPxGeometry>,
    pub material: Handle<BPxMaterial>,
    pub query_filter_data: BPxFilterData,
    pub simulation_filter_data: BPxFilterData,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct BPxFilterData([ u32; 4 ]);

impl BPxFilterData {
    pub fn new(word0: u32, word1: u32, word2: u32, word3: u32) -> Self {
        Self([ word0, word1, word2, word3 ])
    }
}

impl From<BPxFilterData> for PxFilterData {
    fn from(value: BPxFilterData) -> Self {
        let [ word0, word1, word2, word3 ] = value.0;
        unsafe { PxFilterData_new_2(word0, word1, word2, word3) }
    }
}

#[derive(Component)]
pub struct BPxShapeHandle(Option<Owner<PxShape>>);

impl BPxShapeHandle {
    pub fn new(px_shape: Owner<PxShape>) -> Self {
        Self(Some(px_shape))
    }

    pub fn create_shape(physics: &mut BPxPhysics, geometry: &mut BPxGeometry, material: &mut BPxMaterial, user_data: Entity) -> Self {
        let geometry_ptr = match geometry {
            BPxGeometry::Sphere(geom)  => { geom.as_ptr() },
            BPxGeometry::Plane(geom)   => { geom.as_ptr() },
            BPxGeometry::Capsule(geom) => { geom.as_ptr() },
            BPxGeometry::Box(geom)     => { geom.as_ptr() },
            BPxGeometry::ConvexMesh(mesh) => {
                PxConvexMeshGeometry::new(
                    mesh.as_mut(),
                    unsafe { &PxMeshScale_new() },
                    PxConvexMeshGeometryFlags { mBits: PxConvexMeshGeometryFlag::eTIGHT_BOUNDS as u8 }
                ).as_ptr()
            },
            BPxGeometry::TriangleMesh(mesh) => {
                PxTriangleMeshGeometry::new(
                    mesh.as_mut(),
                    unsafe { &PxMeshScale_new() },
                    PxMeshGeometryFlags { mBits: PxMeshGeometryFlag::eDOUBLE_SIDED as u8 }
                ).as_ptr()
            },
        };

        //let shape = physics.create_shape(geometry, materials, is_exclusive, shape_flags, user_data)
        let shape : Owner<PxShape> = unsafe {
            Shape::from_raw(
                PxPhysics_createShape_mut(
                    physics.physics_mut().as_mut_ptr(),
                    geometry_ptr,
                    material.as_ptr(),
                    true,
                    (ShapeFlag::SceneQueryShape | ShapeFlag::SimulationShape | ShapeFlag::Visualization).into_px(),
                ),
                user_data
            ).unwrap()
        };

        Self::new(shape)
    }
}

impl Drop for BPxShapeHandle {
    fn drop(&mut self) {
        // TODO: remove this entire drop when this gets fixed:
        // https://github.com/EmbarkStudios/physx-rs/issues/180
        let mut shape = self.0.take().unwrap();
        unsafe {
            drop_in_place(shape.get_user_data_mut());
            PxShape_release_mut(shape.as_mut_ptr());
        }
        std::mem::forget(shape);
    }
}

impl Deref for BPxShapeHandle {
    type Target = PxShape;

    fn deref(&self) -> &Self::Target {
        // TODO: replace with Deref/DerefMut derive when this gets fixed:
        // https://github.com/EmbarkStudios/physx-rs/issues/180
        self.0.as_ref().unwrap()
    }
}

impl DerefMut for BPxShapeHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // TODO: replace with Deref/DerefMut derive when this gets fixed:
        // https://github.com/EmbarkStudios/physx-rs/issues/180
        self.0.as_mut().unwrap()
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct BPxRigidDynamicHandle(Owner<PxRigidDynamic>);

impl BPxRigidDynamicHandle {
    pub fn new(px_rigid_dynamic: Owner<PxRigidDynamic>) -> Self {
        Self(px_rigid_dynamic)
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct BPxRigidStaticHandle(Owner<PxRigidStatic>);

impl BPxRigidStaticHandle {
    pub fn new(px_rigid_static: Owner<PxRigidStatic>) -> Self {
        Self(px_rigid_static)
    }
}

#[derive(Component)]
pub struct BPxVehicleHandle {
    // it should be Owner<PxVehicleXXXX> when physx implements vehicles,
    // for now we drop pointer manually, similar to what Owner does
    pub ptr: *mut PxVehicleWheels,
    pub wheels: usize,
}

// need some advice on soundness of implementing Send for *mut X
unsafe impl Send for BPxVehicleHandle {}
unsafe impl Sync for BPxVehicleHandle {}

impl Drop for BPxVehicleHandle {
    fn drop(&mut self) {
        unsafe {
            drop_in_place(self.ptr);
        }
    }
}

#[derive(Component, Debug, Default, PartialEq, Reflect, Clone, Copy)]
pub struct BPxVelocity {
    pub linvel: Vec3,
    pub angvel: Vec3,
}

impl BPxVelocity {
    pub fn new(linvel: Vec3, angvel: Vec3) -> Self {
        Self { linvel, angvel }
    }

    pub fn zero() -> Self {
        Self { ..default() }
    }

    pub fn linear(linvel: Vec3) -> Self {
        Self { linvel, ..default() }
    }

    pub fn angular(angvel: Vec3) -> Self {
        Self { angvel, ..default() }
    }
}

#[derive(Component, Clone)]
pub struct BPxVehicleNoDrive {
    wheels: Vec<Entity>,
    controls: Vec<BPxVehicleNoDriveWheelControl>,
}

impl BPxVehicleNoDrive {
    pub fn new(wheels: &[Entity]) -> Self {
        Self {
            wheels: wheels.to_vec(),
            controls: wheels.iter().map(|_| default()).collect::<Vec<_>>(),
        }
    }

    pub fn get_wheels(&self) -> &[Entity] {
        &self.wheels
    }

    pub fn set_drive_torque(&mut self, wheel_id: usize, drive_torque: f32) {
        self.controls[wheel_id].drive_torque = drive_torque;
    }

    pub fn set_brake_torque(&mut self, wheel_id: usize, brake_torque: f32) {
        self.controls[wheel_id].brake_torque = brake_torque;
    }

    pub fn set_steer_angle(&mut self, wheel_id: usize, steer_angle: f32) {
        self.controls[wheel_id].steer_angle = steer_angle;
    }

    pub fn get_drive_torque(&self, wheel_id: usize) -> f32 {
        self.controls[wheel_id].drive_torque
    }

    pub fn get_brake_torque(&self, wheel_id: usize) -> f32 {
        self.controls[wheel_id].brake_torque
    }

    pub fn get_steer_angle(&self, wheel_id: usize) -> f32 {
        self.controls[wheel_id].steer_angle
    }
}

#[derive(Clone, Default)]
struct BPxVehicleNoDriveWheelControl {
    drive_torque: f32,
    brake_torque: f32,
    steer_angle: f32,
}

#[derive(Component, Clone)]
pub struct BPxVehicle4W {
    wheels: Vec<Entity>,
    drive: BPxVehicleDriveSimData4W,
}

impl BPxVehicle4W {
    pub fn new(wheels: &[Entity], drive: BPxVehicleDriveSimData4W) -> Self {
        Self {
            wheels: wheels.to_vec(),
            drive
        }
    }

    pub fn get_wheels(&self) -> &[Entity] {
        &self.wheels
    }

    pub fn get_drive(&self) -> &BPxVehicleDriveSimData4W {
        &self.drive
    }
}

#[derive(Component, Clone)]
pub struct BPxVehicleNW {
    wheels: Vec<Entity>,
    drive: BPxVehicleDriveSimDataNW,
}

impl BPxVehicleNW {
    pub fn new(wheels: &[Entity], drive: BPxVehicleDriveSimDataNW) -> Self {
        Self {
            wheels: wheels.to_vec(),
            drive,
        }
    }

    pub fn get_wheels(&self) -> &[Entity] {
        &self.wheels
    }

    pub fn get_drive(&self) -> &BPxVehicleDriveSimDataNW {
        &self.drive
    }
}

#[derive(Component, Clone)]
pub struct BPxVehicleTank {
    wheels: Vec<Entity>,
    drive: BPxVehicleDriveSimData,
}

impl BPxVehicleTank {
    pub fn new(wheels: &[Entity], drive: BPxVehicleDriveSimData) -> Self {
        Self {
            wheels: wheels.to_vec(),
            drive,
        }
    }

    pub fn get_wheels(&self) -> &[Entity] {
        &self.wheels
    }

    pub fn get_drive(&self) -> &BPxVehicleDriveSimData {
        &self.drive
    }
}

#[derive(Debug, Component, Clone)]
pub struct BPxVehicleWheel {
    pub wheel_data: BPxVehicleWheelData,
    pub tire_data: BPxVehicleTireData,
    pub suspension_data: BPxVehicleSuspensionData,
    pub anti_roll_bar: BPxVehicleAntiRollBarData,
    pub susp_travel_direction: Vec3,
    pub susp_force_app_point_offset: Vec3,
    pub tire_force_app_point_offset: Vec3,
}

impl Default for BPxVehicleWheel {
    fn default() -> Self {
        Self {
            wheel_data: default(),
            tire_data: default(),
            suspension_data: default(),
            anti_roll_bar: default(),
            susp_travel_direction: Vec3::new(0., -1., 0.),
            susp_force_app_point_offset: Vec3::ZERO,
            tire_force_app_point_offset: Vec3::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BPxVehicleWheelData {
    pub radius: f32,
    pub width: f32,
    pub mass: f32,
    pub moi: f32,
    pub damping_rate: f32,
    pub max_brake_torque: f32,
    pub max_hand_brake_torque: f32,
    pub max_steer: f32,
    pub toe_angle: f32,
}

impl BPxVehicleWheelData {
    pub fn to_physx(&self) -> PxVehicleWheelData {
        let mut wheel_data = unsafe { PxVehicleWheelData_new() };
        wheel_data.mRadius = self.radius;
        wheel_data.mWidth = self.width;
        wheel_data.mMass = self.mass;
        wheel_data.mMOI = self.moi;
        wheel_data.mDampingRate = self.damping_rate;
        wheel_data.mMaxBrakeTorque = self.max_brake_torque;
        wheel_data.mMaxHandBrakeTorque = self.max_hand_brake_torque;
        wheel_data.mMaxSteer = self.max_steer;
        wheel_data.mToeAngle = self.toe_angle;
        wheel_data
    }

    pub fn from_physx(wheel_data: PxVehicleWheelData) -> Self {
        Self {
            radius: wheel_data.mRadius,
            width: wheel_data.mWidth,
            mass: wheel_data.mMass,
            moi: wheel_data.mMOI,
            damping_rate: wheel_data.mDampingRate,
            max_brake_torque: wheel_data.mMaxBrakeTorque,
            max_hand_brake_torque: wheel_data.mMaxBrakeTorque,
            max_steer: wheel_data.mMaxSteer,
            toe_angle: wheel_data.mToeAngle,
        }
    }
}

impl Default for BPxVehicleWheelData {
    fn default() -> Self {
        Self::from_physx(unsafe { PxVehicleWheelData_new() })
    }
}

#[derive(Debug, Clone)]
pub struct BPxVehicleTireData {
    pub lat_stiff_x: f32,
    pub lat_stiff_y: f32,
    pub longitudinal_stiffness_per_unit_gravity: f32,
    pub camber_stiffness_per_unit_gravity: f32,
    pub friction_vs_slip_graph: [[f32; 2]; 3],
    pub tire_type: u32,
}

impl BPxVehicleTireData {
    pub fn to_physx(&self) -> PxVehicleTireData {
        let mut tire_data = unsafe { PxVehicleTireData_new() };
        tire_data.mLatStiffX = self.lat_stiff_x;
        tire_data.mLatStiffY = self.lat_stiff_y;
        tire_data.mLongitudinalStiffnessPerUnitGravity = self.longitudinal_stiffness_per_unit_gravity;
        tire_data.mCamberStiffnessPerUnitGravity = self.camber_stiffness_per_unit_gravity;
        tire_data.mFrictionVsSlipGraph = self.friction_vs_slip_graph;
        tire_data.mType = self.tire_type;
        tire_data
    }

    pub fn from_physx(tire_data: PxVehicleTireData) -> Self {
        Self {
            lat_stiff_x: tire_data.mLatStiffX,
            lat_stiff_y: tire_data.mLatStiffY,
            longitudinal_stiffness_per_unit_gravity: tire_data.mLongitudinalStiffnessPerUnitGravity,
            camber_stiffness_per_unit_gravity: tire_data.mCamberStiffnessPerUnitGravity,
            friction_vs_slip_graph: tire_data.mFrictionVsSlipGraph,
            tire_type: tire_data.mType,
        }
    }
}

impl Default for BPxVehicleTireData {
    fn default() -> Self {
        Self::from_physx(unsafe { PxVehicleTireData_new() })
    }
}

#[derive(Debug, Clone)]
pub struct BPxVehicleSuspensionData {
    pub spring_strength: f32,
    pub spring_damper_rate: f32,
    pub max_compression: f32,
    pub max_droop: f32,
    // this will be automatically calculated
    //pub sprung_mass: f32,
    pub camber_at_rest: f32,
    pub camber_at_max_compression: f32,
    pub camber_at_max_droop: f32,
}

impl BPxVehicleSuspensionData {
    pub fn to_physx(&self) -> PxVehicleSuspensionData {
        let mut susp_data = unsafe { PxVehicleSuspensionData_new() };
        susp_data.mSpringStrength = self.spring_strength;
        susp_data.mSpringDamperRate = self.spring_damper_rate;
        susp_data.mMaxCompression = self.max_compression;
        susp_data.mMaxDroop = self.max_droop;
        //susp_data.mSprungMass = self.sprung_mass;
        susp_data.mCamberAtRest = self.camber_at_rest;
        susp_data.mCamberAtMaxCompression = self.camber_at_max_compression;
        susp_data.mCamberAtMaxDroop = self.camber_at_max_droop;
        susp_data
    }

    pub fn from_physx(susp_data: PxVehicleSuspensionData) -> Self {
        Self {
            spring_strength: susp_data.mSpringStrength,
            spring_damper_rate: susp_data.mSpringDamperRate,
            max_compression: susp_data.mMaxCompression,
            max_droop: susp_data.mMaxDroop,
            //sprung_mass: susp_data.mSprungMass,
            camber_at_rest: susp_data.mCamberAtRest,
            camber_at_max_compression: susp_data.mCamberAtMaxCompression,
            camber_at_max_droop: susp_data.mCamberAtMaxDroop,
        }
    }
}

impl Default for BPxVehicleSuspensionData {
    fn default() -> Self {
        Self::from_physx(unsafe { PxVehicleSuspensionData_new() })
    }
}

#[derive(Debug, Clone, Default)]
pub struct BPxVehicleAntiRollBarData {
    pub with_wheel_id: u32,
    pub stiffness: f32,
}

#[derive(Component, Debug, Clone)]
pub enum BPxMassProperties {
    Density {
        density: f32,
        center: Vec3,
    },
    Mass {
        mass: f32,
        center: Vec3,
    },
}

impl BPxMassProperties {
    pub fn density(density: f32) -> Self {
        Self::Density { density, center: Vec3::ZERO }
    }

    pub fn mass(mass: f32) -> Self {
        Self::Mass { mass, center: Vec3::ZERO }
    }

    pub fn density_with_center(density: f32, center: Vec3) -> Self {
        Self::Density { density, center }
    }

    pub fn mass_with_center(mass: f32, center: Vec3) -> Self {
        Self::Mass { mass, center }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BPxVehicleDriveSimData {
    pub engine: BPxVehicleEngineData,
    pub gears: BPxVehicleGearsData,
    pub clutch: BPxVehicleClutchData,
    pub autobox: BPxVehicleAutoBoxData,
}

impl BPxVehicleDriveSimData {
    pub fn to_physx(&self) -> PxVehicleDriveSimData {
        unsafe {
            let mut drive_data = PxVehicleDriveSimData_new();
            PxVehicleDriveSimData_setEngineData_mut(&mut drive_data as *mut _, &self.engine.to_physx() as *const _);
            PxVehicleDriveSimData_setGearsData_mut(&mut drive_data as *mut _, &self.gears.to_physx() as *const _);
            PxVehicleDriveSimData_setClutchData_mut(&mut drive_data as *mut _, &self.clutch.to_physx() as *const _);
            PxVehicleDriveSimData_setAutoBoxData_mut(&mut drive_data as *mut _, &self.autobox.to_physx() as *const _);
            drive_data
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BPxVehicleDriveSimData4W {
    pub engine: BPxVehicleEngineData,
    pub gears: BPxVehicleGearsData,
    pub clutch: BPxVehicleClutchData,
    pub autobox: BPxVehicleAutoBoxData,
    pub diff: BPxVehicleDifferential4WData,
    pub ackermann_geometry: BPxVehicleAckermannGeometryData,
}

impl BPxVehicleDriveSimData4W {
    pub fn to_physx(&self) -> PxVehicleDriveSimData4W {
        unsafe {
            let mut drive_data = PxVehicleDriveSimData4W_new();
            PxVehicleDriveSimData_setEngineData_mut(&mut drive_data as *mut PxVehicleDriveSimData4W as *mut _, &self.engine.to_physx() as *const _);
            PxVehicleDriveSimData_setGearsData_mut(&mut drive_data as *mut PxVehicleDriveSimData4W as *mut _, &self.gears.to_physx() as *const _);
            PxVehicleDriveSimData_setClutchData_mut(&mut drive_data as *mut PxVehicleDriveSimData4W as *mut _, &self.clutch.to_physx() as *const _);
            PxVehicleDriveSimData_setAutoBoxData_mut(&mut drive_data as *mut PxVehicleDriveSimData4W as *mut _, &self.autobox.to_physx() as *const _);
            PxVehicleDriveSimData4W_setDiffData_mut(&mut drive_data as *mut _, &self.diff.to_physx() as *const _);
            PxVehicleDriveSimData4W_setAckermannGeometryData_mut(&mut drive_data as *mut _, &self.ackermann_geometry.to_physx() as *const _);
            drive_data
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BPxVehicleDriveSimDataNW {
    pub engine: BPxVehicleEngineData,
    pub gears: BPxVehicleGearsData,
    pub clutch: BPxVehicleClutchData,
    pub autobox: BPxVehicleAutoBoxData,
    //pub diff: BPxVehicleDifferentialNWData,
}

impl BPxVehicleDriveSimDataNW {
    pub fn to_physx(&self) -> PxVehicleDriveSimDataNW {
        unsafe {
            let mut drive_data = PxVehicleDriveSimDataNW_new();
            PxVehicleDriveSimData_setEngineData_mut(&mut drive_data as *mut PxVehicleDriveSimDataNW as *mut _, &self.engine.to_physx() as *const _);
            PxVehicleDriveSimData_setGearsData_mut(&mut drive_data as *mut PxVehicleDriveSimDataNW as *mut _, &self.gears.to_physx() as *const _);
            PxVehicleDriveSimData_setClutchData_mut(&mut drive_data as *mut PxVehicleDriveSimDataNW as *mut _, &self.clutch.to_physx() as *const _);
            PxVehicleDriveSimData_setAutoBoxData_mut(&mut drive_data as *mut PxVehicleDriveSimDataNW as *mut _, &self.autobox.to_physx() as *const _);
            drive_data
        }
    }
}

#[derive(Debug, Clone)]
pub struct BPxVehicleEngineData {
    pub moi: f32,
    pub peak_torque: f32,
    pub max_omega: f32,
    pub damping_rate_full_throttle: f32,
    pub damping_rate_zero_throttle_clutch_engaged: f32,
    pub damping_rate_zero_throttle_clutch_disengaged: f32,
}

impl BPxVehicleEngineData {
    pub fn to_physx(&self) -> PxVehicleEngineData {
        let mut engine_data = unsafe { PxVehicleEngineData_new() };
        engine_data.mMOI = self.moi;
        engine_data.mPeakTorque = self.peak_torque;
        engine_data.mMaxOmega = self.max_omega;
        engine_data.mDampingRateFullThrottle = self.damping_rate_full_throttle;
        engine_data.mDampingRateZeroThrottleClutchEngaged = self.damping_rate_zero_throttle_clutch_engaged;
        engine_data.mDampingRateZeroThrottleClutchDisengaged = self.damping_rate_zero_throttle_clutch_disengaged;
        engine_data
    }

    pub fn from_physx(engine_data: PxVehicleEngineData) -> Self {
        Self {
            moi: engine_data.mMOI,
            peak_torque: engine_data.mPeakTorque,
            max_omega: engine_data.mMaxOmega,
            damping_rate_full_throttle: engine_data.mDampingRateFullThrottle,
            damping_rate_zero_throttle_clutch_engaged: engine_data.mDampingRateZeroThrottleClutchEngaged,
            damping_rate_zero_throttle_clutch_disengaged: engine_data.mDampingRateZeroThrottleClutchDisengaged,
        }
    }
}

impl Default for BPxVehicleEngineData {
    fn default() -> Self {
        Self::from_physx(unsafe { PxVehicleEngineData_new() })
    }
}

#[derive(Debug, Clone)]
pub struct BPxVehicleGearsData {
    pub ratios: [f32; 32],
    pub final_ratio: f32,
    pub nb_ratios: u32,
    pub switch_time: f32,
}

impl BPxVehicleGearsData {
    pub fn to_physx(&self) -> PxVehicleGearsData {
        let mut gears_data = unsafe { PxVehicleGearsData_new() };
        gears_data.mRatios = self.ratios;
        gears_data.mFinalRatio = self.final_ratio;
        gears_data.mNbRatios = self.nb_ratios;
        gears_data.mSwitchTime = self.switch_time;
        gears_data
    }

    pub fn from_physx(gears_data: PxVehicleGearsData) -> Self {
        Self {
            ratios: gears_data.mRatios,
            final_ratio: gears_data.mFinalRatio,
            nb_ratios: gears_data.mNbRatios,
            switch_time: gears_data.mSwitchTime,
        }
    }
}

impl Default for BPxVehicleGearsData {
    fn default() -> Self {
        Self::from_physx(unsafe { PxVehicleGearsData_new() })
    }
}

#[derive(Debug, Clone)]
pub struct BPxVehicleClutchData {
    pub strength: f32,
    pub accuracy_mode: u32,
    pub estimate_iterations: u32,
}

impl BPxVehicleClutchData {
    pub fn to_physx(&self) -> PxVehicleClutchData {
        let mut clutch_data = unsafe { PxVehicleClutchData_new() };
        clutch_data.mStrength = self.strength;
        clutch_data.mAccuracyMode = self.accuracy_mode;
        clutch_data.mEstimateIterations = self.estimate_iterations;
        clutch_data
    }

    pub fn from_physx(clutch_data: PxVehicleClutchData) -> Self {
        Self {
            strength: clutch_data.mStrength,
            accuracy_mode: clutch_data.mAccuracyMode,
            estimate_iterations: clutch_data.mEstimateIterations,
        }
    }
}

impl Default for BPxVehicleClutchData {
    fn default() -> Self {
        Self::from_physx(unsafe { PxVehicleClutchData_new() })
    }
}

#[derive(Debug, Clone)]
pub struct BPxVehicleAutoBoxData {
    pub up_ratios: [f32; 32],
    pub down_ratios: [f32; 32],
}

impl BPxVehicleAutoBoxData {
    pub fn to_physx(&self) -> PxVehicleAutoBoxData {
        let mut autobox_data = unsafe { PxVehicleAutoBoxData_new() };
        autobox_data.mUpRatios = self.up_ratios;
        autobox_data.mDownRatios = self.down_ratios;
        autobox_data
    }

    pub fn from_physx(autobox_data: PxVehicleAutoBoxData) -> Self {
        Self {
            up_ratios: autobox_data.mUpRatios,
            down_ratios: autobox_data.mDownRatios,
        }
    }
}

impl Default for BPxVehicleAutoBoxData {
    fn default() -> Self {
        Self::from_physx(unsafe { PxVehicleAutoBoxData_new() })
    }
}

#[derive(Debug, Clone)]
pub struct BPxVehicleDifferential4WData {
    pub front_rear_split: f32,
    pub front_left_right_split: f32,
    pub rear_left_right_split: f32,
    pub centre_bias: f32,
    pub front_bias: f32,
    pub rear_bias: f32,
    pub diff_type: u32,
}

impl BPxVehicleDifferential4WData {
    pub fn to_physx(&self) -> PxVehicleDifferential4WData {
        let mut diff_data = unsafe { PxVehicleDifferential4WData_new() };
        diff_data.mFrontRearSplit = self.front_rear_split;
        diff_data.mFrontLeftRightSplit = self.front_left_right_split;
        diff_data.mRearLeftRightSplit = self.rear_left_right_split;
        diff_data.mCentreBias = self.centre_bias;
        diff_data.mFrontBias = self.front_bias;
        diff_data.mRearBias = self.rear_bias;
        diff_data.mType = self.diff_type;
        diff_data
    }

    pub fn from_physx(diff_data: PxVehicleDifferential4WData) -> Self {
        Self {
            front_rear_split: diff_data.mFrontRearSplit,
            front_left_right_split: diff_data.mFrontLeftRightSplit,
            rear_left_right_split: diff_data.mRearLeftRightSplit,
            centre_bias: diff_data.mCentreBias,
            front_bias: diff_data.mFrontBias,
            rear_bias: diff_data.mRearBias,
            diff_type: diff_data.mType,
        }
    }
}

impl Default for BPxVehicleDifferential4WData {
    fn default() -> Self {
        Self::from_physx(unsafe { PxVehicleDifferential4WData_new() })
    }
}

#[derive(Debug, Clone)]
pub struct BPxVehicleAckermannGeometryData {
    pub accuracy: f32,
    pub front_width: f32,
    pub rear_width: f32,
    pub axle_separation: f32,
}

impl BPxVehicleAckermannGeometryData {
    pub fn to_physx(&self) -> PxVehicleAckermannGeometryData {
        let mut ageom_data = unsafe { PxVehicleAckermannGeometryData_new() };
        ageom_data.mAccuracy = self.accuracy;
        ageom_data.mFrontWidth = self.front_width;
        ageom_data.mRearWidth = self.rear_width;
        ageom_data.mAxleSeparation = self.axle_separation;
        ageom_data
    }

    pub fn from_physx(ageom_data: PxVehicleAckermannGeometryData) -> Self {
        Self {
            accuracy: ageom_data.mAccuracy,
            front_width: ageom_data.mFrontWidth,
            rear_width: ageom_data.mRearWidth,
            axle_separation: ageom_data.mAxleSeparation,
        }
    }
}

impl Default for BPxVehicleAckermannGeometryData {
    fn default() -> Self {
        Self::from_physx(unsafe { PxVehicleAckermannGeometryData_new() })
    }
}
