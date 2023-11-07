//! Enable kinematic mode for an actor, set kinematic target.
//!
//! When [Kinematic] component is added, rigid body is set to be
//! kinematic body, and its target will be updated each frame
//! to [Kinematic::target]. When the component is removed, body
//! stops being kinematic.
//!
//! Check out `examples/kinematic.rs` as an example.
use bevy::prelude::*;
use physx::prelude::*;

use crate::prelude::{Scene, *};

#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
/// Enable kinematic mode for an actor, set kinematic target.
pub struct Kinematic {
    pub target: Transform,
}

impl Kinematic {
    pub fn new(target: Transform) -> Self {
        Self { target }
    }
}

pub struct KinematicPlugin;

impl Plugin for KinematicPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Kinematic>();
        app.add_systems(PhysicsSchedule, (
            kinematic_enable,
            kinematic_disable,
            kinematic_apply,
        ).chain().in_set(PhysicsSet::Sync));
    }
}

pub fn kinematic_enable(
    mut scene: ResMut<Scene>,
    mut added: Query<
        (&mut RigidDynamicHandle, &Kinematic),
        Or<(
            Added<RigidDynamicHandle>,
            Added<Kinematic>,
        )>,
    >,
) {
    for (mut actor, kinematic) in added.iter_mut() {
        let mut rigid_body = actor.get_mut(&mut scene);

        rigid_body.set_global_pose(&kinematic.target.to_physx(), false);
        rigid_body.set_rigid_body_flag(RigidBodyFlag::Kinematic, true);
    }
}

pub fn kinematic_disable(
    mut scene: ResMut<Scene>,
    mut removed: RemovedComponents<Kinematic>,
    mut handles: Query<&mut RigidDynamicHandle>,
) {
    for entity in removed.read() {
        if let Ok(mut actor) = handles.get_mut(entity) {
            let mut rigid_body = actor.get_mut(&mut scene);

            rigid_body.set_rigid_body_flag(RigidBodyFlag::Kinematic, false);

            // Kinematic body might be sleeping in awkward places (e.g. midair), and if it becomes
            // dynamic, it doesn't wake up automatically. We need to wake it up and force it to
            // re-evaluate its life choices.
            rigid_body.wake_up();
        };
    }
}

pub fn kinematic_apply(
    mut scene: ResMut<Scene>,
    mut actors: Query<
        (Option<&mut RigidDynamicHandle>, Ref<Kinematic>),
        Or<(Added<RigidDynamicHandle>, Changed<Kinematic>)>,
    >,
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (actor, kinematic) in actors.iter_mut() {
        if let Some(mut actor) = actor {
            let mut rigid_body = actor.get_mut(&mut scene);

            rigid_body.set_kinematic_target(&kinematic.target.to_physx());

        } else if !kinematic.is_added() {
            bevy::log::warn!("Kinematic component exists, but it's not a rigid dynamic");
        };
    }
}
