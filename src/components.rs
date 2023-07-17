use std::ptr::drop_in_place;

use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use physx::prelude::*;
use physx::traits::Class;
use physx_sys::{PxMeshScale_new_3, PxPhysics_createShape_mut, PxShape_release_mut};

use crate::assets::GeometryInner;
use crate::prelude::{self as bpx, IntoPxQuat, IntoPxVec3};
use crate::resources::SceneRwLock;
use crate::types::*;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RigidBody {
    Dynamic,
    Static,
    ArticulationLink,
}

#[derive(Default, Clone, Copy)]
pub enum ArticulationJointMotion {
    #[default]
    Locked,
    Free,
    Limited { min: f32, max: f32 },
}

#[derive(Component, Clone)]
pub struct ArticulationJoint {
    pub parent: Entity,
    pub joint_type: ArticulationJointType,
    pub motion_twist: ArticulationJointMotion,
    pub motion_swing1: ArticulationJointMotion,
    pub motion_swing2: ArticulationJointMotion,
    pub motion_x: ArticulationJointMotion,
    pub motion_y: ArticulationJointMotion,
    pub motion_z: ArticulationJointMotion,
    pub parent_pose: Transform,
    pub child_pose: Transform,
    pub max_joint_velocity: f32,
    pub friction_coefficient: f32,
}

impl Default for ArticulationJoint {
    fn default() -> Self {
        Self {
            // Making invalid Entity here, because we want Default trait for creating
            // ArticulationJoint with ..default(), but user must always set this.
            //
            // If user leaves it as is, entity won't be found, and user will get runtime error.
            //
            // Default invalid Handle exists, so why not have default invalid Entity?
            // All accesses to it are checked anyways.
            //
            parent: unsafe { std::mem::transmute(u64::MAX) },

            // For Fixed joints all motions must be locked (0 degrees of freedom).
            //
            // For Prismatic joints there is one motion (x, y or z) that's not locked
            // (1 degree of freedom).
            //
            // For Revolute/RevoluteUnwrapped joints there is one motion (twist, swing1
            // or swing2), that's not locked (1 degree of freedom).
            //
            // For Spherical joints twist, swing1, or swing2 can be in any state
            // (up to 3 degrees of freedom).
            //
            joint_type: ArticulationJointType::Fix,
            motion_twist: default(),
            motion_swing1: default(),
            motion_swing2: default(),
            motion_x: default(),
            motion_y: default(),
            motion_z: default(),

            // Pose should always be set by user.
            // Parent and child pose can in theory be set at runtime, but it results
            // in funky behavior in my testing, so we only set it on creation.
            parent_pose: default(),
            child_pose: default(),

            // PhysX default values.
            max_joint_velocity: 100.,
            friction_coefficient: 0.5,
        }
    }
}

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
            flags: ShapeFlag::SceneQueryShape
                | ShapeFlag::SimulationShape
                | ShapeFlag::Visualization,
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
        material: &mut bpx::Material,
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
                    physx_sys::PxShapeFlags { mBits: flags.bits() as u8 },
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

#[derive(Component, Deref, DerefMut)]
pub struct RigidDynamicHandle {
    #[deref]
    #[deref_mut]
    handle: SceneRwLock<Owner<PxRigidDynamic>>,
    // used for change detection
    pub predicted_gxform: GlobalTransform,
}

impl RigidDynamicHandle {
    pub fn new(px_rigid_dynamic: Owner<PxRigidDynamic>, predicted_gxform: GlobalTransform) -> Self {
        Self { handle: SceneRwLock::new(px_rigid_dynamic), predicted_gxform }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct RigidStaticHandle {
    #[deref]
    #[deref_mut]
    handle: SceneRwLock<Owner<PxRigidStatic>>,
    // used for change detection
    pub predicted_gxform: GlobalTransform,
}

impl RigidStaticHandle {
    pub fn new(px_rigid_static: Owner<PxRigidStatic>, predicted_gxform: GlobalTransform) -> Self {
        Self { handle: SceneRwLock::new(px_rigid_static), predicted_gxform }
    }
}

#[derive(Component)]
pub struct ArticulationRootHandle {
    handle: Option<SceneRwLock<Owner<PxArticulationReducedCoordinate>>>,
}

impl ArticulationRootHandle {
    pub fn new(px_articulation_root: Owner<PxArticulationReducedCoordinate>) -> Self {
        Self { handle: Some(SceneRwLock::new(px_articulation_root)) }
    }
}

impl std::ops::Deref for ArticulationRootHandle {
    type Target = SceneRwLock<Owner<PxArticulationReducedCoordinate>>;

    fn deref(&self) -> &Self::Target {
        self.handle.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for ArticulationRootHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.handle.as_mut().unwrap()
    }
}

impl Drop for ArticulationRootHandle {
    fn drop(&mut self) {
        // TODO: it needs to be removed from scene first
        std::mem::forget(self.handle.take());
    }
}

#[derive(Component)]
pub struct ArticulationLinkHandle {
    handle: Option<SceneRwLock<Owner<PxArticulationLink>>>,
    // used for change detection
    pub predicted_gxform: GlobalTransform,
}

impl ArticulationLinkHandle {
    pub fn new(px_articulation_link: Owner<PxArticulationLink>, predicted_gxform: GlobalTransform) -> Self {
        Self { handle: Some(SceneRwLock::new(px_articulation_link)), predicted_gxform }
    }
}

impl std::ops::Deref for ArticulationLinkHandle {
    type Target = SceneRwLock<Owner<PxArticulationLink>>;

    fn deref(&self) -> &Self::Target {
        self.handle.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for ArticulationLinkHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.handle.as_mut().unwrap()
    }
}

impl Drop for ArticulationLinkHandle {
    fn drop(&mut self) {
        // avoid calling release, because we cannot release an articulation link while it's attached to a scene;
        // TODO: this should be released from ArticulationRoot
        std::mem::forget(self.handle.take());
    }
}
