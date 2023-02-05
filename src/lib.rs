#![warn(clippy::manual_assert)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use std::time::Duration;

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

use components::BPxVelocity;
use resources::{DefaultMaterial, BPxVehicleRaycastBuffer, BPxVehicleFrictionPairs};

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
    pub timestep: f32,
}

impl Plugin for PhysXPlugin {
    fn build(&self, app: &mut App) {
        let mut physics = Physics::new(self.debugger, self.vehicles);
        let scene = bpx::Scene::new(&mut physics, self.gravity);

        app.add_asset::<bpx::Geometry>();
        app.add_asset::<bpx::Material>();

        app.add_event::<Tick>();

        app.register_type::<BPxVelocity>();

        if self.cooking {
            app.insert_resource(Cooking::new(&mut physics));
        }

        if self.vehicles {
            app.insert_resource(BPxVehicleRaycastBuffer::default());
            app.insert_resource(BPxVehicleFrictionPairs::default());
        }

        app.insert_resource(scene);
        app.insert_resource(BPxTimeSync::new(self.timestep));
        app.insert_resource(DefaultMaterial::default());

        // physics must be last (so it will be dropped last)
        app.insert_resource(physics);

        #[derive(Debug, StageLabel)]
        struct PhysXStage;

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
            timestep: 1. / 60.,
        }
    }
}

#[derive(Resource, Default)]
struct BPxTimeSync {
    timestep: f32,
    speed_factor: f32,
    bevy_physx_delta: f32,
}

impl BPxTimeSync {
    pub fn new(timestep: f32) -> Self {
        Self { timestep, speed_factor: 1., ..default() }
    }

    /*pub fn get_delta(&self) -> f32 {
        self.bevy_physx_delta
    }*/

    pub fn advance_bevy_time(&mut self, time: &Time) {
        self.bevy_physx_delta += time.delta_seconds() * self.speed_factor;
    }

    pub fn check_advance_physx_time(&mut self) -> Option<f32> {
        if self.bevy_physx_delta >= self.timestep {
            self.bevy_physx_delta -= self.timestep;
            Some(self.timestep)
        } else {
            None
        }
    }
}

pub struct Tick(pub Duration);

fn time_sync(
    time: Res<Time>,
    mut timesync: ResMut<BPxTimeSync>,
    mut physx_ticks: EventWriter<Tick>,
) {
    timesync.advance_bevy_time(&time);

    if let Some(delta) = timesync.check_advance_physx_time() {
        physx_ticks.send(Tick(Duration::from_secs_f32(delta)));
    }
}
