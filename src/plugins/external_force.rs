// this covers ExternalForce and ExternalImpulse (see ForceMode attribute of the struct)
use crate::components::RigidDynamicHandle;
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
        app.add_system(sync.in_base_set(PhysicsSet::Writeback).in_schedule(PhysicsSchedule));
    }
}

pub fn sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<(&mut RigidDynamicHandle, &ExternalForce)>
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (mut actor, extforce) in actors.iter_mut() {
        if extforce.force != Vec3::ZERO || extforce.torque != Vec3::ZERO {
            let mut actor_handle = actor.get_mut(&mut scene);
            actor_handle.set_force_and_torque(&extforce.force.to_physx(), &extforce.torque.to_physx(), extforce.mode);
        }
    }
}
