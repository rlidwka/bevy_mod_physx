use bevy::prelude::*;
use physx::prelude::*;
use physx::traits::Class;
use physx_sys::{PxRigidBody_setRigidBodyFlag_mut, PxRigidDynamic_setKinematicTarget_mut};

use crate::components::RigidDynamicHandle;
use crate::prelude::{Scene, *};

#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
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
        let mut handle = actor.get_mut(&mut scene);

        handle.set_global_pose(&kinematic.target.to_physx(), false);
        unsafe { PxRigidBody_setRigidBodyFlag_mut(handle.as_mut_ptr(), RigidBodyFlag::Kinematic, true); }
    }
}

pub fn kinematic_disable(
    mut scene: ResMut<Scene>,
    mut removed: RemovedComponents<Kinematic>,
    mut handles: Query<&mut RigidDynamicHandle>,
) {
    for entity in removed.iter() {
        if let Ok(mut actor) = handles.get_mut(entity) {
            let mut handle = actor.get_mut(&mut scene);

            unsafe { PxRigidBody_setRigidBodyFlag_mut(handle.as_mut_ptr(), RigidBodyFlag::Kinematic, false); }
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
            let mut handle = actor.get_mut(&mut scene);
            unsafe {
                PxRigidDynamic_setKinematicTarget_mut(
                    handle.as_mut_ptr(),
                    kinematic.target.to_physx().as_ptr(),
                );
            }
        } else if !kinematic.is_added() {
            bevy::log::warn!("Kinematic component exists, but it's not a rigid dynamic");
        };
    }
}
