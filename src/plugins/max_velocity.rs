use crate::components::{RigidDynamicHandle, ArticulationLinkHandle};
use crate::prelude::{Scene, *};
use bevy::prelude::*;
use physx::prelude::*;

#[derive(Component, Debug, Default, PartialEq, Reflect, Clone, Copy)]
pub struct MaxVelocity {
    pub linear: f32,
    pub angular: f32,
}

impl MaxVelocity {
    pub fn new(linear: f32, angular: f32) -> Self {
        Self { linear, angular }
    }

    pub fn linear(linear: f32) -> Self {
        Self { linear, ..default() }
    }

    pub fn angular(angular: f32) -> Self {
        Self { angular, ..default() }
    }
}

pub struct MaxVelocityPlugin;

impl Plugin for MaxVelocityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MaxVelocity>();
        app.add_system(max_velocity_sync.in_base_set(PhysicsSet::Sync).in_schedule(PhysicsSchedule));
    }
}

pub fn max_velocity_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<(Option<&mut RigidDynamicHandle>, Option<&mut ArticulationLinkHandle>, &MaxVelocity), Changed<MaxVelocity>>
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (dynamic, articulation, max_velocity) in actors.iter_mut() {
        if let Some(mut actor) = dynamic {
            let mut actor_handle = actor.get_mut(&mut scene);
            actor_handle.set_max_linear_velocity(max_velocity.linear);
            actor_handle.set_max_angular_velocity(max_velocity.angular);
        } else if let Some(mut actor) = articulation {
            let mut actor_handle = actor.get_mut(&mut scene);
            actor_handle.set_max_linear_velocity(max_velocity.linear);
            actor_handle.set_max_angular_velocity(max_velocity.angular);
        } else {
            bevy::log::warn!("Damping component exists, but it's neither a rigid dynamic nor articulation link");
        };
    }
}
