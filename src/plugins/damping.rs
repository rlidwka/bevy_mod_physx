use crate::components::{ArticulationLinkHandle, RigidDynamicHandle};
use crate::prelude::{Scene, *};
use bevy::prelude::*;
use physx::prelude::*;

#[derive(Component, Debug, Default, PartialEq, Reflect, Clone, Copy)]
pub struct Damping {
    pub linear: f32,
    pub angular: f32,
}

impl Damping {
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

pub struct DampingPlugin;

impl Plugin for DampingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Damping>();
        app.add_system(damping_sync.in_base_set(PhysicsSet::Sync).in_schedule(PhysicsSchedule));
    }
}

pub fn damping_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<(Option<&mut RigidDynamicHandle>, Option<&mut ArticulationLinkHandle>, &Damping), Changed<Damping>>
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (dynamic, articulation, damping) in actors.iter_mut() {
        if let Some(mut actor) = dynamic {
            let mut actor_handle = actor.get_mut(&mut scene);
            actor_handle.set_linear_damping(damping.linear);
            actor_handle.set_angular_damping(damping.angular);
        } else if let Some(mut actor) = articulation {
            let mut actor_handle = actor.get_mut(&mut scene);
            actor_handle.set_linear_damping(damping.linear);
            actor_handle.set_angular_damping(damping.angular);
        } else {
            bevy::log::warn!("Damping component exists, but it's neither a rigid dynamic nor articulation link");
        };
    }
}
