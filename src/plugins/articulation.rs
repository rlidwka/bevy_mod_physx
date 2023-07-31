use bevy::prelude::*;
use physx::prelude::*;
use physx::traits::Class;
use physx_sys::{PxArticulationDrive, PxArticulationLink_getInboundJoint, PxArticulationJointReducedCoordinate_setDriveVelocity_mut, PxArticulationJointReducedCoordinate_getJointPosition, PxArticulationJointReducedCoordinate_setJointPosition_mut};

use crate::components::{ArticulationLinkHandle, ArticulationRootHandle};
use crate::prelude::{Scene, *};

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
    pub twist: PxArticulationDrive,
    pub swing1: PxArticulationDrive,
    pub swing2: PxArticulationDrive,
    pub x: PxArticulationDrive,
    pub y: PxArticulationDrive,
    pub z: PxArticulationDrive,
}

impl Default for ArticulationJointDrives {
    fn default() -> Self {
        let default = PxArticulationDrive { stiffness: 0., damping: 0., maxForce: 0., driveType: ArticulationDriveType::None };
        Self {
            twist: default,
            swing1: default,
            swing2: default,
            x: default,
            y: default,
            z: default,
        }
    }
}

#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
pub struct ArticulationJointDriveTarget {
    pub twist: f32,
    pub swing1: f32,
    pub swing2: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
pub struct ArticulationJointDriveVelocity {
    pub twist: f32,
    pub swing1: f32,
    pub swing2: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
pub struct ArticulationJointPosition {
    pub twist: f32,
    pub swing1: f32,
    pub swing2: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct ArticulationPlugin;

impl Plugin for ArticulationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ArticulationRoot>();
        app.register_type::<ArticulationJointDriveTarget>();
        app.register_type::<ArticulationJointDriveVelocity>();
        app.register_type::<ArticulationJointPosition>();

        app.add_systems(PhysicsSchedule, (
            articulation_root_sync,
            articulation_drives_sync,
            articulation_drive_target_sync,
            articulation_drive_velocity_sync,
            articulation_joint_position_sync,
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
            //With<ArticulationJoint>,
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
            let handle = link.get_mut(&mut scene);

            let joint = unsafe { PxArticulationLink_getInboundJoint(handle.as_ptr()) };
            assert!(!joint.is_null());

            // SAFETY: ArticulationJointReducedCoordinate is repr(transparent) wrapper
            let joint = unsafe { &mut *(joint as *mut ArticulationJointReducedCoordinate) };

            fn set_drive(joint: &mut ArticulationJointReducedCoordinate, axis: ArticulationAxis, value: PxArticulationDrive) {
                joint.set_drive(axis, value.stiffness, value.damping, value.maxForce, value.driveType);
            }

            set_drive(joint, ArticulationAxis::Twist, drives.twist);
            set_drive(joint, ArticulationAxis::Swing1, drives.swing1);
            set_drive(joint, ArticulationAxis::Swing2, drives.swing2);
            set_drive(joint, ArticulationAxis::X, drives.x);
            set_drive(joint, ArticulationAxis::Y, drives.y);
            set_drive(joint, ArticulationAxis::Z, drives.z);
        } else if !drives.is_added() {
            bevy::log::warn!("ArticulationJointDrives component exists, but it's not an articulation link with inbound joint");
        };
    }
}

pub fn articulation_drive_target_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<
        (
            Option<&mut ArticulationLinkHandle>,
            &mut ArticulationJointDriveTarget,
        ),
        (
            //With<ArticulationJoint>,
            Or<(
                Added<ArticulationLinkHandle>,
                Changed<ArticulationJointDriveTarget>,
            )>,
        ),
    >,
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (link, params) in actors.iter_mut() {
        if let Some(mut link) = link {
            let handle = link.get_mut(&mut scene);

            let joint = unsafe { PxArticulationLink_getInboundJoint(handle.as_ptr()) };
            assert!(!joint.is_null());

            // SAFETY: ArticulationJointReducedCoordinate is repr(transparent) wrapper
            let joint = unsafe { &mut *(joint as *mut ArticulationJointReducedCoordinate) };

            fn set_drive_param(joint: &mut ArticulationJointReducedCoordinate, axis: ArticulationAxis, value: f32) {
                joint.set_drive_target(value, axis);
            }

            set_drive_param(joint, ArticulationAxis::Twist, params.twist);
            set_drive_param(joint, ArticulationAxis::Swing1, params.swing1);
            set_drive_param(joint, ArticulationAxis::Swing2, params.swing2);
            set_drive_param(joint, ArticulationAxis::X, params.x);
            set_drive_param(joint, ArticulationAxis::Y, params.y);
            set_drive_param(joint, ArticulationAxis::Z, params.z);
        } else if !params.is_added() {
            bevy::log::warn!("ArticulationJointDriveTarget component exists, but it's not an articulation link with inbound joint");
        }
    }
}

pub fn articulation_drive_velocity_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<
        (
            Option<&mut ArticulationLinkHandle>,
            &mut ArticulationJointDriveVelocity,
        ),
        (
            //With<ArticulationJoint>,
            Or<(
                Added<ArticulationLinkHandle>,
                Changed<ArticulationJointDriveVelocity>,
            )>,
        ),
    >,
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (link, params) in actors.iter_mut() {
        if let Some(mut link) = link {
            let handle = link.get_mut(&mut scene);

            let joint = unsafe { PxArticulationLink_getInboundJoint(handle.as_ptr()) };
            assert!(!joint.is_null());

            // SAFETY: ArticulationJointReducedCoordinate is repr(transparent) wrapper
            let joint = unsafe { &mut *(joint as *mut ArticulationJointReducedCoordinate) };

            fn set_drive_param(joint: &mut ArticulationJointReducedCoordinate, axis: ArticulationAxis, value: f32) {
                unsafe {
                    PxArticulationJointReducedCoordinate_setDriveVelocity_mut(
                        joint.as_mut_ptr(), axis, value, true
                    );
                };
            }

            set_drive_param(joint, ArticulationAxis::Twist, params.twist);
            set_drive_param(joint, ArticulationAxis::Swing1, params.swing1);
            set_drive_param(joint, ArticulationAxis::Swing2, params.swing2);
            set_drive_param(joint, ArticulationAxis::X, params.x);
            set_drive_param(joint, ArticulationAxis::Y, params.y);
            set_drive_param(joint, ArticulationAxis::Z, params.z);
        } else if !params.is_added() {
            bevy::log::warn!("ArticulationJointDriveVelocity component exists, but it's not an articulation link with inbound joint");
        }
    }
}

pub fn articulation_joint_position_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<(
        Option<&mut ArticulationLinkHandle>,
        &mut ArticulationJointPosition,
    )>,
) {
    // this function does two things: sets physx property (if changed) or writes it back (if not);
    // we need it to happen inside a single system to avoid change detection loops, but
    // user will experience 1-tick delay on any changes
    for (link, mut params) in actors.iter_mut() {
        if let Some(mut link) = link {
            if params.is_changed() || link.is_added() {
                let handle = link.get_mut(&mut scene);

                let joint = unsafe { PxArticulationLink_getInboundJoint(handle.as_ptr()) };
                assert!(!joint.is_null());

                // SAFETY: ArticulationJointReducedCoordinate is repr(transparent) wrapper
                let joint = unsafe { &mut *(joint as *mut ArticulationJointReducedCoordinate) };

                fn set_drive_param(joint: &mut ArticulationJointReducedCoordinate, axis: ArticulationAxis, value: f32) {
                    unsafe {
                        PxArticulationJointReducedCoordinate_setJointPosition_mut(
                            joint.as_mut_ptr(), axis, value
                        );
                    };
                }

                set_drive_param(joint, ArticulationAxis::Twist, params.twist);
                set_drive_param(joint, ArticulationAxis::Swing1, params.swing1);
                set_drive_param(joint, ArticulationAxis::Swing2, params.swing2);
                set_drive_param(joint, ArticulationAxis::X, params.x);
                set_drive_param(joint, ArticulationAxis::Y, params.y);
                set_drive_param(joint, ArticulationAxis::Z, params.z);
            } else {
                let handle = link.get_mut(&mut scene);

                let joint = unsafe { PxArticulationLink_getInboundJoint(handle.as_ptr()) };
                assert!(!joint.is_null());

                // SAFETY: ArticulationJointReducedCoordinate is repr(transparent) wrapper
                let joint = unsafe { &mut *(joint as *mut ArticulationJointReducedCoordinate) };

                fn get_drive_param(joint: &ArticulationJointReducedCoordinate, axis: ArticulationAxis) -> f32 {
                    unsafe {
                        PxArticulationJointReducedCoordinate_getJointPosition(
                            joint.as_ptr(), axis
                        )
                    }
                }

                let new_params = ArticulationJointPosition {
                    twist: get_drive_param(joint, ArticulationAxis::Twist),
                    swing1: get_drive_param(joint, ArticulationAxis::Swing1),
                    swing2: get_drive_param(joint, ArticulationAxis::Swing2),
                    x: get_drive_param(joint, ArticulationAxis::X),
                    y: get_drive_param(joint, ArticulationAxis::Y),
                    z: get_drive_param(joint, ArticulationAxis::Z),
                };

                // extra check so we don't mutate on every frame without changes
                if *params != new_params { *params = new_params }
            }
        } else if !params.is_added() {
            bevy::log::warn!("ArticulationJointPosition component exists, but it's not an articulation link with inbound joint");
        }
    }
}
