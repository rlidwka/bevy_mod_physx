use std::ptr::drop_in_place;
use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use physx::prelude::*;
use physx::traits::{Class, PxFlags};
use physx_sys::{
    PxShape_release_mut, PxPhysics_createShape_mut, PxConvexMeshGeometryFlag, PxConvexMeshGeometryFlags,
    PxMeshGeometryFlags, PxMeshGeometryFlag, PxMeshScale_new, PxVehicleWheelData_new, PxVehicleWheelData,
    PxVehicleTireData, PxVehicleTireData_new, PxVehicleSuspensionData_new, PxVehicleSuspensionData,
    PxVehicleWheels,
};
use super::{PxRigidStatic, PxRigidDynamic, PxShape};
use super::assets::{BPxGeometry, BPxMaterial};
use super::resources::BPxPhysics;

#[derive(Component, Clone)]
pub enum BPxActor {
    Dynamic,
    Static,
}

#[derive(Component, Clone)]
pub struct BPxShape {
    pub geometry: Handle<BPxGeometry>,
    pub material: Handle<BPxMaterial>,
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
pub struct BPxVehicle;

#[derive(Debug, Component, Clone)]
pub struct BPxVehicleWheel {
    pub wheel_data: BPxVehicleWheelData,
    pub tire_data: BPxVehicleTireData,
    pub suspension_data: BPxVehicleSuspensionData,
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
        susp_data.mCamberAtMaxCompression = self.max_compression;
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
