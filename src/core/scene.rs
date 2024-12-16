//! A scene is a collection of bodies and constraints which can interact.
use std::ptr::null;

use bevy::prelude::*;
use physx::prelude::*;
use physx::scene::{
    FrictionType,
    SceneFlags,
    SceneLimits,
    SceneQueryUpdateMode,
};
use physx::traits::Class;
use physx_sys::{
    PxScene_lockRead_mut,
    PxScene_lockWrite_mut,
    PxScene_removeArticulation_mut,
    PxScene_unlockRead_mut,
    PxScene_unlockWrite_mut,
};

use crate::prelude::{self as bpx, *};
use crate::types::*;

#[derive(Resource)]
/// A scene is a collection of bodies and constraints which can interact.
pub struct Scene {
    scene: SceneRwLock<Owner<PxScene>>,
    use_physx_lock: bool,
    pub(crate) send_sleep_notifies: bool,
}

impl Scene {
    pub fn new(physics: &mut bpx::Physics, d: &SceneDescriptor, on_wake_sleep: Option<OnWakeSleep>) -> Self {
        use physx::physics::Physics; // physx trait clashes with our wrapper

        // PxBounds3 doesn't have Clone/Copy, even though it should
        let sanity_bounds = unsafe {
            std::mem::transmute_copy(&d.sanity_bounds)
            //physx::math::PxBounds3::from(*(std::mem::transmute::<&physx::math::PxBounds3, &physx_sys::PxBounds3>(&d.sanity_bounds)))
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

impl<T> Drop for SceneRwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        if let Some(scene) = self.scene {
            unsafe { PxScene_unlockRead_mut(scene) }
        }
    }
}

#[derive(Deref, DerefMut)]
pub struct SceneRwLockWriteGuard<'t, T> {
    #[deref]
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

impl<T> Drop for SceneRwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        if let Some(scene) = self.scene {
            unsafe { PxScene_unlockWrite_mut(scene) }
        }
    }
}

/// Descriptor class for creating a scene.
pub struct SceneDescriptor {
    /// This is called when certain contact events occur.
    ///
    /// The method will be called for a pair of actors if one of the colliding
    /// shape pairs requested contact notification. You request which events
    /// are reported using the filter shader/callback mechanism.
    ///
    /// Do not keep references to the passed objects, as they will be invalid
    /// after this function returns.
    pub on_collision: Option<OnCollision>,

    /// This is called with the current trigger pair events.
    ///
    /// Shapes which have been marked as triggers using [ShapeFlag::TriggerShape]
    /// will send events according to the pair flag specification in the filter shader.
    pub on_trigger: Option<OnTrigger>,

    /// This is called when a breakable constraint breaks.
    pub on_constraint_break: Option<OnConstraintBreak>,

    //pub on_wake_sleep: Option<callbacks::OnWakeSleep>, // built-in callback

    /// Provides early access to the new pose of moving rigid bodies.
    ///
    /// When this call occurs, rigid bodies having the [RigidBodyFlag::EnablePoseIntegrationPreview]
    /// flag set, were moved by the simulation and their new poses can be accessed
    /// through the provided buffers.
    pub on_advance: Option<OnAdvance>,
    /// Gravity vector. In bevy plugin, it is set to `Vec3(0, -9.81, 0)` by default.
    pub gravity: Vec3,
    /// Filtering mode for kinematic-kinematic pairs in the broadphase.
    ///
    /// Default: [PairFilteringMode::Suppress]
    pub kine_kine_filtering_mode: PairFilteringMode,
    /// Filtering mode for static-kinematic pairs in the broadphase.
    ///
    /// Default: [PairFilteringMode::Suppress]
    pub static_kine_filtering_mode: PairFilteringMode,
    /// Selects the broad-phase algorithm to use.
    ///
    /// Default: [BroadPhaseType::Pabp]
    pub broad_phase_type: BroadPhaseType,
    /// Expected scene limits.
    pub limits: SceneLimits,
    /// Selects the friction algorithm to use for simulation.
    ///
    /// Default: [FrictionType::Patch]
    pub friction_type: FrictionType,
    /// Selects the solver algorithm to use.
    ///
    /// Default: [SolverType::Pgs]
    pub solver_type: SolverType,
    /// A contact with a relative velocity below this will not bounce.
    ///
    /// A typical value for simulation. stability is about 0.2 * gravity.
    ///
    /// Default: 0.2 * TolerancesScale::speed\
    /// Range: (0, PX_MAX_F32)
    pub bounce_threshold_velocity: f32,
    /// A threshold of contact separation distance used to decide if a contact
    /// point will experience friction forces.
    ///
    /// Default: 0.04 * PxTolerancesScale::length\
    /// Range: [0, PX_MAX_F32)
    pub friction_offset_threshold: f32,
    /// A threshold for speculative CCD.
    ///
    /// Used to control whether bias, restitution or a combination of the two are
    /// used to resolve the contacts.
    ///
    /// Default: 0.04 * PxTolerancesScale::length\
    /// Range: [0, PX_MAX_F32)
    pub ccd_max_separation: f32,
    /// Flags used to select scene options.
    ///
    /// Default: [SceneFlag::EnablePcm]
    pub flags: SceneFlags,
    /// Defines the structure used to store static objects (PxRigidStatic actors).
    ///
    /// There are usually a lot more static actors than dynamic actors in a scene,
    /// so they are stored in a separate structure. The idea is that when dynamic
    /// actors move each frame, the static structure remains untouched and does
    /// not need updating.
    ///
    /// Default: [PruningStructureType::DynamicAabbTree]
    pub static_structure: PruningStructureType,
    /// Defines the structure used to store dynamic objects (non-PxRigidStatic actors).
    ///
    /// Default: [PruningStructureType::DynamicAabbTree]
    pub dynamic_structure: PruningStructureType,
    /// Hint for how much work should be done per simulation frame to rebuild
    /// the pruning structures.
    ///
    /// This parameter gives a hint on the distribution of the workload for
    /// rebuilding the dynamic AABB tree pruning structure
    /// [PruningStructureType::DynamicAabbTree]. It specifies the desired number
    /// of simulation frames the rebuild process should take. Higher values will
    /// decrease the workload per frame but the pruning structure will get more
    /// and more outdated the longer the rebuild takes (which can make scene
    /// queries less efficient).
    ///
    /// Default: 100\
    /// Range: [4, PX_MAX_U32)
    pub dynamic_tree_rebuild_rate_hint: u32,
    /// Defines the scene query update mode.
    ///
    /// Default: [SceneQueryUpdateMode::BuildEnabledCommitEnabled]
    pub scene_query_update_mode: SceneQueryUpdateMode,
    /// Defines the number of actors required to spawn a separate rigid body
    /// solver island task chain.
    ///
    /// This parameter defines the minimum number of actors required to spawn
    /// a separate rigid body solver task chain. Setting a low value will potentially
    /// cause more task chains to be generated. This may result in the overhead of
    /// spawning tasks can become a limiting performance factor. Setting a high value
    /// will potentially cause fewer islands to be generated. This may reduce thread
    /// scaling (fewer task chains spawned) and may detrimentally affect performance
    /// if some bodies in the scene have large solver iteration counts because all
    /// constraints in a given island are solved by the maximum number of solver
    /// iterations requested by any body in the island.
    ///
    /// Note that a rigid body solver task chain is spawned as soon as either
    /// a sufficient number of rigid bodies or articulations are batched together.
    ///
    /// Default: 128
    pub solver_batch_size: u32,
    /// Defines the number of articulations required to spawn a separate rigid body
    /// solver island task chain.
    ///
    /// This parameter defines the minimum number of articulations required to spawn
    /// a separate rigid body solver task chain. Setting a low value will potentially
    /// cause more task chains to be generated. This may result in the overhead of
    /// spawning tasks can become a limiting performance factor. Setting a high value
    /// will potentially cause fewer islands to be generated. This may reduce thread
    /// scaling (fewer task chains spawned) and may detrimentally affect performance
    /// if some bodies in the scene have large solver iteration counts because all
    /// constraints in a given island are solved by the maximum number of solver
    /// iterations requested by any body in the island.
    ///
    /// Note that a rigid body solver task chain is spawned as soon as either
    /// a sufficient number of rigid bodies or articulations are batched together.
    ///
    /// Default: 16
    pub solver_articulation_batch_size: u32,
    /// Setting to define the number of 16K blocks that will be initially reserved
    /// to store contact, friction, and contact cache data.
    ///
    /// This is the number of 16K memory blocks that will be automatically allocated
    /// from the user allocator when the scene is instantiated. Further 16k memory
    /// blocks may be allocated during the simulation up to maxNbContactDataBlocks.
    ///
    /// Default: 0\
    /// Range: [0, PX_MAX_U32]
    pub nb_contact_data_blocks: u32,
    /// Setting to define the maximum number of 16K blocks that can be allocated to
    /// store contact, friction, and contact cache data.
    ///
    /// As the complexity of a scene increases, the SDK may require to allocate new
    /// 16k blocks in addition to the blocks it has already allocated. This variable
    /// controls the maximum number of blocks that the SDK can allocate.
    ///
    /// In the case that the scene is sufficiently complex that all the permitted
    /// 16K blocks are used, contacts will be dropped and a warning passed to the
    /// error stream.
    ///
    /// If a warning is reported to the error stream to indicate the number of 16K
    /// blocks is insufficient for the scene complexity then the choices are either
    /// (i) re-tune the number of 16K data blocks until a number is found that is
    /// sufficient for the scene complexity, (ii) to simplify the scene or
    /// (iii) to opt to not increase the memory requirements of physx and accept
    /// some dropped contacts.
    ///
    /// Default: 65536\
    /// Range: [0, PX_MAX_U32]
    pub max_nb_contact_data_blocks: u32,
    /// The maximum bias coefficient used in the constraint solver.
    ///
    /// When geometric errors are found in the constraint solver, either as a result
    /// of shapes penetrating or joints becoming separated or violating limits, a bias
    /// is introduced in the solver position iterations to correct these errors.
    /// This bias is proportional to 1/dt, meaning that the bias becomes increasingly
    /// strong as the time-step passed to PxScene::simulate(…) becomes smaller. This
    /// coefficient allows the application to restrict how large the bias coefficient is,
    /// to reduce how violent error corrections are. This can improve simulation quality
    /// in cases where either variable time-steps or extremely small time-steps are used.
    ///
    /// Default: PX_MAX_F32\
    /// Range: [0, PX_MAX_F32]
    pub max_bias_coefficient: f32,
    /// Size of the contact report stream (in bytes).
    ///
    /// The contact report stream buffer is used during the simulation to store all
    /// the contact reports. If the size is not sufficient, the buffer will grow by
    /// a factor of two. It is possible to disable the buffer growth by setting the
    /// flag [SceneFlag::DisableContactReportBufferResize]. In that case the buffer
    /// will not grow but contact reports not stored in the buffer will not get sent
    /// in the contact report callbacks.
    ///
    /// Default: 8192\
    /// Range: (0, PX_MAX_U32]
    pub contact_report_stream_buffer_size: u32,
    /// Maximum number of CCD passes.
    ///
    /// The CCD performs multiple passes, where each pass every object advances to its time
    /// of first impact. This value defines how many passes the CCD system should perform.
    ///
    /// Default: 1\
    /// Range: [1, PX_MAX_U32]
    pub ccd_max_passes: u32,
    /// CCD threshold.
    ///
    /// CCD performs sweeps against shapes if and only if the relative motion of
    /// the shapes is fast-enough that a collision would be missed by the discrete
    /// contact generation. However, in some circumstances, e.g. when the environment
    /// is constructed from large convex shapes, this approach may produce undesired
    /// simulation artefacts. This parameter defines the minimum relative motion that
    /// would be required to force CCD between shapes. The smaller of this value and
    /// the sum of the thresholds calculated for the shapes involved will be used.
    ///
    /// Default: PX_MAX_F32\
    /// Range: [Eps, PX_MAX_F32]
    pub ccd_threshold: f32,
    /// The wake counter reset value.
    /// Calling wakeUp() on objects which support sleeping will set their wake counter
    /// value to the specified reset value.
    ///
    /// Default: 0.4 (which corresponds to 20 frames for a time step of 0.02)\
    /// Range: (0, PX_MAX_F32)
    pub wake_counter_reset_value: f32,
    /// The bounds used to sanity check user-set positions of actors and articulation links.
    ///
    /// These bounds are used to check the position values of rigid actors inserted
    /// into the scene, and positions set for rigid actors already within the scene.
    ///
    /// Default: (-PX_MAX_BOUNDS_EXTENTS, PX_MAX_BOUNDS_EXTENTS) on each axis\
    /// Range: any valid [PxBounds3]
    pub sanity_bounds: PxBounds3,

    /// The custom filter shader to use for collision filtering.
    pub simulation_filter_shader: FilterShaderDescriptor,

    pub thread_count: u32,
    /// Limitation for the partitions in the GPU dynamics pipeline.
    pub gpu_max_num_partitions: u32,
    //pub gpu_compute_version: u32, // according to physx docs, shouldn't modify this
}

impl Default for SceneDescriptor {
    fn default() -> Self {
        let d = physx::traits::descriptor::SceneDescriptor::<
            (), PxArticulationLink, PxRigidStatic, PxRigidDynamic,
            PxArticulationReducedCoordinate,
            OnCollision, OnTrigger, OnConstraintBreak,
            OnWakeSleep, OnAdvance
        >::new(());

        SceneDescriptor {
            on_collision: None,
            on_trigger: None,
            on_constraint_break: None,
            on_advance: None,
            //on_wake_sleep: None, // built-in callback
            // override default gravity, as we know bevy's coordinate system,
            // and default zero gravity doesn't work with vehicles and such
            gravity: Vec3::new(0.0, -9.81, 0.0),
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
            sanity_bounds: d.sanity_bounds,
            simulation_filter_shader: d.simulation_filter_shader,
            thread_count: d.thread_count,
            gpu_max_num_partitions: d.gpu_max_num_partitions,
            //gpu_compute_version: d.gpu_compute_version,
        }
    }
}
