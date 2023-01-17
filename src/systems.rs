use std::ptr::null;
use bevy::prelude::*;
use physx::prelude::*;
use physx::scene::Scene;
use physx::traits::Class;
use physx_sys::{phys_PxCreateDynamic, PxScene_addActor_mut, phys_PxCreateStatic};

use super::{prelude::*, PxRigidDynamic, PxRigidStatic};
use super::assets::{BPxGeometry, BPxMaterial};
use super::components::{BPxActor, BPxRigidDynamic, BPxRigidStatic, BPxVelocity};
use super::resources::{BPxScene, BPxPhysics, BPxTimeSync};

pub fn scene_simulate(time: Res<Time>, mut scene: ResMut<BPxScene>, mut timesync: ResMut<BPxTimeSync>) {
    timesync.advance_bevy_time(&time);

    if let Some(delta) = timesync.check_advance_physx_time() {
        scene.simulate(delta, None, None);
        scene.fetch_results(true).unwrap();
    }
}

pub fn create_dynamic_actors(
    mut commands: Commands,
    mut physics: ResMut<BPxPhysics>,
    mut scene: ResMut<BPxScene>,
    new_actors: Query<(Entity, &BPxActor, &GlobalTransform, Option<&BPxVelocity>), (Without<BPxRigidDynamic>, Without<BPxRigidStatic>)>,
    geometries: Res<Assets<BPxGeometry>>,
    mut materials: ResMut<Assets<BPxMaterial>>,
) {
    for (entity, actor_cfg, transform, velocity) in new_actors.iter() {
        match actor_cfg {
            BPxActor::Dynamic { geometry, material, density, shape_offset } => {
                let geometry = geometries.get(geometry).expect("geometry not found for BPxGeometry");
                let material = materials.get_mut(material).expect("material not found for BPxMaterial");

                let mut actor : Owner<PxRigidDynamic> = unsafe {
                    RigidDynamic::from_raw(
                        phys_PxCreateDynamic(
                            physics.physics_mut().as_mut_ptr(),
                            transform.to_physx().as_ptr(),
                            geometry.as_ptr(),
                            material.as_mut_ptr(),
                            *density,
                            shape_offset.to_physx().as_ptr(),
                        ),
                        (),
                    )
                }.unwrap();

                if let Some(BPxVelocity { linvel, angvel }) = velocity {
                    actor.set_linear_velocity(&linvel.to_physx(), false);
                    actor.set_angular_velocity(&angvel.to_physx(), false);
                }

                unsafe {
                    PxScene_addActor_mut(scene.as_mut_ptr(), actor.as_mut_ptr(), null());
                }

                commands.entity(entity)
                    .insert(BPxRigidDynamic::new(actor));
            }

            BPxActor::Static { geometry, material, shape_offset } => {
                let geometry = geometries.get(geometry).expect("geometry not found for BPxGeometry");
                let material = materials.get_mut(material).expect("material not found for BPxMaterial");

                let mut actor : Owner<PxRigidStatic> = unsafe {
                    RigidStatic::from_raw(
                        phys_PxCreateStatic(
                            physics.physics_mut().as_mut_ptr(),
                            transform.to_physx().as_ptr(),
                            geometry.as_ptr(),
                            material.as_mut_ptr(),
                            shape_offset.to_physx().as_ptr(),
                        ),
                        (),
                    )
                }.unwrap();

                if velocity.is_some() {
                    bevy::log::warn!("ignoring BPxVelocity component from a static actor");
                }

                unsafe {
                    PxScene_addActor_mut(scene.as_mut_ptr(), actor.as_mut_ptr(), null());
                }

                commands.entity(entity)
                    .insert(BPxRigidStatic::new(actor));
            }

            BPxActor::Plane { normal, offset, material } => {
                let material = materials.get_mut(material).expect("material not found for BPxMaterial");
                let mut actor = physics
                    .create_plane(normal.to_physx(), *offset, material, ())
                    .unwrap();

                if velocity.is_some() {
                    bevy::log::warn!("ignoring BPxVelocity component from a static actor");
                }

                unsafe {
                    PxScene_addActor_mut(scene.as_mut_ptr(), actor.as_mut_ptr(), null());
                }

                commands.entity(entity)
                    .insert(BPxRigidStatic::new(actor));
            }
        }
    }
}

pub fn writeback_actors(
    global_transforms: Query<&GlobalTransform>,
    mut actors: Query<(&BPxRigidDynamic, Option<&Parent>, Option<&mut Transform>, Option<&mut BPxVelocity>)>
) {
    for (actor, parent, transform, velocity) in actors.iter_mut() {
        let xform = actor.get_global_pose();

        if let Some(mut transform) = transform {
            let mut new_xform = xform.to_bevy();

            if let Some(parent_transform) = parent.and_then(|p| global_transforms.get(**p).ok()) {
                let (_scale, inv_rotation, inv_translation) =
                    parent_transform.affine().inverse().to_scale_rotation_translation();

                new_xform.rotation = inv_rotation * new_xform.rotation;
                new_xform.translation = inv_rotation * new_xform.translation + inv_translation;
            }

            // avoid triggering bevy's change tracking if no change
            if new_xform != *transform { *transform = new_xform; }
        }

        if let Some(mut velocity) = velocity {
            let newvel = BPxVelocity::new(
                actor.get_linear_velocity().to_bevy(),
                actor.get_angular_velocity().to_bevy(),
            );

            // avoid triggering bevy's change tracking if no change
            if newvel != *velocity { *velocity = newvel; }
        }
    }
}
