use std::ptr::null;
use bevy::prelude::*;
use physx::prelude::*;
use physx::scene::Scene;
use physx::traits::{Class, PxFlags};
use physx_sys::{PxScene_addActor_mut, PxPhysics_createShape_mut, PxRigidBodyExt_updateMassAndInertia_mut_1};

use crate::components::BPxShapeHandle;

use super::{prelude::*, PxRigidDynamic, PxRigidStatic};
use super::assets::{BPxGeometry, BPxMaterial};
use super::components::{BPxActor, BPxRigidDynamicHandle, BPxRigidStaticHandle, BPxShape, BPxVelocity};
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
    new_actors: Query<(Entity, &BPxActor, Option<&BPxShape>, &GlobalTransform, Option<&BPxVelocity>), (Without<BPxRigidDynamicHandle>, Without<BPxRigidStaticHandle>)>,
    geometries: Res<Assets<BPxGeometry>>,
    mut materials: ResMut<Assets<BPxMaterial>>,
) {
    for (entity, actor_cfg, shape_cfg, transform, velocity) in new_actors.iter() {
        match actor_cfg {
            BPxActor::Dynamic { density } => {
                let Some(BPxShape { geometry, material }) = shape_cfg else { panic!() };

                let geometry = geometries.get(geometry).expect("geometry not found for BPxGeometry");
                let material = materials.get_mut(material).expect("material not found for BPxMaterial");

                let mut actor : Owner<PxRigidDynamic> = physics.create_dynamic(&transform.to_physx(), ()).unwrap();

                // create via unsafe raw function call instead of physics.create_shape() because it can't do boxed dyns
                let mut shape = unsafe {
                    Shape::from_raw(
                        PxPhysics_createShape_mut(
                            physics.physics_mut().as_mut_ptr(),
                            geometry.as_ptr(),
                            material.as_ptr(),
                            true,
                            (ShapeFlag::SceneQueryShape | ShapeFlag::SimulationShape | ShapeFlag::Visualization).into_px(),
                        ),
                        ()
                    ).unwrap()
                };

                actor.attach_shape(&mut shape);

                unsafe {
                    PxRigidBodyExt_updateMassAndInertia_mut_1(
                        actor.as_mut_ptr(),
                        *density,
                        null(),
                        false
                    );
                }

                if let Some(BPxVelocity { linvel, angvel }) = velocity {
                    actor.set_linear_velocity(&linvel.to_physx(), false);
                    actor.set_angular_velocity(&angvel.to_physx(), false);
                }

                // unsafe raw function call is required to avoid consuming actor
                unsafe {
                    PxScene_addActor_mut(scene.as_mut_ptr(), actor.as_mut_ptr(), null());
                }

                commands.entity(entity)
                    .insert(BPxRigidDynamicHandle::new(actor))
                    .insert(BPxShapeHandle::new(shape));
            }

            BPxActor::Static => {
                let Some(BPxShape { geometry, material }) = shape_cfg else { panic!() };

                let geometry = geometries.get(geometry).expect("geometry not found for BPxGeometry");
                let material = materials.get_mut(material).expect("material not found for BPxMaterial");

                let mut actor : Owner<PxRigidStatic> = physics.create_static(transform.to_physx(), ()).unwrap();

                // create via unsafe raw function call instead of physics.create_shape() because it can't do boxed dyns
                let mut shape = unsafe {
                    Shape::from_raw(
                        PxPhysics_createShape_mut(
                            physics.physics_mut().as_mut_ptr(),
                            geometry.as_ptr(),
                            material.as_ptr(),
                            true,
                            (ShapeFlag::SceneQueryShape | ShapeFlag::SimulationShape | ShapeFlag::Visualization).into_px(),
                        ),
                        ()
                    ).unwrap()
                };

                actor.attach_shape(&mut shape);

                if velocity.is_some() {
                    bevy::log::warn!("ignoring BPxVelocity component from a static actor");
                }

                // unsafe raw function call is required to avoid consuming actor
                unsafe {
                    PxScene_addActor_mut(scene.as_mut_ptr(), actor.as_mut_ptr(), null());
                }

                commands.entity(entity)
                    .insert(BPxRigidStaticHandle::new(actor))
                    .insert(BPxShapeHandle::new(shape));
            }

            BPxActor::Plane { normal, offset, material } => {
                let material = materials.get_mut(material).expect("material not found for BPxMaterial");
                let mut actor = physics
                    .create_plane(normal.to_physx(), *offset, material, ())
                    .unwrap();

                if velocity.is_some() {
                    bevy::log::warn!("ignoring BPxVelocity component from a static actor");
                }

                // unsafe raw function call is required to avoid consuming actor
                unsafe {
                    PxScene_addActor_mut(scene.as_mut_ptr(), actor.as_mut_ptr(), null());
                }

                commands.entity(entity)
                    .insert(BPxRigidStaticHandle::new(actor));
            }
        }
    }
}

pub fn writeback_actors(
    global_transforms: Query<&GlobalTransform>,
    mut actors: Query<(&BPxRigidDynamicHandle, Option<&Parent>, Option<&mut Transform>, Option<&mut BPxVelocity>)>
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
