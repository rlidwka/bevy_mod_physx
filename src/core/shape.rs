//! Defines characteristics of collision shapes (geometry, material).
use std::ptr::drop_in_place;

use bevy::prelude::*;
use physx::prelude::*;
use physx::traits::Class;
use physx_sys::{PxPhysics_createShape_mut, PxShape_release_mut};

use crate::core::geometry::GeometryInner;
use crate::core::scene::SceneRwLock;
use crate::prelude as bpx;
use crate::types::*;

#[derive(Component, Clone)]
pub struct Shape {
    pub geometry: Handle<bpx::Geometry>,
    pub material: Handle<bpx::Material>,
    pub flags: ShapeFlags,
}

impl Default for Shape {
    fn default() -> Self {
        Self {
            geometry: default(),
            material: default(),
            flags: ShapeFlags::SceneQueryShape
                | ShapeFlags::SimulationShape
                | ShapeFlags::Visualization,
        }
    }
}

#[derive(Component)]
pub struct ShapeHandle {
    handle: Option<SceneRwLock<Owner<PxShape>>>,
    // we want to specify outward normal for PxPlane specifically, so need to return transform for this
    pub custom_xform: Transform,
}

impl ShapeHandle {
    pub fn new(px_shape: Owner<PxShape>, custom_xform: Transform) -> Self {
        Self { handle: Some(SceneRwLock::new(px_shape)), custom_xform }
    }

    pub fn create_shape(
        physics: &mut bpx::Physics,
        geometry: &mut bpx::Geometry,
        material: &bpx::Material,
        flags: ShapeFlags,
        user_data: Entity,
    ) -> Self {
        // we want to specify outward normal for PxPlane specifically, so need to return transform for this
        let mut transform = Transform::IDENTITY;

        let geometry_ptr = match &mut geometry.obj {
            GeometryInner::Sphere(geom)  => { geom.as_ptr() },
            GeometryInner::Plane { plane, normal } => {
                transform.rotate(Quat::from_rotation_arc(Vec3::X, *normal));
                plane.as_ptr()
            },
            GeometryInner::Capsule(geom) => { geom.as_ptr() },
            GeometryInner::Box(geom)     => { geom.as_ptr() },
            GeometryInner::ConvexMesh { mesh, scale, flags } => {
                PxConvexMeshGeometry::new(
                    mesh.lock().unwrap().as_mut(),
                    scale,
                    *flags,
                ).as_ptr()
            },
            GeometryInner::TriangleMesh { mesh, scale, flags } => {
                PxTriangleMeshGeometry::new(
                    mesh.lock().unwrap().as_mut(),
                    scale,
                    *flags,
                ).as_ptr()
            },
            GeometryInner::HeightField { mesh, scale, flags } => {
                PxHeightFieldGeometry::new(
                    mesh.lock().unwrap().as_mut(),
                    *flags,
                    scale.scale.y,
                    scale.scale.x,
                    scale.scale.z,
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
                    flags,
                ),
                user_data
            ).unwrap()
        };

        Self::new(shape, transform)
    }
}

impl Drop for ShapeHandle {
    fn drop(&mut self) {
        // TODO: remove this entire drop when this gets fixed:
        // https://github.com/EmbarkStudios/physx-rs/issues/180
        let mut shape = self.handle.take().unwrap();
        unsafe {
            use physx::shape::Shape;
            drop_in_place(shape.get_mut_unsafe().get_user_data_mut());
            PxShape_release_mut(shape.get_mut_unsafe().as_mut_ptr());
        }
        std::mem::forget(shape);
    }
}

impl std::ops::Deref for ShapeHandle {
    type Target = SceneRwLock<Owner<PxShape>>;

    fn deref(&self) -> &Self::Target {
        // TODO: replace with Deref/DerefMut derive when this gets fixed:
        // https://github.com/EmbarkStudios/physx-rs/issues/180
        self.handle.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for ShapeHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // TODO: replace with Deref/DerefMut derive when this gets fixed:
        // https://github.com/EmbarkStudios/physx-rs/issues/180
        self.handle.as_mut().unwrap()
    }
}
