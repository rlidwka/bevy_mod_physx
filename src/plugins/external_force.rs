// this covers ExternalForce and ExternalImpulse (see ForceMode attribute of the struct)
use crate::components::{RigidDynamicHandle, ArticulationLinkHandle};
use crate::prelude::{Scene, *};
use bevy::prelude::*;
use physx::prelude::*;

#[derive(Component, Debug, PartialEq, Reflect, Clone, Copy)]
pub struct ExternalForce {
    pub force: Vec3,
    pub torque: Vec3,
    #[reflect(ignore)]
    // TODO: https://github.com/bevyengine/bevy/pull/6042
    pub mode: ForceMode,
}

impl ExternalForce {
    pub fn at_point(force: Vec3, point: Vec3, center_of_mass: Vec3) -> Self {
        Self {
            force,
            torque: (point - center_of_mass).cross(force),
            ..default()
        }
    }
}

impl Default for ExternalForce {
    fn default() -> Self {
        Self { force: default(), torque: default(), mode: ForceMode::Force }
    }
}

pub struct ExternalForcePlugin;

impl Plugin for ExternalForcePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ExternalForce>();
        app.add_system(external_force_sync.in_base_set(PhysicsSet::Sync).in_schedule(PhysicsSchedule));
    }
}

pub fn external_force_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<(Option<&mut RigidDynamicHandle>, Option<&mut ArticulationLinkHandle>, &ExternalForce)>
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (dynamic, articulation, extforce) in actors.iter_mut() {
        if extforce.force != Vec3::ZERO || extforce.torque != Vec3::ZERO {
            if let Some(mut actor) = dynamic {
                let mut actor_handle = actor.get_mut(&mut scene);
                actor_handle.set_force_and_torque(&extforce.force.to_physx(), &extforce.torque.to_physx(), extforce.mode);
            } else if let Some(mut actor) = articulation {
                let mut actor_handle = actor.get_mut(&mut scene);
                actor_handle.set_force_and_torque(&extforce.force.to_physx(), &extforce.torque.to_physx(), extforce.mode);
            } else {
                bevy::log::warn!("ExternalForce component exists, but it's neither a rigid dynamic nor articulation link");
            };
        }
    }
}
