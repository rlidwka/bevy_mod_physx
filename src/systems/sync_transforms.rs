use bevy::prelude::*;
use physx::prelude::*;
use physx::traits::Class;
use physx_sys::{PxShape_getLocalPose, PxShape_setLocalPose_mut};

use crate::components::{
    ArticulationLinkHandle,
    RigidDynamicHandle,
    RigidStaticHandle,
    ShapeHandle,
};
use crate::prelude::{self as bpx, *};

pub fn sync_transform_dynamic(
    mut scene: ResMut<bpx::Scene>,
    global_transforms: Query<&GlobalTransform>,
    mut actors: Query<(
        &mut RigidDynamicHandle,
        &mut Transform,
        &GlobalTransform,
        Option<&Parent>,
    )>,
    // TODO:
    // Or<(Changed<Transform>, Without<Sleeping>)>>,
) {
    // this function does two things: sets physx property (if changed) or writes it back (if not);
    // we need it to happen inside a single system to avoid change detection loops, but
    // user will experience 1-tick delay on any changes
    for (mut actor, mut xform, gxform, parent) in actors.iter_mut() {
        if *gxform != actor.predicted_gxform {
            actor.get_mut(&mut scene).set_global_pose(&gxform.to_physx(), true);
            actor.predicted_gxform = *gxform;
        } else {
            let actor_handle = actor.get(&scene);
            let new_gxform = actor_handle.get_global_pose().to_bevy();
            let mut new_xform = new_gxform;

            if let Some(parent_transform) = parent.and_then(|p| global_transforms.get(**p).ok()) {
                let (_scale, inv_rotation, inv_translation) =
                    parent_transform.affine().inverse().to_scale_rotation_translation();

                new_xform.rotation = inv_rotation * new_xform.rotation;
                new_xform.translation = inv_rotation * new_xform.translation + inv_translation;
            }

            // avoid triggering bevy's change tracking if no change
            if *xform != new_xform { *xform = new_xform; }

            drop(actor_handle);
            actor.predicted_gxform = new_gxform.into();
        }
    }
}

pub fn sync_transform_articulation_links(
    mut scene: ResMut<bpx::Scene>,
    global_transforms: Query<&GlobalTransform>,
    mut actors: Query<(&mut ArticulationLinkHandle, &mut Transform, &GlobalTransform, Option<&Parent>)>,
) {
    // this function does two things: sets physx property (if changed) or writes it back (if not);
    // we need it to happen inside a single system to avoid change detection loops, but
    // user will experience 1-tick delay on any changes
    for (mut actor, mut xform, gxform, parent) in actors.iter_mut() {
        if *gxform != actor.predicted_gxform {
            actor.get_mut(&mut scene).set_global_pose(&gxform.to_physx(), true);
            actor.predicted_gxform = *gxform;
        } else {
            let actor_handle = actor.get(&scene);
            let new_gxform = actor_handle.get_global_pose().to_bevy();
            let mut new_xform = new_gxform;

            if let Some(parent_transform) = parent.and_then(|p| global_transforms.get(**p).ok()) {
                let (_scale, inv_rotation, inv_translation) =
                    parent_transform.affine().inverse().to_scale_rotation_translation();

                new_xform.rotation = inv_rotation * new_xform.rotation;
                new_xform.translation = inv_rotation * new_xform.translation + inv_translation;
            }

            // avoid triggering bevy's change tracking if no change
            if *xform != new_xform { *xform = new_xform; }

            drop(actor_handle);
            actor.predicted_gxform = new_gxform.into();
        }
    }
}

pub fn sync_transform_static(
    mut scene: ResMut<bpx::Scene>,
    mut actors: Query<(&mut RigidStaticHandle, &GlobalTransform), Changed<GlobalTransform>>,
) {
    // we don't expect static object position to change from physx, so we only sync user changes
    for (mut actor, gxform) in actors.iter_mut() {
        if *gxform != actor.predicted_gxform {
            actor.get_mut(&mut scene).set_global_pose(&gxform.to_physx(), true);
            actor.predicted_gxform = *gxform;
        }
    }
}

pub fn sync_transform_nested_shapes(
    mut scene: ResMut<bpx::Scene>,
    mut shapes: Query<
        (&mut ShapeHandle, &mut Transform),
        (Without<RigidStaticHandle>, Without<RigidDynamicHandle>, Without<ArticulationLinkHandle>)
    >,
) {
    // this function does two things: sets physx property (if changed) or writes it back (if not);
    // we need it to happen inside a single system to avoid change detection loops, but
    // user will experience 1-tick delay on any changes
    for (mut shape, mut shape_xform) in shapes.iter_mut() {
        // we assume that nested Shape is always a child of an Actor parent in bevy hierarchy,
        // (so physx hierarchy matches bevy's), otherwise math gets too complicated and expensive
        if shape_xform.is_changed() {
            let custom_transform = shape.custom_xform;
            let mut bevy_xform = *shape_xform;

            if custom_transform != Transform::IDENTITY {
                bevy_xform = custom_transform * bevy_xform;
            }

            unsafe {
                PxShape_setLocalPose_mut(
                    shape.get_mut(&mut scene).as_mut_ptr(),
                    bevy_xform.to_physx().as_ptr(),
                );
            }
        } else {
            let custom_transform = shape.custom_xform;
            let handle = shape.get(&scene);
            let mut physx_xform = unsafe {
                PxShape_getLocalPose(handle.as_ptr())
            }.to_bevy();

            if custom_transform != Transform::IDENTITY {
                let (_scale, inv_rotation, inv_translation) =
                custom_transform.compute_affine().inverse().to_scale_rotation_translation();

                physx_xform.rotation = inv_rotation * physx_xform.rotation;
                physx_xform.translation = inv_rotation * physx_xform.translation + inv_translation;
            }

            // avoid triggering bevy's change tracking if no change
            if *shape_xform != physx_xform { *shape_xform = physx_xform; }
        }
    }
}
