use crate::prelude::*;
use bevy::prelude::*;
use physx::vehicles::*;

mod components;
pub use components::{Vehicle, VehicleHandle};

mod simulation;
pub use simulation::{VehicleSimulation, VehicleSimulationMethod};

use crate::{PhysicsSchedule, PhysicsSet};

#[derive(Clone)]
pub struct VehicleExtensionDescriptor {
    pub enabled: bool,
    pub basis_vectors: [ Vec3; 2 ],
    pub update_mode: VehicleUpdateMode,
    pub max_hit_actor_acceleration: f32,
    pub sweep_hit_rejection_angles: [ f32; 2 ],
    pub simulation_method: VehicleSimulationMethod,
}

impl Default for VehicleExtensionDescriptor {
    fn default() -> Self {
        Self {
            enabled: true,
            basis_vectors: [ Vec3::new(0., 1., 0.), Vec3::new(0., 0., 1.) ],
            update_mode: VehicleUpdateMode::VelocityChange,
            max_hit_actor_acceleration: std::f32::MAX,
            sweep_hit_rejection_angles: [ 0.707f32, 0.707f32 ],
            simulation_method: VehicleSimulationMethod::Sweep {
                nb_hits_per_query: 1,
                sweep_width_scale: 1.,
                sweep_radius_scale: 1.01,
            },
        }
    }
}

pub struct VehiclePlugin(pub VehicleExtensionDescriptor);

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PhysicsSchedule, (
            simulation::scene_simulate_vehicles.before(super::systems::scene_simulate),
        ).in_set(PhysicsSet::Simulate));

        app.add_systems(PhysicsSchedule, (
            // needs to be after mass_properties to calculate suspension correctly
            // (and after calculating transforms for shapes),
            // so we put this on the next tick after actor is created
            simulation::create_vehicle_actors
                .before(super::systems::create_rigid_actors),
        ).in_set(PhysicsSet::Create));
    }

    fn finish(&self, app: &mut App) {
        app.insert_resource(VehicleSimulation::new(self.0.simulation_method));

        vehicle_set_basis_vectors(
            self.0.basis_vectors[0].to_physx(),
            self.0.basis_vectors[1].to_physx(),
        );
        vehicle_set_update_mode(self.0.update_mode);
        vehicle_set_max_hit_actor_acceleration(self.0.max_hit_actor_acceleration);
        vehicle_set_sweep_hit_rejection_angles(
            self.0.sweep_hit_rejection_angles[0],
            self.0.sweep_hit_rejection_angles[1],
        );
    }
}
