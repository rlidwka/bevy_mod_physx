//! Get/set linear and angular velocity of a rigid body, set maximum velocity.
use bevy::prelude::*;
use physx::prelude::*;
use physx::traits::Class;
use physx_sys::{
    PxArticulationReducedCoordinate_getRootAngularVelocity,
    PxArticulationReducedCoordinate_getRootLinearVelocity,
    PxArticulationReducedCoordinate_setRootAngularVelocity_mut,
    PxArticulationReducedCoordinate_setRootLinearVelocity_mut,
};

use crate::prelude::{Scene, *};

#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
/// Two-way sync of the linear and angular velocity of the rigid body.
///
/// Changing these values wakes the actor if it is sleeping.
pub struct Velocity {
    /// The linear velocity of the actor.
    pub linear: Vec3,
    /// The angular velocity of the actor.
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

#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
/// Set the maximum linear and angular velocity permitted for this actor.
///
/// With this component, you can set the maximum linear and angular velocity permitted
/// for this rigid body. Higher velocities are clamped to these value.
pub struct MaxVelocity {
    /// Max allowable linear velocity for actor. Range: [0, PX_MAX_F32). Default: PX_MAX_F32.
    pub linear: f32,
    /// Max allowable angular velocity for actor. Range: [0, PX_MAX_F32). Default: 100.
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

pub struct VelocityPlugin;

impl Plugin for VelocityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Velocity>();
        app.register_type::<MaxVelocity>();

        app.add_systems(PhysicsSchedule, velocity_sync.in_set(PhysicsSet::Sync));
        app.add_systems(PhysicsSchedule, max_velocity_sync.in_set(PhysicsSet::Sync));
    }
}

pub fn velocity_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<(
        Option<&mut RigidDynamicHandle>,
        Option<&ArticulationLinkHandle>,
        Option<&mut ArticulationRootHandle>,
        &mut Velocity,
    ), Or<(Changed<Velocity>, Without<Sleeping>)>>,
) {
    // this function does two things: sets physx property (if changed) or writes it back (if not);
    // we need it to happen inside a single system to avoid change detection loops, but
    // user will experience 1-tick delay on any changes
    for (dynamic, articulation, articulation_base, mut velocity) in actors.iter_mut() {
        if let Some(mut actor) = dynamic {
            let mut velocity_set = false;

            if velocity.is_changed() || actor.is_added() {
                let mut actor_handle = actor.get_mut(&mut scene);

                // should not set initial velocity when component is added, but actor is kinematic
                if !actor_handle.get_rigid_body_flags().contains(RigidBodyFlags::Kinematic) {
                    actor_handle.set_linear_velocity(&velocity.linear.to_physx(), true);
                    actor_handle.set_angular_velocity(&velocity.angular.to_physx(), true);
                    velocity_set = true;
                }
            }

            if !velocity_set {
                let actor_handle = actor.get(&scene);

                let newvel = Velocity::new(
                    actor_handle.get_linear_velocity().to_bevy(),
                    actor_handle.get_angular_velocity().to_bevy(),
                );

                // extra check so we don't mutate on every frame without changes
                if *velocity != newvel { *velocity = newvel; }
            }
        } else if let Some(mut root) = articulation_base {
            if velocity.is_changed() || root.is_added() {
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
        } else if !velocity.is_added() {
            bevy::log::warn!("Velocity component exists, but it's neither a rigid dynamic nor articulation link");
        }
    }
}

pub fn max_velocity_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<
        (
            Option<&mut RigidDynamicHandle>,
            Option<&mut ArticulationLinkHandle>,
            Ref<MaxVelocity>,
        ),
        Or<(
            Added<RigidDynamicHandle>,
            Added<ArticulationLinkHandle>,
            Changed<MaxVelocity>,
        )>,
    >,
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
        } else if !max_velocity.is_added() {
            bevy::log::warn!("MaxVelocity component exists, but it's neither a rigid dynamic nor articulation link");
        };
    }
}
