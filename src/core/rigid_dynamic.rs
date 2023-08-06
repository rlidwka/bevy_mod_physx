//! Dynamic rigid simulation object in the Physics SDK.
use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use physx::prelude::*;

use crate::core::scene::SceneRwLock;
use crate::types::*;

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
