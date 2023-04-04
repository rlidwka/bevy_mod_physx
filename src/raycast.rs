use std::{mem::MaybeUninit, ptr::null_mut};
use bevy::prelude::{Entity, Vec3};
use physx::prelude::*;
use physx::traits::Class;
use physx_sys::{PxSceneQueryExt_raycastSingle, PxHitFlags, PxQueryFilterData_new, PxBase_getConcreteType, PxConcreteType};

use crate::prelude::{*, Scene};

#[derive(Debug)]
pub struct RaycastHit {
    pub actor: Entity,
    pub shape: Entity,
    pub face_index: u32,
    pub flags: PxHitFlags,
    pub position: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub u: f32,
    pub v: f32,
}

pub trait SceneQueryExt {
    fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<RaycastHit>;
}

impl SceneQueryExt for Scene {
    fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<RaycastHit> {
        let scene = self.get();
        let mut raycast_hit = MaybeUninit::uninit();

        if !unsafe {
            PxSceneQueryExt_raycastSingle(
                scene.as_ptr(),
                &origin.to_physx_sys(),
                &direction.to_physx_sys(),
                max_distance,
                PxHitFlags::Default,
                raycast_hit.as_mut_ptr(),
                &PxQueryFilterData_new(),
                null_mut(),
                null_mut(),
            )
        } { return None; }

        // SAFETY: raycastSingle returned true, so we assume buffer is initialized
        let raycast_hit = unsafe { raycast_hit.assume_init() };

        // SAFETY: PxRigidActor is subclass of PxBase
        let actor_type = PxConcreteType::from(unsafe { PxBase_getConcreteType(raycast_hit.actor as *const _) });
        let actor_entity = match actor_type {
            PxConcreteType::RigidDynamic => {
                // SAFETY: assume that every shape in physx scene is created by us,
                // with our prototype and userdata; and that physx returns proper concrete type
                let actor: Owner<crate::PxRigidDynamic> = unsafe { std::mem::transmute(raycast_hit.actor) };
                let entity = *actor.get_user_data();
                // SAFETY: we temporarily create second owned pointer (first one is stored in bevy ECS),
                // so we must drop it until anything bad happens
                std::mem::forget(actor);
                entity
            }
            PxConcreteType::RigidStatic => {
                // SAFETY: assume that every shape in physx scene is created by us,
                // with our prototype and userdata; and that physx returns proper concrete type
                let actor: Owner<crate::PxRigidStatic> = unsafe { std::mem::transmute(raycast_hit.actor) };
                let entity = *actor.get_user_data();
                // SAFETY: we temporarily create second owned pointer (first one is stored in bevy ECS),
                // so we must drop it until anything bad happens
                std::mem::forget(actor);
                entity
            }
            // SAFETY: actor must be either dynamic or static, otherwise physx hierarchy is broken
            _ => unreachable!()
        };

        let shape_entity = {
            // SAFETY: assume that every shape in physx scene is created by us,
            // with our prototype and userdata
            let shape: Owner<crate::PxShape> = unsafe { std::mem::transmute(raycast_hit.shape) };
            let entity = *shape.get_user_data();
            // SAFETY: we temporarily create second owned pointer (first one is stored in bevy ECS),
            // so we must drop it until anything bad happens
            std::mem::forget(shape);
            entity
        };

        Some(RaycastHit {
            actor: actor_entity,
            shape: shape_entity,
            face_index: raycast_hit.faceIndex,
            flags: raycast_hit.flags,
            position: raycast_hit.position.to_bevy(),
            normal: raycast_hit.normal.to_bevy(),
            distance: raycast_hit.distance,
            u: raycast_hit.u,
            v: raycast_hit.v,
        })
    }
}
