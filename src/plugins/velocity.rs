use crate::components::{ArticulationLinkHandle, ArticulationRootHandle, RigidDynamicHandle};
use crate::prelude::{Scene, *};
use bevy::prelude::*;
use physx::prelude::*;

#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect, FromReflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
pub struct Velocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

impl Velocity {
    pub fn new(linear: Vec3, angular: Vec3) -> Self {
        Self { linear, angular }
    }

    pub fn zero() -> Self {
        Self { ..default() }
    }

    pub fn linear(linear: Vec3) -> Self {
        Self { linear, ..default() }
    }

    pub fn angular(angular: Vec3) -> Self {
        Self { angular, ..default() }
    }
}

pub struct VelocityPlugin;

impl Plugin for VelocityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Velocity>();
        app.add_system(velocity_sync.in_base_set(PhysicsSet::Sync).in_schedule(PhysicsSchedule));
    }
}

pub fn velocity_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<(Option<&mut RigidDynamicHandle>, Option<&ArticulationLinkHandle>, Option<&mut ArticulationRootHandle>, &mut Velocity)>
) {
    // this function does two things: sets physx property (if changed) or writes it back (if not);
    // we need it to happen inside a single system to avoid change detection loops, but
    // user will experience 1-tick delay on any changes
    for (dynamic, articulation, _articulation_base, mut velocity) in actors.iter_mut() {
        if let Some(mut actor) = dynamic {
            if velocity.is_changed() {
                let mut actor_handle = actor.get_mut(&mut scene);

                actor_handle.set_linear_velocity(&velocity.linear.to_physx(), true);
                actor_handle.set_angular_velocity(&velocity.angular.to_physx(), true);
            } else {
                let actor_handle = actor.get(&scene);

                let newvel = Velocity::new(
                    actor_handle.get_linear_velocity().to_bevy(),
                    actor_handle.get_angular_velocity().to_bevy(),
                );

                // extra check so we don't mutate on every frame without changes
                if *velocity != newvel { *velocity = newvel; }
            }
        } else if let Some(actor) = articulation {
            // velocity for articulation link cannot be changed by a user
            // so this is just writeback
            let actor_handle = actor.get(&scene);

            let newvel = Velocity::new(
                actor_handle.get_linear_velocity().to_bevy(),
                actor_handle.get_angular_velocity().to_bevy(),
            );

            // extra check so we don't mutate on every frame without changes
            if *velocity != newvel { *velocity = newvel; }
        } else {
            bevy::log::warn!("Velocity component exists, but it's neither a rigid dynamic nor articulation link");
        }
    }
}
