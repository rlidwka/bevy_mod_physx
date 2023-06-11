use crate::components::{ArticulationLinkHandle, ArticulationRootHandle, RigidDynamicHandle};
use crate::prelude::{Scene, *};
use bevy::prelude::*;
use physx::prelude::*;
use physx::traits::Class;
use physx_sys::{
    PxArticulationReducedCoordinate_getRootAngularVelocity,
    PxArticulationReducedCoordinate_getRootLinearVelocity,
    PxArticulationReducedCoordinate_setRootAngularVelocity_mut,
    PxArticulationReducedCoordinate_setRootLinearVelocity_mut,
};

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
    for (dynamic, articulation, articulation_base, mut velocity) in actors.iter_mut() {
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
        } else if let Some(mut root) = articulation_base {
            if velocity.is_changed() {
                let ptr = root.get_mut(&mut scene).as_mut_ptr();

                unsafe {
                    PxArticulationReducedCoordinate_setRootLinearVelocity_mut(ptr, &velocity.linear.to_physx_sys(), true);
                    PxArticulationReducedCoordinate_setRootAngularVelocity_mut(ptr, &velocity.angular.to_physx_sys(), true);
                }
            } else {
                let ptr = root.get(&scene).as_ptr();

                let newvel = Velocity::new(
                    unsafe { PxArticulationReducedCoordinate_getRootLinearVelocity(ptr) }.to_bevy(),
                    unsafe { PxArticulationReducedCoordinate_getRootAngularVelocity(ptr) }.to_bevy(),
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
