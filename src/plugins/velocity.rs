use crate::components::RigidDynamicHandle;
use crate::prelude::{Scene, *};
use bevy::prelude::*;
use physx::prelude::*;

#[derive(Component, Debug, Default, PartialEq, Reflect, Clone, Copy)]
pub struct Velocity {
    pub linvel: Vec3,
    pub angvel: Vec3,
}

impl Velocity {
    pub fn new(linvel: Vec3, angvel: Vec3) -> Self {
        Self { linvel, angvel }
    }

    pub fn zero() -> Self {
        Self { ..default() }
    }

    pub fn linear(linvel: Vec3) -> Self {
        Self { linvel, ..default() }
    }

    pub fn angular(angvel: Vec3) -> Self {
        Self { angvel, ..default() }
    }
}

pub struct VelocityPlugin;

impl Plugin for VelocityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Velocity>();
        app.add_system(sync.in_base_set(PhysicsSet::Writeback).in_schedule(PhysicsSchedule));
    }
}

pub fn sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<(&mut RigidDynamicHandle, &mut Velocity)>
) {
    // this function does two things: sets physx property (if changed) or writes it back (if not);
    // we need it to happen inside a single system to avoid change detection loops, but
    // user will experience 1-tick delay on any changes
    for (mut actor, mut velocity) in actors.iter_mut() {
        if velocity.is_changed() {
            let mut actor_handle = actor.get_mut(&mut scene);

            actor_handle.set_linear_velocity(&velocity.linvel.to_physx(), true);
            actor_handle.set_angular_velocity(&velocity.angvel.to_physx(), true);
        } else {
            let actor_handle = actor.get(&scene);

            let newvel = Velocity::new(
                actor_handle.get_linear_velocity().to_bevy(),
                actor_handle.get_angular_velocity().to_bevy(),
            );

            // extra check so we don't mutate on every frame without changes
            if *velocity != newvel { *velocity = newvel; }
        }
    }
}
