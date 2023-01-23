#![warn(clippy::manual_assert)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
mod type_bridge;

mod systems;
pub mod assets;
pub mod components;
pub mod callbacks;
pub mod prelude;
pub mod resources;

use assets::{BPxGeometry, BPxMaterial};
use components::BPxVelocity;
use resources::{BPxCooking, BPxPhysics, BPxScene, BPxTimeSync, BPxDefaultMaterial, BPxVehicleRaycastBuffer, BPxVehicleFrictionPairs};

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

pub struct BPxPlugin {
    pub vehicles: bool,
    pub cooking: bool,
    pub debugger: bool,
    pub gravity: Vec3,
    pub timestep: f32,
}

impl Plugin for BPxPlugin {
    fn build(&self, app: &mut App) {
        let mut physics = BPxPhysics::new(self.debugger, self.vehicles);
        let scene = BPxScene::new(&mut physics, self.gravity);

        app.add_asset::<BPxGeometry>();
        app.add_asset::<BPxMaterial>();

        app.register_type::<BPxVelocity>();

        if self.cooking {
            app.insert_resource(BPxCooking::new(&mut physics));
        }

        if self.vehicles {
            app.insert_resource(BPxVehicleRaycastBuffer::default());
            app.insert_resource(BPxVehicleFrictionPairs::default());
        }

        app.insert_resource(scene);
        app.insert_resource(BPxTimeSync::new(self.timestep));
        app.insert_resource(BPxDefaultMaterial::default());

        // physics must be last (so it will be dropped last)
        app.insert_resource(physics);

        #[derive(Debug, StageLabel)]
        struct PhysXStage;

        let mut stage = SystemStage::parallel();
        stage.add_system(systems::scene_simulate);
        stage.add_system(systems::create_dynamic_actors.after(systems::scene_simulate));
        stage.add_system(systems::writeback_actors.after(systems::scene_simulate));

        app.add_stage_after(CoreStage::Update, PhysXStage, stage);
    }
}

impl Default for BPxPlugin {
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
