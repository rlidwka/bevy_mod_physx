#![warn(clippy::manual_assert)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
use crate::prelude::*;
use crate::prelude as bpx;
mod type_bridge;

mod systems;
pub mod assets;
pub mod components;
pub mod callbacks;
pub mod prelude;
pub mod resources;

// reexport physx to avoid version conflicts
pub use physx;
pub use physx_sys;

use resources::{DefaultMaterial, VehicleSceneQueryData, VehicleFrictionPairs};

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

pub struct PhysXPlugin {
    pub vehicles: bool,
    pub cooking: bool,
    pub debugger: bool,
    pub gravity: Vec3,
    pub timestep: TimestepMode,
}

#[derive(Debug, StageLabel)]
pub struct PhysXStage;

impl Plugin for PhysXPlugin {
    fn build(&self, app: &mut App) {
        let mut physics = Physics::new(self.debugger, self.vehicles);
        let scene = bpx::Scene::new(&mut physics, self.gravity);

        app.add_asset::<bpx::Geometry>();
        app.add_asset::<bpx::Material>();

        app.register_type::<Velocity>();

        if self.cooking {
            app.insert_resource(Cooking::new(&mut physics));
        }

        if self.vehicles {
            app.insert_resource(VehicleSceneQueryData::default());
            app.insert_resource(VehicleFrictionPairs::default());
        }

        app.insert_resource(scene);
        app.insert_resource(DefaultMaterial::default());

        app.register_type::<SimTime>();
        app.insert_resource(SimTime::new(self.timestep));

        // physics must be last (so it will be dropped last)
        app.insert_resource(physics);

        let mut stage = SystemStage::parallel();
        stage.add_system(time_sync.before(systems::scene_simulate));
        stage.add_system(systems::scene_simulate);
        stage.add_system(systems::create_dynamic_actors.after(systems::scene_simulate));
        stage.add_system(systems::writeback_actors.after(systems::scene_simulate));

        // this needs to happen after globaltransform is applied,
        // and inserting it after(CoreStage::Update) messes with conditional staging;
        // after(PostUpdate) works, but need to investigate which is the better timing
        app.add_stage_after(CoreStage::PostUpdate, PhysXStage, stage);
    }
}

impl Default for PhysXPlugin {
    fn default() -> Self {
        Self {
            vehicles: true,
            cooking: true,
            debugger: true,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            timestep: default(),
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
