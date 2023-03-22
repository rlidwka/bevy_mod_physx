#![warn(clippy::manual_assert)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::ecs::schedule::SystemConfigs;
use bevy::prelude::*;
use enumflags2::BitFlags;
use physx::prelude::*;
use physx::vehicles::VehicleUpdateMode;
use physx::scene::{FilterShaderDescriptor, SceneQueryUpdateMode, PruningStructureType, PairFilteringMode, BroadPhaseType, SceneLimits, SolverType, FrictionType, SceneFlag};
use physx_sys::{PxTolerancesScale, PxTolerancesScale_new};
use crate::prelude::*;
use crate::prelude as bpx;
mod type_bridge;

mod systems;
pub mod assets;
pub mod callbacks;
pub mod components;
pub mod prelude;
pub mod resources;
pub mod render;

// reexport physx to avoid version conflicts
pub use physx;
pub use physx_sys;

use resources::{DefaultMaterial, VehicleSimulation, VehicleSimulationMethod};

type PxMaterial = physx::material::PxMaterial<()>;
type PxShape = physx::shape::PxShape<Entity, PxMaterial>;
type PxArticulationLink = physx::articulation_link::PxArticulationLink<(), PxShape>;
type PxRigidStatic = physx::rigid_static::PxRigidStatic<Entity, PxShape>;
type PxRigidDynamic = physx::rigid_dynamic::PxRigidDynamic<Entity, PxShape>;
type PxArticulation = physx::articulation::PxArticulation<(), PxArticulationLink>;
type PxArticulationReducedCoordinate =
    physx::articulation_reduced_coordinate::PxArticulationReducedCoordinate<(), PxArticulationLink>;

type PxScene = physx::scene::PxScene<
    (),
    PxArticulationLink,
    PxRigidStatic,
    PxRigidDynamic,
    PxArticulation,
    PxArticulationReducedCoordinate,
    callbacks::OnCollision,
    callbacks::OnTrigger,
    callbacks::OnConstraintBreak,
    callbacks::OnWakeSleep,
    callbacks::OnAdvance,
>;

#[derive(Debug, Clone, Copy)]
pub struct TolerancesScale {
    pub length: f32,
    pub speed: f32,
}

impl From<PxTolerancesScale> for TolerancesScale {
    fn from(value: PxTolerancesScale) -> Self {
        Self {
            length: value.length,
            speed: value.speed,
        }
    }
}

impl From<TolerancesScale> for PxTolerancesScale {
    fn from(value: TolerancesScale) -> Self {
        let mut result = unsafe { PxTolerancesScale_new() };
        result.length = value.length;
        result.speed = value.speed;
        result
    }
}

impl Default for TolerancesScale {
    fn default() -> Self {
        unsafe { PxTolerancesScale_new() }.into()
    }
}

#[derive(Debug, Clone)]
pub struct FoundationDescriptor {
    pub cooking: bool,
    pub extensions: bool,
    pub tolerances: TolerancesScale,

    pub visual_debugger: bool,
    pub visual_debugger_port: i32,
    pub visual_debugger_remote: Option<String>,

    pub vehicles: bool,
    pub vehicles_basis_vectors: [ Vec3; 2 ],
    pub vehicles_update_mode: VehicleUpdateMode,
    pub vehicles_max_hit_actor_acceleration: f32,
    pub vehicles_sweep_hit_rejection_angles: [ f32; 2 ],
    pub vehicles_simulation_method: VehicleSimulationMethod,
}

impl Default for FoundationDescriptor {
    fn default() -> Self {
        Self {
            cooking: true,
            extensions: true,
            tolerances: default(),
            visual_debugger: true,
            visual_debugger_port: 5425,
            visual_debugger_remote: None,
            vehicles: true,
            vehicles_basis_vectors: [ Vec3::new(0., 1., 0.), Vec3::new(0., 0., 1.) ],
            vehicles_update_mode: VehicleUpdateMode::VelocityChange,
            vehicles_max_hit_actor_acceleration: std::f32::MAX,
            vehicles_sweep_hit_rejection_angles: [ 0.707f32, 0.707f32 ],
            vehicles_simulation_method: VehicleSimulationMethod::Sweep {
                nb_hits_per_query: 1,
                sweep_width_scale: 1.,
                sweep_radius_scale: 1.01,
            },
        }
    }
}

pub struct SceneDescriptor {
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
    pub solver_offset_slop: f32,
    pub flags: BitFlags<SceneFlag>,
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
            PxArticulation, PxArticulationReducedCoordinate,
            callbacks::OnCollision, callbacks::OnTrigger, callbacks::OnConstraintBreak,
            callbacks::OnWakeSleep, callbacks::OnAdvance
        >::new(());

        SceneDescriptor {
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
            solver_offset_slop: d.solver_offset_slop,
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

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
#[system_set(base)]
pub enum PhysicsSet {
    First,
    Simulation,
    Last,
}

impl PhysicsSet {
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::First, Self::Simulation, Self::Last].into_iter()
    }
}

pub struct PhysXPlugin {
    pub foundation: FoundationDescriptor,
    pub scene: SceneDescriptor,
    pub timestep: TimestepMode,
    pub default_system_setup: bool,
}

impl Default for PhysXPlugin {
    fn default() -> Self {
        Self {
            foundation: default(),
            scene: default(),
            timestep: default(),
            default_system_setup: true,
        }
    }
}

impl PhysXPlugin {
    pub fn get_systems(set: PhysicsSet) -> SystemConfigs {
        match set {
            PhysicsSet::First => (
                time_sync,
                systems::apply_user_changes,
            ).into_configs(),

            PhysicsSet::Simulation => (
                systems::scene_simulate,
            ).into_configs(),

            PhysicsSet::Last => (
                systems::create_dynamic_actors,
                systems::writeback_actors,
            ).into_configs(),
        }
    }
}

impl Plugin for PhysXPlugin {
    fn build(&self, app: &mut App) {
        let mut physics = bpx::Physics::new(&self.foundation);
        let scene = bpx::Scene::new(&mut physics, &self.scene);

        app.add_asset::<bpx::Geometry>();
        app.add_asset::<bpx::Material>();

        app.register_type::<Velocity>();

        if self.foundation.cooking {
            app.insert_resource(Cooking::new(&mut physics));
        }

        if self.foundation.vehicles {
            app.insert_resource(VehicleSimulation::new(self.foundation.vehicles_simulation_method));
        }

        app.insert_resource(scene);
        app.insert_resource(DefaultMaterial::default());

        app.register_type::<SimTime>();
        app.insert_resource(SimTime::new(self.timestep));

        // physics must be last (so it will be dropped last)
        app.insert_resource(physics);

        if self.default_system_setup {
            // user may want to add more restrictions on how sets are run,
            // but it must run before PostUpdate for GlobalTransform to propagate
            app.configure_sets((
                PhysicsSet::First,
                PhysicsSet::Simulation,
                PhysicsSet::Last,
            ).chain().before(CoreSet::PostUpdate));

            for set in PhysicsSet::iter() {
                app.add_systems(Self::get_systems(set).in_base_set(set));
            }
        }
    }
}

#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct SimTime {
    pub timestep: TimestepMode,
    delta: f32,
    current_tick: (f32, usize),
}

impl SimTime {
    pub fn new(timestep: TimestepMode) -> Self {
        Self { timestep, delta: 0., current_tick: (0., 1) }
    }

    pub fn update(&mut self, time: &Time) {
        match self.timestep {
            TimestepMode::Fixed { dt, substeps } => {
                self.delta = 0.;
                self.current_tick = (dt / substeps as f32, substeps);
            },
            TimestepMode::Variable { max_dt, time_scale, substeps } => {
                self.delta += time.delta_seconds() * time_scale;

                if self.delta > max_dt && max_dt > 0. {
                    self.current_tick = (max_dt / substeps as f32, substeps);
                    self.delta = 0.;
                    //self.delta -= max_dt;
                } else {
                    self.current_tick = (self.delta / substeps as f32, substeps);
                    self.delta = 0.;
                }
            },
            TimestepMode::Interpolated { dt, time_scale, substeps } => {
                self.delta += time.delta_seconds() * time_scale;

                if self.delta > dt && dt > 0. {
                    self.current_tick = (dt / substeps as f32, substeps);
                    self.delta -= dt;
                    // avoid endless accumulating of lag
                    if self.delta > dt { self.delta = dt; }
                } else {
                    self.current_tick = (0., 0);
                }
            }
        }
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }

    pub fn ticks(&self) -> impl Iterator<Item = f32> + '_ {
        SimTimeIterator::new(self.current_tick.0, self.current_tick.1)
    }
}

struct SimTimeIterator {
    duration: f32,
    remaining: usize,
}

impl SimTimeIterator {
    fn new(duration: f32, substeps: usize) -> Self {
        Self { duration, remaining: substeps }
    }
}

impl Iterator for SimTimeIterator {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining > 0 {
            self.remaining -= 1;
            Some(self.duration)
        } else {
            None
        }
    }
}

impl std::iter::FusedIterator for SimTimeIterator {}

#[derive(Debug, Reflect, PartialEq, Clone, Copy)]
pub enum TimestepMode {
    /// Physics simulation will be advanced by dt at each Bevy tick.
    /// Real time does not make any difference for this timestep mode.
    /// This is preferred method if you have fixed FPS with the tools like bevy_framepace.
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
}

impl Default for TimestepMode {
    fn default() -> Self {
        Self::Interpolated { dt: 1. / 60., time_scale: 1., substeps: 1 }
    }
}

fn time_sync(time: Res<Time>, mut simtime: ResMut<SimTime>) {
    simtime.update(&time);
}
