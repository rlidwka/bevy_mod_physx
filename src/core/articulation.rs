//! A tree structure of bodies connected by joints that is treated as a unit by the dynamics solver.
use bevy::prelude::*;
use physx::prelude::*;

use crate::core::scene::SceneRwLock;
use crate::types::*;

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
            parent: Entity::PLACEHOLDER,

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
