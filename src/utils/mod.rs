pub mod type_bridge;

use bevy::prelude::*;
use physx::prelude::*;
use physx_sys::PxBase_getConcreteType;

/// # Safety
/// User must ensure that pointer is valid.
pub unsafe fn get_actor_entity_from_ptr(actor: *const physx_sys::PxRigidActor) -> Entity {
    // SAFETY: PxRigidActor is subclass of PxBase
    let actor_type = ConcreteType::from(unsafe { PxBase_getConcreteType(actor as *const _) });
    match actor_type {
        ConcreteType::RigidDynamic => {
            // SAFETY: assume that every shape in physx scene is created by us,
            // with our prototype and userdata; and that physx returns proper concrete type
            let actor: &mut crate::PxRigidDynamic = unsafe { &mut *(actor as *mut _) };
            let entity = *actor.get_user_data();
            entity
        }
        ConcreteType::RigidStatic => {
            // SAFETY: assume that every shape in physx scene is created by us,
            // with our prototype and userdata; and that physx returns proper concrete type
            let actor: &mut crate::PxRigidStatic = unsafe { &mut *(actor as *mut _) };
            let entity = *actor.get_user_data();
            entity
        }
        ConcreteType::ArticulationLink => {
            // SAFETY: assume that every shape in physx scene is created by us,
            // with our prototype and userdata; and that physx returns proper concrete type
            let actor: &mut crate::PxArticulationLink = unsafe { &mut *(actor as *mut _) };
            let entity = *actor.get_user_data();
            entity
        }
        // SAFETY: actor must be either dynamic, static, or articulation
        // TODO: use physx-rs ActorMap to do the conversion here
        _ => unreachable!()
    }
}

/// # Safety
/// User must ensure that pointer is valid.
pub unsafe fn get_shape_entity_from_ptr(shape: *const physx_sys::PxShape) -> Entity {
    // SAFETY: assume that every shape in physx scene is created by us,
    // with our prototype and userdata
    let shape: Owner<crate::PxShape> = unsafe { std::mem::transmute(&*shape) };
    let entity = *shape.get_user_data();
    // SAFETY: we temporarily create second owned pointer (first one is stored in bevy ECS),
    // so we must drop it until anything bad happens
    std::mem::forget(shape);
    entity
}
