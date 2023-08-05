//! <p align="left">
//!   <a href="https://github.com/rlidwka/bevy_mod_physx/blob/master/examples/articulation.rs" title="see collision example">
//!     <img src="https://user-images.githubusercontent.com/999113/253824185-ade6f3c1-0ce7-4e95-833a-daa619acbcb6.png" width="48%">
//!   </a>
//!   &nbsp;
//!   <a href="https://github.com/rlidwka/bevy_mod_physx/blob/master/examples/cube_stacks.rs" title="see articulation example">
//!     <img src="https://user-images.githubusercontent.com/999113/253824183-11d21bb3-700d-4a0b-aab4-60b48af49c23.png" width="48%">
//!   </a>
//! </p>
//!
//! [PhysX](https://github.com/NVIDIA-Omniverse/PhysX) is an open-source Physics SDK written in C++ and developed by Nvidia. \
//! This crate is a bridge between Bevy ECS and Rust [bindings](https://github.com/EmbarkStudios/physx-rs) made by Embark Studios.

// useful asserts that's off by default
#![warn(clippy::manual_assert)]
#![warn(clippy::semicolon_if_nothing_returned)]
//
// these are often intentionally not collapsed for readability
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_match)]
//
// these are intentional in bevy systems: nobody is directly calling those,
// so extra arguments don't decrease readability
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use std::time::Duration;

use bevy::app::PluginGroupBuilder;
use bevy::ecs::schedule::{ScheduleLabel, SystemSetConfigs};
use bevy::prelude::*;
use physx::prelude::*;
use physx::scene::{
    BroadPhaseType,
    FilterShaderDescriptor,
    FrictionType,
    PairFilteringMode,
    PruningStructureType,
    SceneFlags,
    SceneLimits,
    SceneQueryUpdateMode,
    SolverType,
};
use physx_sys::PxTolerancesScale;

pub mod assets;
pub mod components;
pub mod events;
pub mod plugins;
pub mod prelude;
pub mod raycast;
pub mod resources;
pub mod systems;
pub mod types;
pub mod utils;

// reexport physx to avoid version conflicts
pub use physx;
pub use physx_sys;
pub mod physx_extras;

use crate::prelude as bpx;
use crate::resources::{DefaultMaterial, DefaultMaterialHandle};
use crate::types::*;

#[derive(Clone)]
/// Descriptor class for creating a physics foundation.
pub struct FoundationDescriptor {
    /// Initialize the PhysXExtensions library.
    ///
    /// Default: true
    pub extensions: bool,
    /// Values used to determine default tolerances for objects at creation time.
    ///
    /// Default: length=1, speed=10
    pub tolerances: PxTolerancesScale,
    /// Enable visual debugger (PVD).
    ///
    /// Default: true
    pub visual_debugger: bool,
    /// IP port used for PVD, should same as the port setting
    /// in PVD application.
    ///
    /// Default: 5425
    pub visual_debugger_port: i32,
    /// Host address of the PVD application.
    ///
    /// Default: localhost
    pub visual_debugger_host: Option<String>,
}

impl Default for FoundationDescriptor {
    fn default() -> Self {
        Self {
            extensions: true,
            tolerances: PxTolerancesScale { length: 1., speed: 10. },
            visual_debugger: true,
            visual_debugger_port: 5425,
            visual_debugger_host: None,
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

/// Dedicated schedule for all physics-related systems.
///
/// By default, it is executed in `PreUpdate` by [run_physics_schedule],
/// see its documentation for details.
///
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PhysicsSchedule;

/// SystemSet inside [PhysicsSchedule] where all physics systems live.
///
/// It is advised to put your own physics-related functions in PhysicsSchedule
/// before or after this set.
///
/// ### Sync set execution order caveat
///
/// Note: [PhysicsSet::Sync] may be configured either at the end or at the
/// beginning depending on [PhysicsCore] `sync_first` setting.
///
/// I.e. end user can choose two orders of execution:
/// - Simulate, SimulateFlush, Create, CreateFlush, Sync
/// - Sync, Simulate, SimulateFlush, Create, CreateFlush (default)
///
/// This is made so because PhysX debug render lags one frame behind:
///  - <https://github.com/NVIDIA-Omniverse/PhysX/issues/169>
///
/// If you do not care about debug render, you can apply transforms
/// a frame earlier by changing order of execution.
///
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub enum PhysicsSet {
    /// First label in [PhysicsSet]. Use it to order your own systems
    /// within [PhysicsSchedule].
    First,
    /// Update GlobalTransform component of all entities.
    PropagateTransforms,
    /// Two-way sync of states between PhysX engine and existing Bevy components.
    Sync,
    /// Systems that request simulation from PhysX.
    ///
    /// This includes scene simulation and (in the future) vehicle simulation.
    Simulate,
    /// A copy of [apply_deferred] that is required to ensure that deferred
    /// system parameters are applied *before* Create set is executed.
    SimulateFlush,
    /// Create new actors, and everything else that creates new components
    /// or executes other Commands.
    Create,
    /// A copy of [apply_deferred] that is required to ensure that deferred
    /// system parameters *from* Create set are applied.
    CreateFlush,
    /// Last label in [PhysicsSet]. Use it to order your own systems
    /// within [PhysicsSchedule].
    Last,
}

impl PhysicsSet {
    pub fn sets(sync_first: bool) -> SystemSetConfigs {
        if sync_first {
            // sync is placed first to match debug visualization,
            // some users may want to place sync last to get transform data faster
            (
                Self::First,
                Self::PropagateTransforms,
                Self::Sync,
                Self::Simulate,
                Self::SimulateFlush,
                Self::Create,
                Self::CreateFlush,
                Self::Last,
            ).chain()
        } else {
            (
                Self::First,
                Self::PropagateTransforms,
                Self::Simulate,
                Self::SimulateFlush,
                Self::Create,
                Self::CreateFlush,
                Self::Sync,
                Self::Last,
            ).chain()
        }
    }
}

pub struct PhysicsPlugins;

impl PluginGroup for PhysicsPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(PhysicsCore::default())
            .add(crate::plugins::articulation::ArticulationPlugin)
            .add(crate::plugins::damping::DampingPlugin)
            .add(crate::plugins::debug_render::DebugRenderPlugin)
            .add(crate::plugins::external_force::ExternalForcePlugin)
            .add(crate::plugins::kinematic::KinematicPlugin)
            .add(crate::plugins::mass_properties::MassPropertiesPlugin)
            .add(crate::plugins::name::NamePlugin)
            .add(crate::plugins::shape_filter_data::ShapeFilterDataPlugin)
            .add(crate::plugins::shape_offsets::ShapeOffsetsPlugin)
            .add(crate::plugins::sleep::SleepPlugin)
            .add(crate::plugins::velocity::VelocityPlugin)
    }
}

pub struct PhysicsCore {
    pub foundation: FoundationDescriptor,
    pub scene: SceneDescriptor,
    pub timestep: TimestepMode,
    pub default_material: DefaultMaterial,
    pub sync_first: bool,
}

impl Default for PhysicsCore {
    fn default() -> Self {
        Self {
            foundation: default(),
            scene: default(),
            timestep: default(),
            default_material: DefaultMaterial {
                static_friction: 0.5,
                dynamic_friction: 0.5,
                restitution: 0.6,
            },
            sync_first: true,
        }
    }
}

impl Plugin for PhysicsCore {
    fn build(&self, app: &mut App) {
        app.init_schedule(PhysicsSchedule);

        if !app.is_plugin_added::<AssetPlugin>() {
            // this is required for Geometry and Material,
            // which are stored internally as custom assets
            app.add_plugins(AssetPlugin::default());
        }

        app.add_asset::<bpx::Geometry>();
        app.add_asset::<bpx::Material>();

        app.register_type::<PhysicsTime>();
        app.insert_resource(PhysicsTime::new(self.timestep));
        app.init_resource::<BevyTimeDelta>();

        // it's important here to configure set order
        app.edit_schedule(PhysicsSchedule, |schedule| {
            schedule.configure_sets(PhysicsSet::sets(self.sync_first));
        });

        // add all systems to the set
        app.add_systems(PhysicsSchedule, (
            bevy::transform::systems::propagate_transforms,
            bevy::transform::systems::sync_simple_transforms,
        ).in_set(PhysicsSet::PropagateTransforms));

        app.add_systems(PhysicsSchedule, (
            systems::sync_transform_static,
            systems::sync_transform_dynamic,
            systems::sync_transform_articulation_links,
            systems::sync_transform_nested_shapes,
        ).in_set(PhysicsSet::Sync));

        app.add_systems(PhysicsSchedule, (
            systems::scene_simulate,
        ).in_set(PhysicsSet::Simulate));

        app.add_systems(PhysicsSchedule, (
            apply_deferred,
        ).in_set(PhysicsSet::SimulateFlush));

        app.add_systems(PhysicsSchedule, (
            systems::create_rigid_actors,
        ).in_set(PhysicsSet::Create));

        app.add_systems(PhysicsSchedule, (
            apply_deferred,
        ).in_set(PhysicsSet::CreateFlush));

        // add scheduler
        app.add_systems(PreUpdate, run_physics_schedule);
    }

    fn finish(&self, app: &mut App) {
        let mut physics = bpx::Physics::new(&self.foundation);

        let wake_sleep_callback = app.world.remove_resource::<crate::plugins::sleep::WakeSleepCallback>();
        let scene = bpx::Scene::new(&mut physics, &self.scene, wake_sleep_callback.map(|x| x.0));

        app.insert_resource(scene);

        let default_material = DefaultMaterialHandle(
            app.world.resource_mut::<Assets<bpx::Material>>()
                .add(physics.create_material(0.5, 0.5, 0.6, ()).unwrap().into())
        );
        app.insert_resource(default_material);

        // physics must be last (so it will be dropped last)
        app.insert_resource(physics);
    }
}

/// Defines how often physics shall be simulated with respect to bevy time.
#[derive(Debug, PartialEq, Clone, Copy, Reflect)]
#[reflect(Default)]
pub enum TimestepMode {
    /// Physics simulation will be advanced by `dt` at each Bevy tick.
    /// Real time does not make any difference for this timestep mode.
    /// This is preferred method if you have fixed FPS with the tools like bevy_framepace,
    /// or running it in a [FixedUpdate] schedule.
    Fixed {
        dt: f32,
        substeps: usize,
    },
    /// Physics simulation will be advanced at each Bevy tick by the real time elapsed,
    /// but no more than `max_dt`.
    /// Simulation time will always match real time, unless system can't handle
    /// frames in time.
    Variable {
        max_dt: f32,
        time_scale: f32,
        substeps: usize,
    },
    /// Physics simulation will be advanced by `dt`, but advance
    /// no more than real time multiplied by time_scale (so some ticks might get skipped).
    /// Simulation time will lag up to `dt` with respect to real time.
    /// This is preferred method if you don't have limited FPS.
    Interpolated {
        dt: f32,
        time_scale: f32,
        substeps: usize,
    },
    /// Physics simulation advancement is controlled by user manually running
    /// `world.run_schedule(PhysicsSchedule)`.
    Custom,
}

impl Default for TimestepMode {
    fn default() -> Self {
        Self::Interpolated { dt: 1. / 60., time_scale: 1., substeps: 1 }
    }
}

/// A clock that tracks how much time was simulated by the physics engine.
///
/// This clock is similar to [Time] and measures execution of [PhysicsSchedule],
/// delta corresponds to time advanced from previous PhysicsSchedule execution.
#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource, Default)]
pub struct PhysicsTime {
    timestep: TimestepMode,
    delta: Duration,
    delta_seconds: f32,
    delta_seconds_f64: f64,
    elapsed: Duration,
    elapsed_seconds: f32,
    elapsed_seconds_f64: f64,
}

impl PhysicsTime {
    pub fn new(timestep: TimestepMode) -> Self {
        Self { timestep, ..default() }
    }

    pub fn update(&mut self, delta: Duration) {
        self.delta = delta;
        self.delta_seconds = self.delta.as_secs_f32();
        self.delta_seconds_f64 = self.delta.as_secs_f64();

        self.elapsed += delta;
        self.elapsed_seconds = self.elapsed.as_secs_f32();
        self.elapsed_seconds_f64 = self.elapsed.as_secs_f64();
    }

    #[inline]
    pub fn timestep(&self) -> TimestepMode {
        self.timestep
    }

    #[inline]
    pub fn set_timestep(&mut self, timestep: TimestepMode) {
        self.timestep = timestep;
    }

    #[inline]
    pub fn delta(&self) -> Duration {
        self.delta
    }

    #[inline]
    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }

    #[inline]
    pub fn delta_seconds_f64(&self) -> f64 {
        self.delta_seconds_f64
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    #[inline]
    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed_seconds
    }

    #[inline]
    pub fn elapsed_seconds_f64(&self) -> f64 {
        self.elapsed_seconds_f64
    }
}

#[derive(Resource, Default)]
struct BevyTimeDelta(f32);

/// Runs [PhysicsSchedule] in `PreUpdate`.
///
/// All systems related to physics simulation are put in a separate schedule in
/// order to have more control over timing. See [TimestepMode] for details how
/// to configure this, and you can use [TimestepMode::Custom] to replace this
/// with a custom runner.
///
pub fn run_physics_schedule(world: &mut World) {
    fn simulate(world: &mut World, delta: f32, substeps: usize) {
        let dt = Duration::from_secs_f32(delta / substeps as f32);
        for _ in 0..substeps {
            let mut pxtime = world.resource_mut::<PhysicsTime>();
            pxtime.update(dt);
            world.run_schedule(PhysicsSchedule);
        }
    }

    match world.resource::<PhysicsTime>().timestep() {
        TimestepMode::Fixed { dt, substeps } => {
            let mut pxdelta = world.resource_mut::<BevyTimeDelta>();
            pxdelta.0 = 0.;
            simulate(world, dt, substeps);
        }

        TimestepMode::Variable { max_dt, time_scale, substeps } => {
            let bevy_delta = world.resource::<Time>().delta_seconds();
            let mut pxdelta = world.resource_mut::<BevyTimeDelta>();
            pxdelta.0 += bevy_delta * time_scale;

            let dt = if pxdelta.0 > max_dt && max_dt > 0. {
                max_dt
            } else {
                pxdelta.0
            };
            pxdelta.0 = 0.;

            simulate(world, dt, substeps);
        }

        TimestepMode::Interpolated { dt, time_scale, substeps } => {
            let bevy_delta = world.resource::<Time>().delta_seconds();
            let mut pxdelta = world.resource_mut::<BevyTimeDelta>();
            pxdelta.0 += bevy_delta * time_scale;

            if pxdelta.0 > dt && dt > 0. {
                pxdelta.0 -= dt;
                // avoid endless accumulating of lag
                if pxdelta.0 > dt { pxdelta.0 = dt; }
                simulate(world, dt, substeps);
            }
        }

        TimestepMode::Custom => {
            // up to the user to handle this
        }
    }
}
