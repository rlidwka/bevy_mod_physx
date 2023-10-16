


use bevy::prelude::*;
use physx::traits::Class;
use crate::prelude::{Scene, *};

#[derive(Component, Debug, PartialEq, Clone, Copy, Reflect, Default)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
/// Set lock flags for a dynamic rigid body.
pub struct RigidDynamicLockFlags {
    pub lock_linear_x: bool,
    pub lock_linear_y: bool,
    pub lock_linear_z: bool,
    pub lock_angular_x: bool,
    pub lock_angular_y: bool,
    pub lock_angular_z: bool,
}



pub struct LockFlagsPlugin;

impl Plugin for LockFlagsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RigidDynamicLockFlags>();
        app.add_systems(PhysicsSchedule, lock_flags_sync.in_set(PhysicsSet::Sync));
    }
}

pub fn lock_flags_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<(
        Option<&mut RigidDynamicHandle>,
        Ref<RigidDynamicLockFlags>,
    ),
    Or<(
        Added<RigidDynamicHandle>,
        Changed<RigidDynamicLockFlags>,
    )>>
    ) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (dynamic, lock_flags) in actors.iter_mut() {

        let actor_handle = if let Some(mut actor) = dynamic {
            actor.get_mut(&mut scene).as_mut_ptr()
        } else {
            if !lock_flags.is_added() {
                bevy::log::warn!("RigidDynamicLockFlags component exists, but it's neither a rigid dynamic nor articulation link");
            }
            continue;
        };

        unsafe {
            info!("lock_flags_sync");
            physx_sys::PxRigidDynamic_setRigidDynamicLockFlag_mut(actor_handle, physx_sys::PxRigidDynamicLockFlag::LockLinearX, lock_flags.lock_linear_x);
            physx_sys::PxRigidDynamic_setRigidDynamicLockFlag_mut(actor_handle, physx_sys::PxRigidDynamicLockFlag::LockLinearY, lock_flags.lock_linear_y);
            physx_sys::PxRigidDynamic_setRigidDynamicLockFlag_mut(actor_handle, physx_sys::PxRigidDynamicLockFlag::LockLinearZ, lock_flags.lock_linear_z);

            physx_sys::PxRigidDynamic_setRigidDynamicLockFlag_mut(actor_handle, physx_sys::PxRigidDynamicLockFlag::LockAngularX, lock_flags.lock_angular_x);
            physx_sys::PxRigidDynamic_setRigidDynamicLockFlag_mut(actor_handle, physx_sys::PxRigidDynamicLockFlag::LockAngularY, lock_flags.lock_angular_y);
            physx_sys::PxRigidDynamic_setRigidDynamicLockFlag_mut(actor_handle, physx_sys::PxRigidDynamicLockFlag::LockAngularZ, lock_flags.lock_angular_z);

        }

    } 
}