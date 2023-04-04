use std::ptr::drop_in_place;

use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use physx::prelude::*;
use physx::traits::Class;
use physx_sys::{
    PxShape_release_mut, PxPhysics_createShape_mut, PxFilterData, PxFilterData_new_2, PxMeshScale_new_3,
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

    pub fn create_shape(physics: &mut bpx::Physics, geometry: &mut bpx::Geometry, material: &mut bpx::Material, user_data: Entity) -> (Self, Transform) {
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
            GeometryInner::ConvexMesh { mesh, scale, rotation, flags } => {
                PxConvexMeshGeometry::new(
                    mesh.lock().unwrap().as_mut(),
                    unsafe { &PxMeshScale_new_3(scale.to_physx_sys().as_ptr(), rotation.to_physx().as_ptr()) },
                    *flags,
                ).as_ptr()
            },
            GeometryInner::TriangleMesh { mesh, scale, rotation, flags } => {
                PxTriangleMeshGeometry::new(
                    mesh.lock().unwrap().as_mut(),
                    unsafe { &PxMeshScale_new_3(scale.to_physx_sys().as_ptr(), rotation.to_physx().as_ptr()) },
                    *flags,
                ).as_ptr()
            },
            GeometryInner::HeightField { mesh, scale, flags } => {
                PxHeightFieldGeometry::new(
                    mesh.lock().unwrap().as_mut(),
                    *flags,
                    scale.y,
                    scale.x,
                    scale.z,
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
                    ShapeFlags::SceneQueryShape | ShapeFlags::SimulationShape | ShapeFlags::Visualization,
                ),
                user_data
            ).unwrap()
        };

        (Self::new(shape), transform)
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
