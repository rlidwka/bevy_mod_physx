use std::collections::HashMap;
use std::ptr::drop_in_place;
use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use physx::prelude::*;
use physx::traits::{Class, PxFlags};
use physx_sys::{
    PxShape_release_mut, PxPhysics_createShape_mut, PxConvexMeshGeometryFlag, PxConvexMeshGeometryFlags,
    PxMeshGeometryFlags, PxMeshGeometryFlag, PxMeshScale_new, PxFilterData, PxFilterData_new_2,
};

use crate::vehicles::{VehicleNoDrive, PxVehicleNoDrive, PxVehicleDriveTank, VehicleDriveTank, PxVehicleDriveSimData, VehicleWheels, VehicleDriveSimData4W, VehicleDriveSimDataNW, VehicleDriveSimData, PxVehicleDriveSimDataNW, PxVehicleDriveSimData4W, PxVehicleDrive4W, PxVehicleDriveNW, VehicleDrive4W, VehicleDriveNW};

use super::{PxRigidStatic, PxRigidDynamic, PxShape};
use super::assets::{BPxGeometry, BPxMaterial};
use super::resources::BPxPhysics;
use super::vehicles::VehicleWheelsSimData;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Component)]
pub enum BPxVehicle {
    NoDrive {
        wheels: Vec<Entity>,
        wheels_sim_data: crate::vehicles::Owner<VehicleWheelsSimData>,
    },
    Drive4W {
        wheels: Vec<Entity>,
        wheels_sim_data: crate::vehicles::Owner<VehicleWheelsSimData>,
        drive_sim_data: Box<PxVehicleDriveSimData4W>,
    },
    DriveNW {
        wheels: Vec<Entity>,
        wheels_sim_data: crate::vehicles::Owner<VehicleWheelsSimData>,
        drive_sim_data: Box<PxVehicleDriveSimDataNW>,
    },
    DriveTank {
        wheels: Vec<Entity>,
        wheels_sim_data: crate::vehicles::Owner<VehicleWheelsSimData>,
        drive_sim_data: Box<PxVehicleDriveSimData>,
    },
}

#[derive(Component)]
pub enum BPxVehicleHandle {
    NoDrive(crate::vehicles::Owner<PxVehicleNoDrive>),
    Drive4W(crate::vehicles::Owner<PxVehicleDrive4W>),
    DriveNW(crate::vehicles::Owner<PxVehicleDriveNW>),
    DriveTank(crate::vehicles::Owner<PxVehicleDriveTank>),
}

impl BPxVehicleHandle {
    pub fn new(vehicle_desc: &BPxVehicle, physics: &mut BPxPhysics, actor: &mut PxRigidDynamic) -> Self {
        let (wheels, wheels_sim_data) = match vehicle_desc {
            BPxVehicle::NoDrive { wheels, wheels_sim_data, .. } => (wheels, wheels_sim_data),
            BPxVehicle::Drive4W { wheels, wheels_sim_data, .. } => (wheels, wheels_sim_data),
            BPxVehicle::DriveNW { wheels, wheels_sim_data, .. } => (wheels, wheels_sim_data),
            BPxVehicle::DriveTank { wheels, wheels_sim_data, .. } => (wheels, wheels_sim_data),
        };

        let mut shape_mapping = HashMap::new();
        for (idx, shape) in actor.get_shapes().into_iter().enumerate() {
            shape_mapping.insert(*shape.get_user_data(), idx as i32);
        }

        match vehicle_desc {
            BPxVehicle::NoDrive { .. } => {
                let mut vehicle: crate::vehicles::Owner<PxVehicleNoDrive> = VehicleNoDrive::new(physics.physics_mut(), actor, wheels_sim_data).unwrap();
                let wheelsim = vehicle.wheels_sim_data_mut();

                for (wheel_id, entity) in wheels.iter().enumerate() {
                    wheelsim.set_wheel_shape_mapping(wheel_id as u32, *shape_mapping.get(entity).unwrap());
                }

                Self::NoDrive(vehicle)
            }
            BPxVehicle::Drive4W { drive_sim_data, .. } => {
                let mut vehicle: crate::vehicles::Owner<PxVehicleDrive4W> = VehicleDrive4W::new(physics.physics_mut(), actor, wheels_sim_data, drive_sim_data.as_ref(), wheels.len() as u32 - 4).unwrap();
                let wheelsim = vehicle.wheels_sim_data_mut();

                for (wheel_id, entity) in wheels.iter().enumerate() {
                    wheelsim.set_wheel_shape_mapping(wheel_id as u32, *shape_mapping.get(entity).unwrap());
                }

                Self::Drive4W(vehicle)
            }
            BPxVehicle::DriveNW { drive_sim_data, .. } => {
                let mut vehicle: crate::vehicles::Owner<PxVehicleDriveNW> = VehicleDriveNW::new(physics.physics_mut(), actor, wheels_sim_data, drive_sim_data.as_ref(), wheels.len() as u32).unwrap();
                let wheelsim = vehicle.wheels_sim_data_mut();

                for (wheel_id, entity) in wheels.iter().enumerate() {
                    wheelsim.set_wheel_shape_mapping(wheel_id as u32, *shape_mapping.get(entity).unwrap());
                }

                Self::DriveNW(vehicle)
            }
            BPxVehicle::DriveTank { drive_sim_data, .. } => {
                let mut vehicle: crate::vehicles::Owner<PxVehicleDriveTank> = VehicleDriveTank::new(physics.physics_mut(), actor, wheels_sim_data, drive_sim_data.as_ref(), wheels.len() as u32).unwrap();
                let wheelsim = vehicle.wheels_sim_data_mut();

                for (wheel_id, entity) in wheels.iter().enumerate() {
                    wheelsim.set_wheel_shape_mapping(wheel_id as u32, *shape_mapping.get(entity).unwrap());
                }

                Self::DriveTank(vehicle)
            }
        }
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
