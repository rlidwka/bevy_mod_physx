//! Static rigid simulation object in the Physics SDK.
use bevy::prelude::*;
use physx::prelude::*;

use crate::core::scene::SceneRwLock;
use crate::types::*;

#[derive(Component, Deref, DerefMut)]
pub struct RigidStaticHandle {
    #[deref]
    handle: SceneRwLock<Owner<PxRigidStatic>>,
    // used for change detection
    pub predicted_gxform: GlobalTransform,
}

impl RigidStaticHandle {
    pub fn new(px_rigid_static: Owner<PxRigidStatic>, predicted_gxform: GlobalTransform) -> Self {
        Self { handle: SceneRwLock::new(px_rigid_static), predicted_gxform }
    }
}
