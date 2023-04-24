#![warn(clippy::manual_assert)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use std::time::Duration;

use bevy::ecs::schedule::{SystemSetConfigs, ScheduleLabel};
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
use crate::prelude as bpx;

pub mod assets;
pub mod callbacks;
pub mod components;
pub mod plugins;
pub mod prelude;
pub mod raycast;
pub mod render;
pub mod resources;
pub mod systems;
pub mod utils;

// reexport physx to avoid version conflicts
pub use physx;
pub use physx_sys;

use resources::DefaultMaterial;

type PxMaterial = physx::material::PxMaterial<()>;
type PxShape = physx::shape::PxShape<Entity, PxMaterial>;
type PxArticulationLink = physx::articulation_link::PxArticulationLink<Entity, PxShape>;
type PxRigidStatic = physx::rigid_static::PxRigidStatic<Entity, PxShape>;
type PxRigidDynamic = physx::rigid_dynamic::PxRigidDynamic<Entity, PxShape>;
type PxArticulationReducedCoordinate =
    physx::articulation_reduced_coordinate::PxArticulationReducedCoordinate<Entity, PxArticulationLink>;

type PxScene = physx::scene::PxScene<
    (),
    PxArticulationLink,
    PxRigidStatic,
    PxRigidDynamic,
    PxArticulationReducedCoordinate,
    callbacks::OnCollision,
    callbacks::OnTrigger,
    callbacks::OnConstraintBreak,
    callbacks::OnWakeSleep,
    callbacks::OnAdvance,
>;

#[derive(Clone)]
pub struct FoundationDescriptor {
    pub extensions: bool,
    pub tolerances: PxTolerancesScale,
    pub visual_debugger: bool,
    pub visual_debugger_port: i32,
    pub visual_debugger_remote: Option<String>,
}

impl Default for FoundationDescriptor {
    fn default() -> Self {
        Self {
            extensions: true,
            tolerances: PxTolerancesScale { length: 1., speed: 10. },
            visual_debugger: true,
            visual_debugger_port: 5425,
            visual_debugger_remote: None,
        }
    }
}

pub struct SceneDescriptor {
    pub on_collision: Option<callbacks::OnCollision>,
    pub on_trigger: Option<callbacks::OnTrigger>,
    pub on_constraint_break: Option<callbacks::OnConstraintBreak>,
    pub on_wake_sleep: Option<callbacks::OnWakeSleep>,
    pub on_advance: Option<callbacks::OnAdvance>,
    pub gravity: Vec3,
    pub kine_kine_filtering_mode: PairFilteringMode,
    pub static_kine_filtering_mode: PairFilteringMode,
    pub broad_phase_type: BroadPhaseType,
    pub limits: SceneLimits,
    pub friction_type: FrictionType,
    pub solver_type: SolverType,
    pub bounce_threshold_velocity: f32,
    pub friction_offset_threshold: f32,
    pub ccd_max_separation: f32,
    pub flags: SceneFlags,
    pub static_structure: PruningStructureType,
    pub dynamic_structure: PruningStructureType,
    pub dynamic_tree_rebuild_rate_hint: u32,
    pub scene_query_update_mode: SceneQueryUpdateMode,
    pub solver_batch_size: u32,
    pub solver_articulation_batch_size: u32,
    pub nb_contact_data_blocks: u32,
    pub max_nb_contact_data_blocks: u32,
    pub max_bias_coefficient: f32,
    pub contact_report_stream_buffer_size: u32,
    pub ccd_max_passes: u32,
    pub ccd_threshold: f32,
    pub wake_counter_reset_value: f32,
    pub sanity_bounds: PxBounds3,

    pub simulation_filter_shader: FilterShaderDescriptor,

    pub thread_count: u32,
    pub gpu_max_num_partitions: u32,
    pub gpu_compute_version: u32,
}

impl Default for SceneDescriptor {
    fn default() -> Self {
        let d = physx::traits::descriptor::SceneDescriptor::<
            (), PxArticulationLink, PxRigidStatic, PxRigidDynamic,
            PxArticulationReducedCoordinate,
            callbacks::OnCollision, callbacks::OnTrigger, callbacks::OnConstraintBreak,
            callbacks::OnWakeSleep, callbacks::OnAdvance
        >::new(());

        SceneDescriptor {
            on_collision: None,
            on_trigger: None,
            on_constraint_break: None,
            on_wake_sleep: None,
            on_advance: None,
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
            gpu_compute_version: d.gpu_compute_version,
        }
    }
}

// All systems related to physics simulation are added to a separate schedule.
// This allows user the flexibility of running it whenever needed
// (for example, on fixed schedule).
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PhysicsSchedule;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
#[system_set(base)]
pub enum PhysicsSet {
    // everything related to simulation itself
    // (scene simulation + vehicle simulation)
    Simulate,
    // - propagate transforms
    // - create new actors, and everything else that uses
    //   commands to insert new components
    // - apply system buffers
    // maybe it should be split to more stages, but I don't expect
    // this to have too many systems
    Create,
    // two-way sync of physx state with existing bevy components
    Sync,
}

impl PhysicsSet {
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Simulate, Self::Create, Self::Sync].into_iter()
    }

    pub fn sets() -> SystemSetConfigs {
        (Self::Simulate, Self::Create, Self::Sync).chain()
    }
}

pub struct PhysXPlugin {
    pub foundation: FoundationDescriptor,
    pub scene: SceneDescriptor,
    pub timestep: TimestepMode,
}

impl Default for PhysXPlugin {
    fn default() -> Self {
        Self {
            foundation: default(),
            scene: default(),
            timestep: default(),
        }
    }
}

impl Plugin for PhysXPlugin {
    fn build(&self, app: &mut App) {
        let mut physics = bpx::Physics::new(&self.foundation);
        let scene = bpx::Scene::new(&mut physics, &self.scene);

        app.init_schedule(PhysicsSchedule);

        if !app.is_plugin_added::<AssetPlugin>() {
            app.add_plugin(AssetPlugin::default());
        }

        app.add_asset::<bpx::Geometry>();
        app.add_asset::<bpx::Material>();

        app.insert_resource(scene);

        let default_material = DefaultMaterial(
            app.world.resource_mut::<Assets<bpx::Material>>()
                .add(physics.create_material(0.5, 0.5, 0.6, ()).unwrap().into())
        );
        app.insert_resource(default_material);

        app.register_type::<PhysicsTime>();
        app.insert_resource(PhysicsTime::new(self.timestep));
        app.init_resource::<BevyTimeDelta>();

        // physics must be last (so it will be dropped last)
        app.insert_resource(physics);

        // it's important here to configure set order
        app.edit_schedule(PhysicsSchedule, |schedule| {
            schedule.configure_sets(PhysicsSet::sets());
        });

        app.add_plugin(crate::plugins::ArticulationPlugin);
        app.add_plugin(crate::plugins::DampingPlugin);
        app.add_plugin(crate::plugins::ExternalForcePlugin);
        app.add_plugin(crate::plugins::MassPropertiesPlugin);
        app.add_plugin(crate::plugins::MaxVelocityPlugin);
        app.add_plugin(crate::plugins::VelocityPlugin);

        // add all systems to the set
        app.add_systems((
            systems::scene_simulate,
        ).in_base_set(PhysicsSet::Simulate).in_schedule(PhysicsSchedule));

        app.add_systems((
            bevy::transform::systems::propagate_transforms.before(systems::create_rigid_actors),
            bevy::transform::systems::sync_simple_transforms.before(systems::create_rigid_actors),
            systems::create_rigid_actors.before(apply_system_buffers),
            apply_system_buffers,
        ).in_base_set(PhysicsSet::Create).in_schedule(PhysicsSchedule));

        app.add_systems((
            systems::sync_transform_static,
            systems::sync_transform_dynamic,
            systems::sync_transform_articulation_links,
            systems::sync_transform_nested_shapes,
        ).in_base_set(PhysicsSet::Sync).in_schedule(PhysicsSchedule));

        // add scheduler
        app.add_system(
            run_physics_schedule
                .in_base_set(CoreSet::FixedUpdate)
                .before(bevy::time::fixed_timestep::run_fixed_update_schedule)
        );
    }
}

#[derive(Debug, Reflect, PartialEq, Clone, Copy)]
pub enum TimestepMode {
    /// Physics simulation will be advanced by dt at each Bevy tick.
    /// Real time does not make any difference for this timestep mode.
    /// This is preferred method if you have fixed FPS with the tools like bevy_framepace,
    /// or running it in a FixedUpdate schedule.
    Fixed {
        dt: f32,
        substeps: usize,
    },
    /// Physics simulation will be advanced at each Bevy tick by the real time elapsed,
    /// but no more than max_dt.
    /// Simulation time will always match real time, unless system can't handle
    /// frames in time.
    Variable {
        max_dt: f32,
        time_scale: f32,
        substeps: usize,
    },
    /// Physics simulation will be advanced by dt, but advance
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

#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
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
