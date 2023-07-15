use std::ptr::null;

use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use physx::prelude::*;
use physx::scene::SceneFlags;
use physx::traits::Class;
use physx_sys::{
    PxErrorCode,
    PxScene_lockRead_mut,
    PxScene_lockWrite_mut,
    PxScene_removeArticulation_mut,
    PxScene_unlockRead_mut,
    PxScene_unlockWrite_mut,
};

use super::prelude::*;
use super::{prelude as bpx, PxScene, PxShape};
use crate::callbacks::OnWakeSleep;
use crate::{FoundationDescriptor, SceneDescriptor};

struct ErrorCallback;

impl physx::physics::ErrorCallback for ErrorCallback {
    fn report_error(&self, code: PxErrorCode, message: &str, file: &str, line: u32) {
        bevy::log::error!(target: "bevy_physx", "[{file:}:{line:}] {code:40}: {message:}", code=code as i32);
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Physics(PhysicsFoundation<physx::foundation::DefaultAllocator, PxShape>);

impl Physics {
    pub fn new(foundation_desc: &FoundationDescriptor) -> Self {
        let mut builder = physx::physics::PhysicsFoundationBuilder::default();
        builder.enable_visual_debugger(foundation_desc.visual_debugger);
        builder.with_extensions(foundation_desc.extensions);
        builder.set_pvd_port(foundation_desc.visual_debugger_port);
        if let Some(host) = foundation_desc.visual_debugger_remote.as_ref() {
            builder.set_pvd_host(host);
        }
        builder.set_length_tolerance(foundation_desc.tolerances.length);
        builder.set_speed_tolerance(foundation_desc.tolerances.speed);
        builder.with_error_callback(ErrorCallback);

        let physics = builder.build();

        if physics.is_none() && foundation_desc.visual_debugger {
            // failed to connect, try without debugger
            let mut without_debugger = foundation_desc.clone();
            without_debugger.visual_debugger = false;
            return Self::new(&without_debugger);
        }

        let physics = physics.expect("building PhysX foundation failed");
        Self(physics)
    }
}

#[derive(Resource)]
pub struct Scene {
    scene: SceneRwLock<Owner<PxScene>>,
    use_physx_lock: bool,
    pub(crate) send_sleep_notifies: bool,
}

impl Scene {
    pub fn new(physics: &mut Physics, d: &SceneDescriptor, on_wake_sleep: Option<OnWakeSleep>) -> Self {
        use physx::physics::Physics; // physx trait clashes with our wrapper

        // PxBounds3 doesn't have Clone/Copy, even though it should
        let sanity_bounds = unsafe {
            physx::math::PxBounds3::from(*(std::mem::transmute::<_, &physx_sys::PxBounds3>(&d.sanity_bounds)))
        };

        // not needless match, as it doesn't support Clone/Copy
        #[allow(clippy::needless_match)]
        let simulation_filter_shader = match d.simulation_filter_shader {
            FilterShaderDescriptor::Default => FilterShaderDescriptor::Default,
            FilterShaderDescriptor::Custom(f) => FilterShaderDescriptor::Custom(f),
            FilterShaderDescriptor::CallDefaultFirst(f) => FilterShaderDescriptor::CallDefaultFirst(f),
        };

        let send_sleep_notifies = on_wake_sleep.is_some();

        let scene = physics
            .create(physx::traits::descriptor::SceneDescriptor {
                on_collide: d.on_collision.as_ref().map(|x| x.initialize()),
                on_trigger: d.on_trigger.as_ref().map(|x| x.initialize()),
                on_constraint_break: d.on_constraint_break.as_ref().map(|x| x.initialize()),
                on_wake_sleep: on_wake_sleep.as_ref().map(|x| x.initialize()),
                on_advance: d.on_advance.as_ref().map(|x| x.initialize()),
                gravity: d.gravity.to_physx(),
                kine_kine_filtering_mode: d.kine_kine_filtering_mode,
                static_kine_filtering_mode: d.static_kine_filtering_mode,
                broad_phase_type: d.broad_phase_type,
                limits: d.limits,
                friction_type: d.friction_type,
                solver_type: d.solver_type,
                bounce_threshold_velocity: d.bounce_threshold_velocity,
                friction_offset_threshold: d.friction_offset_threshold,
                ccd_max_separation: d.ccd_max_separation,
                flags: d.flags,
                static_structure: d.static_structure,
                dynamic_structure: d.dynamic_structure,
                dynamic_tree_rebuild_rate_hint: d.dynamic_tree_rebuild_rate_hint,
                scene_query_update_mode: d.scene_query_update_mode,
                solver_batch_size: d.solver_batch_size,
                solver_articulation_batch_size: d.solver_articulation_batch_size,
                nb_contact_data_blocks: d.nb_contact_data_blocks,
                max_nb_contact_data_blocks: d.max_nb_contact_data_blocks,
                max_bias_coefficient: d.max_bias_coefficient,
                contact_report_stream_buffer_size: d.contact_report_stream_buffer_size,
                ccd_max_passes: d.ccd_max_passes,
                ccd_threshold: d.ccd_threshold,
                wake_counter_reset_value: d.wake_counter_reset_value,
                sanity_bounds,
                simulation_filter_shader,
                thread_count: d.thread_count,
                gpu_max_num_partitions: d.gpu_max_num_partitions,
                gpu_compute_version: d.gpu_compute_version,
                ..physx::traits::descriptor::SceneDescriptor::new(())
            })
            .unwrap();

        Self {
            scene: SceneRwLock::new(scene),
            use_physx_lock: d.flags.contains(SceneFlags::RequireRwLock),
            send_sleep_notifies,
        }
    }

    pub fn get(&self) -> SceneRwLockReadGuard<'_, PxScene> {
        let scene = if self.use_physx_lock { Some(self.scene.0.as_ptr() as *mut _) } else { None };
        SceneRwLockReadGuard::new(&self.scene.0, scene)
    }

    pub fn get_mut(&mut self) -> SceneRwLockWriteGuard<'_, PxScene> {
        let scene = if self.use_physx_lock { Some(self.scene.0.as_mut_ptr()) } else { None };
        SceneRwLockWriteGuard::new(&mut self.scene.0, scene)
    }
}

impl Drop for Scene {
    fn drop(&mut self) {
        use physx::prelude::Scene;
        let scene_ptr = unsafe { self.scene.get_mut_unsafe().as_mut_ptr() };
        let articulations = unsafe { self.scene.get_mut_unsafe() }.get_articulations();

        for articulation in articulations {
            unsafe {
                // TODO: articulations themselves are never freed,
                // need to restructure this to avoid memory leaks
                PxScene_removeArticulation_mut(scene_ptr, articulation.as_mut_ptr(), false);
            }
        }
    }
}

pub struct SceneRwLock<T>(T);

impl<T> SceneRwLock<T> {
    // this structure forces user to ensure their system depends on &Scene for read access,
    // and depends on &mut Scene for write access, letting bevy resolve conflicts
    //
    // scene is not really used (apart from eREQUIRE_RW_LOCK mode), but we just need to
    // check that it's there
    //
    // SceneRwLockXXXGuard has to live longer than Scene, but it's not statically checked
    // (we assume that user won't be manually destroying bevy's resources)
    pub fn new(t: T) -> Self {
        Self(t)
    }

    pub fn get(&self, scene: &Scene) -> SceneRwLockReadGuard<'_, T> {
        // this is technically a conversion from *const to *mut, but shouldn't matter here;
        // current physx binding requires mutable scene to establish a lock
        let scene = if scene.use_physx_lock { Some(scene.scene.0.as_ptr() as *mut _) } else { None };
        SceneRwLockReadGuard::new(&self.0, scene)
    }

    pub fn get_mut(&mut self, scene: &mut Scene) -> SceneRwLockWriteGuard<'_, T> {
        let scene = if scene.use_physx_lock { Some(scene.scene.0.as_mut_ptr()) } else { None };
        SceneRwLockWriteGuard::new(&mut self.0, scene)
    }

    /// # Safety
    /// user must ensure that no other code is writing the same data concurrently
    pub unsafe fn get_unsafe(&self) -> &T {
        &self.0
    }

    /// # Safety
    /// user must ensure that no other code is writing the same data concurrently
    pub unsafe fn get_mut_unsafe(&mut self) -> &mut T {
        &mut self.0
    }
}

#[derive(Deref, DerefMut)]
pub struct SceneRwLockReadGuard<'t, T> {
    #[deref]
    #[deref_mut]
    payload: &'t T,
    scene: Option<*mut physx_sys::PxScene>,
}

impl<'t, T> SceneRwLockReadGuard<'t, T> {
    fn new(payload: &'t T, scene: Option<*mut physx_sys::PxScene>) -> Self {
        if let Some(scene) = scene {
            unsafe { PxScene_lockRead_mut(scene, null(), 0); }
        }
        Self { payload, scene }
    }
}

impl<'t, T> Drop for SceneRwLockReadGuard<'t, T> {
    fn drop(&mut self) {
        if let Some(scene) = self.scene {
            unsafe { PxScene_unlockRead_mut(scene) }
        }
    }
}

#[derive(Deref, DerefMut)]
pub struct SceneRwLockWriteGuard<'t, T> {
    #[deref]
    #[deref_mut]
    payload: &'t mut T,
    scene: Option<*mut physx_sys::PxScene>,
}

impl<'t, T> SceneRwLockWriteGuard<'t, T> {
    fn new(payload: &'t mut T, scene: Option<*mut physx_sys::PxScene>) -> Self {
        if let Some(scene) = scene {
            unsafe { PxScene_lockWrite_mut(scene, null(), 0); }
        }
        Self { payload, scene }
    }
}

impl<'t, T> Drop for SceneRwLockWriteGuard<'t, T> {
    fn drop(&mut self) {
        if let Some(scene) = self.scene {
            unsafe { PxScene_unlockWrite_mut(scene) }
        }
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct DefaultMaterial(pub Handle<bpx::Material>);
