use crate::components::RigidDynamicHandle;
use crate::prelude::{Scene, *};
use bevy::prelude::*;
use physx::prelude::*;

#[derive(Component, Debug, Default, PartialEq, Reflect, Clone, Copy)]
pub struct Damping {
    pub linear_damping: f32,
    pub angular_damping: f32,
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
    mut actors: Query<(&mut RigidDynamicHandle, &Damping), Changed<Damping>>
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (mut actor, damping) in actors.iter_mut() {
        let mut actor_handle = actor.get_mut(&mut scene);
        actor_handle.set_linear_damping(damping.linear_damping);
        actor_handle.set_angular_damping(damping.angular_damping);
    }
}
