use std::{ptr::drop_in_place, ops::{Deref, DerefMut}};

use bevy::prelude::*;
use physx::prelude::*;
use physx::traits::{Class, PxFlags};
use physx_sys::{PxShape_release_mut, PxPhysics_createShape_mut, PxConvexMeshGeometryFlag, PxConvexMeshGeometryFlags, PxMeshGeometryFlags, PxMeshGeometryFlag, PxMeshScale_new};
use super::{PxRigidStatic, PxRigidDynamic, PxShape};
use super::assets::{BPxGeometry, BPxMaterial};
use super::resources::BPxPhysics;

pub struct BPxActorDynamic {
    pub geometry: Handle<BPxGeometry>,
    pub material: Handle<BPxMaterial>,
    pub density: f32,
    pub shape_offset: Transform,
}

impl Default for BPxActorDynamic {
    fn default() -> Self {
        Self {
            geometry: Default::default(),
            material: Default::default(),
            density: 1.0,
            shape_offset: Transform::IDENTITY,
        }
    }
}

pub struct BPxActorStatic {
    pub geometry: Handle<BPxGeometry>,
    pub material: Handle<BPxMaterial>,
    pub shape_offset: Transform,
}

impl Default for BPxActorStatic {
    fn default() -> Self {
        Self {
            geometry: Default::default(),
            material: Default::default(),
            shape_offset: Transform::IDENTITY,
        }
    }
}

#[derive(Component, Clone)]
pub enum BPxActor {
    Dynamic { density: f32 },
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

    pub fn create_shape(physics: &mut BPxPhysics, geometry: &mut BPxGeometry, material: &mut BPxMaterial) -> Self {
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
                ()
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
