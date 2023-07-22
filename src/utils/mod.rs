pub mod type_bridge;

use bevy::prelude::*;
use physx::prelude::*;
use physx_sys::PxBase_getConcreteType;

use crate::types::*;

/// # Safety
/// User must ensure that pointer is valid and created by bevy_mod_physx crate
/// with corresponding prototype and userdata.
///
/// When resolving collisions, you should check PxContactPairFlags::RemovedActorX
/// before executing this function.
pub unsafe fn get_actor_entity_from_ptr(actor: *const physx_sys::PxRigidActor) -> Entity {
    // SAFETY: PxRigidActor is subclass of PxBase
    let actor_type = ConcreteType::from(unsafe { PxBase_getConcreteType(actor as *const _) });
    // NOTE: we don't use PxActor_getType here (and physx-rs ActorMap) because
    // PxBase_getConcreteType is more robust. For example, if user tries to get
    // entity for just removed actor (which he shouldn't), PxActor_getType
    // will crash, but PxBase_getConcreteType will still work probably.
    match actor_type {
        ConcreteType::RigidDynamic => {
            // SAFETY: assume that every shape in physx scene is created by us,
            // with our prototype and userdata; and that physx returns proper concrete type
            let actor: &PxRigidDynamic = unsafe { &*(actor as *const _) };
            let entity = *actor.get_user_data();
            entity
        }
        ConcreteType::RigidStatic => {
            // SAFETY: assume that every shape in physx scene is created by us,
            // with our prototype and userdata; and that physx returns proper concrete type
            let actor: &PxRigidStatic = unsafe { &*(actor as *const _) };
            let entity = *actor.get_user_data();
            entity
        }
        ConcreteType::ArticulationLink => {
            // SAFETY: assume that every shape in physx scene is created by us,
            // with our prototype and userdata; and that physx returns proper concrete type
            let actor: &PxArticulationLink = unsafe { &*(actor as *const _) };
            let entity = *actor.get_user_data();
            entity
        }
        // SAFETY: actor must be either dynamic, static, or articulation
        _ => unreachable!()
    }
}

/// # Safety
/// User must ensure that pointer is valid and created by bevy_mod_physx crate
/// with corresponding prototype and userdata.
///
/// When resolving collisions, you should check PxContactPairFlags::RemovedShapeX
/// before executing this function.
pub unsafe fn get_shape_entity_from_ptr(shape: *const physx_sys::PxShape) -> Entity {
    let shape = &*(shape as *const PxShape);
    *shape.get_user_data()
}
