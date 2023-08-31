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

pub mod core;
pub mod plugins;
pub mod prelude;
pub mod types;
pub mod utils;

// reexport physx to avoid version conflicts
pub use physx;
pub use physx_sys;

use crate::prelude as bpx;
use crate::core::systems;
use crate::core::material::{DefaultMaterial, DefaultMaterialHandle};

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

/// This plugin group will add all available physics plugins.
///
/// Plugin architecture is as follows:
///
/// There's one mandatory plugin called [PhysicsCore], which manages
/// actor creation, simulation and transform synchronization.
///
/// Then there are a lot of optional plugins that each add components
/// (e.g. Velocity), which synchronize their contents with physx
/// engine. This synchronization can be one-way or two-way depending
/// on a specific plugin.
pub struct PhysicsPlugins;

impl PluginGroup for PhysicsPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(PhysicsCore::default())
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

/// Primary physics plugin that manages actor creation, simulation and transforms.
pub struct PhysicsCore {
    pub foundation: bpx::FoundationDescriptor,
    pub scene: bpx::SceneDescriptor,
    pub timestep: TimestepMode,
    pub default_material: DefaultMaterial,
    pub sync_first: bool,
}

impl PhysicsCore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timestep(mut self, timestep: TimestepMode) -> Self {
        self.timestep = timestep;
        self
    }

    pub fn with_gravity(mut self, gravity: Vec3) -> Self {
        self.scene.gravity = gravity;
        self
    }

    pub fn with_pvd(mut self) -> Self {
        self.foundation.visual_debugger = true;
        self
    }
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
