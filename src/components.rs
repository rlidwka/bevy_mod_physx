use std::collections::HashMap;
use std::ptr::drop_in_place;

use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use physx::prelude::*;
use physx::traits::{Class, PxFlags};
use physx_sys::{
    PxShape_release_mut, PxPhysics_createShape_mut, PxFilterData, PxFilterData_new_2, PxMeshScale_new_3,
};

use physx::vehicles::{
    VehicleNoDrive, PxVehicleNoDrive, PxVehicleDriveTank, VehicleDriveTank,
    PxVehicleDriveSimData, PxVehicleDriveSimDataNW, PxVehicleDriveSimData4W,
    PxVehicleDrive4W, PxVehicleDriveNW, VehicleDrive4W, VehicleDriveNW, VehicleWheelsSimData
};

use crate::assets::GeometryInner;
use crate::bpx::{IntoPxVec3, IntoPxQuat};
use crate::prelude as bpx;
use crate::resources::SceneRwLock;
use super::{PxRigidStatic, PxRigidDynamic, PxShape};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RigidBody {
    Dynamic,
    Static,
}

#[derive(Component, Clone, Default)]
pub struct Shape {
    pub geometry: Handle<bpx::Geometry>,
    pub material: Handle<bpx::Material>,
    pub query_filter_data: FilterData,
    pub simulation_filter_data: FilterData,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct FilterData([ u32; 4 ]);

impl FilterData {
    pub fn new(word0: u32, word1: u32, word2: u32, word3: u32) -> Self {
        Self([ word0, word1, word2, word3 ])
    }
}

impl From<FilterData> for PxFilterData {
    fn from(value: FilterData) -> Self {
        let [ word0, word1, word2, word3 ] = value.0;
        unsafe { PxFilterData_new_2(word0, word1, word2, word3) }
    }
}

#[derive(Component)]
pub struct ShapeHandle(Option<Owner<PxShape>>);

impl ShapeHandle {
    pub fn new(px_shape: Owner<PxShape>) -> Self {
        Self(Some(px_shape))
    }

    pub fn create_shape(physics: &mut bpx::Physics, geometry: &mut bpx::Geometry, material: &mut bpx::Material, user_data: Entity) -> Self {
        let geometry_ptr = match geometry.obj {
            GeometryInner::Sphere(geom)  => { geom.as_ptr() },
            GeometryInner::Plane(geom)   => { geom.as_ptr() },
            GeometryInner::Capsule(geom) => { geom.as_ptr() },
            GeometryInner::Box(geom)     => { geom.as_ptr() },
            GeometryInner::ConvexMesh(ref mut geom) => {
                PxConvexMeshGeometry::new(
                    geom.mesh.lock().unwrap().as_mut(),
                    unsafe { &PxMeshScale_new_3(geom.scale.to_physx_sys().as_ptr(), geom.rotation.to_physx().as_ptr()) },
                    geom.flags,
                ).as_ptr()
            },
            GeometryInner::TriangleMesh(ref mut geom) => {
                PxTriangleMeshGeometry::new(
                    geom.mesh.lock().unwrap().as_mut(),
                    unsafe { &PxMeshScale_new_3(geom.scale.to_physx_sys().as_ptr(), geom.rotation.to_physx().as_ptr()) },
                    geom.flags,
                ).as_ptr()
            },
            GeometryInner::HeightField(ref mut geom) => {
                PxHeightFieldGeometry::new(
                    geom.hfield.lock().unwrap().as_mut(),
                    geom.flags,
                    geom.scale.y,
                    geom.scale.x,
                    geom.scale.z,
                ).as_ptr()
            },
        };

        //let shape = physics.create_shape(geometry, materials, is_exclusive, shape_flags, user_data)
        let shape : Owner<PxShape> = unsafe {
            physx::shape::Shape::from_raw(
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

impl Drop for ShapeHandle {
    fn drop(&mut self) {
        // TODO: remove this entire drop when this gets fixed:
        // https://github.com/EmbarkStudios/physx-rs/issues/180
        let mut shape = self.0.take().unwrap();
        unsafe {
            use physx::shape::Shape;
            drop_in_place(shape.get_user_data_mut());
            PxShape_release_mut(shape.as_mut_ptr());
        }
        std::mem::forget(shape);
    }
}

impl std::ops::Deref for ShapeHandle {
    type Target = PxShape;

    fn deref(&self) -> &Self::Target {
        // TODO: replace with Deref/DerefMut derive when this gets fixed:
        // https://github.com/EmbarkStudios/physx-rs/issues/180
        self.0.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for ShapeHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // TODO: replace with Deref/DerefMut derive when this gets fixed:
        // https://github.com/EmbarkStudios/physx-rs/issues/180
        self.0.as_mut().unwrap()
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct RigidDynamicHandle {
    #[deref]
    #[deref_mut]
    pub handle: SceneRwLock<Owner<PxRigidDynamic>>,
    // used for change detection
    pub cached_transform: GlobalTransform,
}

impl RigidDynamicHandle {
    pub fn new(px_rigid_dynamic: Owner<PxRigidDynamic>, transform: GlobalTransform) -> Self {
        Self { handle: SceneRwLock::new(px_rigid_dynamic), cached_transform: transform }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct RigidStaticHandle {
    #[deref]
    #[deref_mut]
    pub handle: SceneRwLock<Owner<PxRigidStatic>>,
    // used for change detection
    pub cached_transform: GlobalTransform,
}

impl RigidStaticHandle {
    pub fn new(px_rigid_static: Owner<PxRigidStatic>, transform: GlobalTransform) -> Self {
        Self { handle: SceneRwLock::new(px_rigid_static), cached_transform: transform }
    }
}

#[derive(Component, Debug, Default, PartialEq, Reflect, Clone, Copy)]
pub struct Velocity {
    pub linvel: Vec3,
    pub angvel: Vec3,
}

impl Velocity {
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

#[derive(Component)]
pub enum Vehicle {
    NoDrive {
        wheels: Vec<Entity>,
        wheels_sim_data: Owner<VehicleWheelsSimData>,
    },
    Drive4W {
        wheels: Vec<Entity>,
        wheels_sim_data: Owner<VehicleWheelsSimData>,
        drive_sim_data: Box<PxVehicleDriveSimData4W>,
    },
    DriveNW {
        wheels: Vec<Entity>,
        wheels_sim_data: Owner<VehicleWheelsSimData>,
        drive_sim_data: Box<PxVehicleDriveSimDataNW>,
    },
    DriveTank {
        wheels: Vec<Entity>,
        wheels_sim_data: Owner<VehicleWheelsSimData>,
        drive_sim_data: Box<PxVehicleDriveSimData>,
    },
}

#[derive(Component)]
pub enum VehicleHandle {
    NoDrive(SceneRwLock<Owner<PxVehicleNoDrive>>),
    Drive4W(SceneRwLock<Owner<PxVehicleDrive4W>>),
    DriveNW(SceneRwLock<Owner<PxVehicleDriveNW>>),
    DriveTank(SceneRwLock<Owner<PxVehicleDriveTank>>),
}

impl VehicleHandle {
    pub fn new(vehicle_desc: &mut Vehicle, physics: &mut bpx::Physics, actor: &mut PxRigidDynamic) -> Self {
        let (wheels, wheels_sim_data) = match vehicle_desc {
            Vehicle::NoDrive { wheels, wheels_sim_data } => (wheels, wheels_sim_data),
            Vehicle::Drive4W { wheels, wheels_sim_data, .. } => (wheels, wheels_sim_data),
            Vehicle::DriveNW { wheels, wheels_sim_data, .. } => (wheels, wheels_sim_data),
            Vehicle::DriveTank { wheels, wheels_sim_data, .. } => (wheels, wheels_sim_data),
        };

        let mut shape_mapping = HashMap::new();
        for (idx, shape) in actor.get_shapes().into_iter().enumerate() {
            use physx::shape::Shape;
            shape_mapping.insert(*shape.get_user_data(), idx as i32);
        }

        for (wheel_id, entity) in wheels.iter().enumerate() {
            wheels_sim_data.set_wheel_shape_mapping(wheel_id as u32, *shape_mapping.get(entity).unwrap());
        }

        match vehicle_desc {
            Vehicle::NoDrive { wheels: _, wheels_sim_data } => {
                Self::NoDrive(
                    SceneRwLock::new(VehicleNoDrive::new(physics.physics_mut(), actor, wheels_sim_data).unwrap())
                )
            }
            Vehicle::Drive4W { wheels, wheels_sim_data, drive_sim_data } => {
                Self::Drive4W(
                    SceneRwLock::new(VehicleDrive4W::new(physics.physics_mut(), actor, wheels_sim_data, drive_sim_data.as_ref(), wheels.len() as u32 - 4).unwrap())
                )
            }
            Vehicle::DriveNW { wheels, wheels_sim_data, drive_sim_data } => {
                Self::DriveNW(
                    SceneRwLock::new(VehicleDriveNW::new(physics.physics_mut(), actor, wheels_sim_data, drive_sim_data.as_ref(), wheels.len() as u32).unwrap())
                )
            }
            Vehicle::DriveTank { wheels, wheels_sim_data, drive_sim_data } => {
                Self::DriveTank(
                    SceneRwLock::new(VehicleDriveTank::new(physics.physics_mut(), actor, wheels_sim_data, drive_sim_data.as_ref(), wheels.len() as u32).unwrap())
                )
            }
        }
    }
}

#[derive(Component, Debug, Clone)]
pub enum MassProperties {
    Density {
        density: f32,
        center: Vec3,
    },
    Mass {
        mass: f32,
        center: Vec3,
    },
}

impl MassProperties {
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
