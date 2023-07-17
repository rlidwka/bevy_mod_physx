use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::ptr::{drop_in_place, null_mut};

use bevy::prelude::*;
use physx::rigid_actor::RigidActor;
use physx::traits::Class;
use physx_sys::{
    create_raycast_filter_callback,
    create_raycast_filter_callback_func,
    PxQueryFilterCallback,
    PxQueryFilterCallback_delete,
    PxQueryFilterData,
    PxQueryFilterData_new,
    PxQueryFlag,
    PxSceneQueryExt_raycastSingle_mut,
    RaycastHitCallback,
};

use crate::prelude::{Scene, *};
use crate::utils::{get_actor_entity_from_ptr, get_shape_entity_from_ptr};

#[derive(Debug)]
pub struct RaycastHit {
    pub actor: Entity,
    pub shape: Entity,
    pub face_index: u32,
    //pub flags: PxHitFlags,
    pub position: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

pub struct SceneQueryFilter {
    filter_data: PxQueryFilterData,
    pre_filter_callback: Option<*mut PxQueryFilterCallback>, // owned
}

impl SceneQueryFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ignore<T: RigidActor>(actor: &T) -> Self {
        let mut result = Self::new();
        result.filter_data.flags.mBits |= PxQueryFlag::ePREFILTER as u16;
        result.pre_filter_callback = Some(unsafe {
            create_raycast_filter_callback(actor.as_ptr())
        });
        result
    }

    // false positive: https://github.com/rust-lang/rust-clippy/issues/3045
    // userdata deref will be done in user function, this function is safe
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn callback(callback: RaycastHitCallback, userdata: *mut c_void) -> Self {
        let mut result = Self::new();
        result.filter_data.flags.mBits |= PxQueryFlag::ePREFILTER as u16;
        result.pre_filter_callback = Some(unsafe {
            create_raycast_filter_callback_func(callback, userdata)
        });
        result
    }

    pub fn without_static(mut self) -> Self {
        self.filter_data.flags.mBits &= PxQueryFlag::eSTATIC as u16;
        self
    }

    pub fn without_dynamic(mut self) -> Self {
        self.filter_data.flags.mBits &= PxQueryFlag::eDYNAMIC as u16;
        self
    }
}

impl Default for SceneQueryFilter {
    fn default() -> Self {
        Self {
            filter_data: unsafe { PxQueryFilterData_new() },
            pre_filter_callback: None,
        }
    }
}

impl Drop for SceneQueryFilter {
    fn drop(&mut self) {
        if let Some(ptr) = self.pre_filter_callback.take() {
            unsafe { PxQueryFilterCallback_delete(ptr) };
            unsafe { drop_in_place(ptr); }
        }
    }
}

pub trait SceneQueryExt {
    fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32, filter: &SceneQueryFilter) -> Option<RaycastHit>;
}

impl SceneQueryExt for Scene {
    fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32, filter: &SceneQueryFilter) -> Option<RaycastHit> {
        let scene = self.get();
        let mut raycast_hit = MaybeUninit::uninit();

        if !unsafe {
            PxSceneQueryExt_raycastSingle_mut(
                scene.as_ptr(),
                &origin.to_physx_sys(),
                &direction.to_physx_sys(),
                max_distance,
                physx_sys::PxSceneQueryFlags { mBits: physx_sys::PxHitFlag::eDEFAULT as u16 },
                raycast_hit.as_mut_ptr(),
                &filter.filter_data as *const _ as *const _,
                filter.pre_filter_callback.unwrap_or(null_mut()) as *mut _,
                null_mut(),
            )
        } { return None; }

        // SAFETY: raycastSingle returned true, so we assume buffer is initialized
        let raycast_hit = unsafe { raycast_hit.assume_init() };

        Some(RaycastHit {
            actor: unsafe { get_actor_entity_from_ptr(raycast_hit.actor) },
            shape: unsafe { get_shape_entity_from_ptr(raycast_hit.shape) },
            face_index: raycast_hit.faceIndex,
            //flags: raycast_hit.flags,
            position: raycast_hit.position.to_bevy(),
            normal: raycast_hit.normal.to_bevy(),
            distance: raycast_hit.distance,
        })
    }
}
