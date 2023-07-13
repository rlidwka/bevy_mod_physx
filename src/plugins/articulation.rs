use crate::components::{ArticulationLinkHandle, ArticulationRootHandle};
use crate::prelude::{Scene, *};
use bevy::prelude::*;
use physx::prelude::*;
use physx::traits::Class;
use physx_sys::{PxArticulationDrive, PxArticulationLink_getInboundJoint};

#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
pub struct ArticulationRoot {
    pub fix_base: bool,
    pub drive_limits_are_forces: bool,
    pub disable_self_collision: bool,
    pub compute_joint_forces: bool,
}

#[derive(Component, Clone)]
pub struct ArticulationJointDrives {
    pub drive_twist: PxArticulationDrive,
    pub drive_swing1: PxArticulationDrive,
    pub drive_swing2: PxArticulationDrive,
    pub drive_x: PxArticulationDrive,
    pub drive_y: PxArticulationDrive,
    pub drive_z: PxArticulationDrive,
}

impl Default for ArticulationJointDrives {
    fn default() -> Self {
        let default = PxArticulationDrive { stiffness: 0., damping: 0., maxForce: 0., driveType: ArticulationDriveType::None };
        Self {
            drive_twist: default,
            drive_swing1: default,
            drive_swing2: default,
            drive_x: default,
            drive_y: default,
            drive_z: default,
        }
    }
}

#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
pub struct ArticulationJointDriveTargets {
    pub drive_twist: f32,
    pub drive_swing1: f32,
    pub drive_swing2: f32,
    pub drive_x: f32,
    pub drive_y: f32,
    pub drive_z: f32,
}

pub struct ArticulationPlugin;

impl Plugin for ArticulationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ArticulationRoot>();
        app.register_type::<ArticulationJointDriveTargets>();

        app.add_systems(PhysicsSchedule, (
            articulation_root_sync,
            articulation_drives_sync,
            articulation_drive_targets_sync,
        ).in_set(PhysicsSet::Sync));
    }
}

pub fn articulation_root_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<
        (Option<&mut ArticulationRootHandle>, Ref<ArticulationRoot>),
        Or<(Added<ArticulationRootHandle>, Changed<ArticulationRoot>)>,
    >,
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (root, flags) in actors.iter_mut() {
        if let Some(mut root) = root {
            let mut handle = root.get_mut(&mut scene);
            handle.set_articulation_flag(ArticulationFlag::FixBase, flags.fix_base);
            handle.set_articulation_flag(ArticulationFlag::DriveLimitsAreForces, flags.drive_limits_are_forces);
            handle.set_articulation_flag(ArticulationFlag::DisableSelfCollision, flags.disable_self_collision);
            handle.set_articulation_flag(ArticulationFlag::ComputeJointForces, flags.compute_joint_forces);
        } else if !flags.is_added() {
            bevy::log::warn!("ArticulationRoot component exists, but it's not an articulation root");
        };
    }
}

pub fn articulation_drives_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<
        (
            Option<&mut ArticulationLinkHandle>,
            Ref<ArticulationJointDrives>,
        ),
        (
            With<ArticulationJoint>,
            Or<(
                Added<ArticulationLinkHandle>,
                Changed<ArticulationJointDrives>,
            )>,
        ),
    >,
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (link, drives) in actors.iter_mut() {
        if let Some(mut link) = link {
            let mut handle = link.get_mut(&mut scene);

            let joint = unsafe { PxArticulationLink_getInboundJoint(handle.as_mut_ptr()) };
            assert!(!joint.is_null());

            // SAFETY: ArticulationJointReducedCoordinate is repr(transparent) wrapper
            let joint = unsafe { &mut *(joint as *mut ArticulationJointReducedCoordinate) };

            fn set_drive(joint: &mut ArticulationJointReducedCoordinate, axis: ArticulationAxis, value: PxArticulationDrive) {
                joint.set_drive(axis, value.stiffness, value.damping, value.maxForce, value.driveType);
            }

            set_drive(joint, ArticulationAxis::Twist, drives.drive_twist);
            set_drive(joint, ArticulationAxis::Swing1, drives.drive_swing1);
            set_drive(joint, ArticulationAxis::Swing2, drives.drive_swing2);
            set_drive(joint, ArticulationAxis::X, drives.drive_x);
            set_drive(joint, ArticulationAxis::Y, drives.drive_y);
            set_drive(joint, ArticulationAxis::Z, drives.drive_z);
        } else if !drives.is_added() {
            bevy::log::warn!("ArticulationJointDrives component exists, but it's not an articulation link with inbound joint");
        };
    }
}

pub fn articulation_drive_targets_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<
        (
            Option<&mut ArticulationLinkHandle>,
            &mut ArticulationJointDriveTargets,
        ),
        (
            With<ArticulationJoint>,
            Or<(
                Added<ArticulationLinkHandle>,
                Changed<ArticulationJointDriveTargets>,
            )>,
        ),
    >,
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (link, drive_targets) in actors.iter_mut() {
        if let Some(mut link) = link {
            let mut handle = link.get_mut(&mut scene);

            let joint = unsafe { PxArticulationLink_getInboundJoint(handle.as_mut_ptr()) };
            assert!(!joint.is_null());

            // SAFETY: ArticulationJointReducedCoordinate is repr(transparent) wrapper
            let joint = unsafe { &mut *(joint as *mut ArticulationJointReducedCoordinate) };

            fn set_drive_target(joint: &mut ArticulationJointReducedCoordinate, axis: ArticulationAxis, value: f32) {
                joint.set_drive_target(value, axis);
            }

            set_drive_target(joint, ArticulationAxis::Twist, drive_targets.drive_twist);
            set_drive_target(joint, ArticulationAxis::Swing1, drive_targets.drive_swing1);
            set_drive_target(joint, ArticulationAxis::Swing2, drive_targets.drive_swing2);
            set_drive_target(joint, ArticulationAxis::X, drive_targets.drive_x);
            set_drive_target(joint, ArticulationAxis::Y, drive_targets.drive_y);
            set_drive_target(joint, ArticulationAxis::Z, drive_targets.drive_z);
        } else if !drive_targets.is_added() {
            bevy::log::warn!("ArticulationJointDriveTargets component exists, but it's not an articulation link with inbound joint");
        }
    }
}
