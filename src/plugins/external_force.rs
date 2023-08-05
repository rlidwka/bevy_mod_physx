//! Continuously apply force and torque to the actor.
//!
//! Force and torque are defined in the global coordinate frame.
//!
//! This is similar to ExternalForce and ExternalImpulse in rapier
//! (see ForceMode attribute of the struct).
use bevy::prelude::*;
use physx::prelude::*;

use crate::components::{ArticulationLinkHandle, RigidDynamicHandle};
use crate::prelude::{Scene, *};

#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
/// Parameter to [ExternalForce], determines the exact operation that is carried out.
pub enum ExternalForceMode {
    /// parameter has unit of length/ time^2, i.e., an acceleration. It gets treated just like
    /// a force except the mass is not divided out before integration.
    Acceleration,
    /// parameter has unit of mass * length / time, i.e., force * time
    Impulse,
    /// parameter has unit of length / time, i.e., the effect is mass independent: a velocity change.
    VelocityChange,
    #[default]
    /// parameter has unit of mass * length / time^2, i.e., a force
    Force,
}

impl From<ForceMode> for ExternalForceMode {
    fn from(value: ForceMode) -> Self {
        match value {
            ForceMode::Acceleration => Self::Acceleration,
            ForceMode::Impulse => Self::Impulse,
            ForceMode::VelocityChange => Self::VelocityChange,
            ForceMode::Force => Self::Force,
        }
    }
}

impl From<ExternalForceMode> for ForceMode {
    fn from(value: ExternalForceMode) -> Self {
        match value {
            ExternalForceMode::Acceleration => Self::Acceleration,
            ExternalForceMode::Impulse => Self::Impulse,
            ExternalForceMode::VelocityChange => Self::VelocityChange,
            ExternalForceMode::Force => Self::Force,
        }
    }
}

#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
/// Continuously apply force and torque to the actor.
pub struct ExternalForce {
    /// Force/Impulse to apply defined in the global frame.
    pub force: Vec3,
    /// Torque to apply defined in the global frame.
    pub torque: Vec3,
    /// The mode to use when applying the force/impulse.
    pub mode: ExternalForceMode,
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

pub struct ExternalForcePlugin;

impl Plugin for ExternalForcePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ExternalForce>();
        app.add_systems(PhysicsSchedule, external_force_sync.in_set(PhysicsSet::Sync));
    }
}

pub fn external_force_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<(
        Option<&mut RigidDynamicHandle>,
        Option<&mut ArticulationLinkHandle>,
        Ref<ExternalForce>,
    )>,
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (dynamic, articulation, extforce) in actors.iter_mut() {
        if extforce.force != Vec3::ZERO || extforce.torque != Vec3::ZERO {
            if let Some(mut actor) = dynamic {
                let mut actor_handle = actor.get_mut(&mut scene);
                actor_handle.set_force_and_torque(&extforce.force.to_physx(), &extforce.torque.to_physx(), extforce.mode.into());
            } else if let Some(mut actor) = articulation {
                let mut actor_handle = actor.get_mut(&mut scene);
                actor_handle.set_force_and_torque(&extforce.force.to_physx(), &extforce.torque.to_physx(), extforce.mode.into());
            } else if !extforce.is_added() {
                bevy::log::warn!("ExternalForce component exists, but it's neither a rigid dynamic nor articulation link");
            };
        }
    }
}
