pub mod type_bridge;

use bevy::prelude::*;
use physx::actor::ActorMap;
use physx::prelude::*;

use crate::physx_extras::ActorMapExtras;
use crate::{PxArticulationLink, PxRigidDynamic, PxRigidStatic};

/// # Safety
/// User must ensure that pointer is valid and created by bevy_physx crate
/// with corresponding prototype and userdata.
pub unsafe fn get_actor_entity_from_ptr(actor: *const physx_sys::PxRigidActor) -> Entity {
    let actor_map = &*(actor as *const ActorMap<PxArticulationLink, PxRigidStatic, PxRigidDynamic>);

    actor_map.cast_map_ref(
        |articulation| *articulation.get_user_data(),
        |rstatic| *rstatic.get_user_data(),
        |rdynamic| *rdynamic.get_user_data(),
    )
}

/// # Safety
/// User must ensure that pointer is valid and created by bevy_physx crate
/// with corresponding prototype and userdata.
pub unsafe fn get_shape_entity_from_ptr(shape: *const physx_sys::PxShape) -> Entity {
    let shape = &*(shape as *const crate::PxShape);
    *shape.get_user_data()
}
