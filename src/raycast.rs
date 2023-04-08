use std::{mem::MaybeUninit, ptr::null_mut};
use bevy::prelude::{Entity, Vec3};
use physx::traits::Class;
use physx_sys::{PxSceneQueryExt_raycastSingle, PxHitFlags, PxQueryFilterData_new};

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

        Some(RaycastHit {
            actor: unsafe { self.get_actor_entity_from_ptr(raycast_hit.actor) },
            shape: unsafe { self.get_shape_entity_from_ptr(raycast_hit.shape) },
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
