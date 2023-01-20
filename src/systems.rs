use std::ptr::null;
use bevy::prelude::*;
use physx::prelude::*;
use physx::scene::Scene;
use physx::traits::Class;
use physx_sys::{PxScene_addActor_mut, PxRigidBodyExt_updateMassAndInertia_mut_1, PxShape_setLocalPose_mut};

use crate::resources::BPxDefaultMaterial;

use super::{prelude::*, PxRigidDynamic, PxRigidStatic};
use super::assets::{BPxGeometry, BPxMaterial};
use super::components::{BPxActor, BPxRigidDynamicHandle, BPxRigidStaticHandle, BPxShape, BPxShapeHandle, BPxVelocity};
use super::resources::{BPxScene, BPxPhysics, BPxTimeSync};

pub fn scene_simulate(time: Res<Time>, mut scene: ResMut<BPxScene>, mut timesync: ResMut<BPxTimeSync>) {
    timesync.advance_bevy_time(&time);

    if let Some(delta) = timesync.check_advance_physx_time() {
        scene.simulate(delta, None, None);
        scene.fetch_results(true).unwrap();
    }
}

fn find_nested_shapes(
    entity: Entity,
    query: &Query<
        (Entity, Option<&BPxActor>, Option<&Children>, Option<&BPxShape>, Option<&GlobalTransform>),
        (Without<BPxShapeHandle>, Without<BPxRigidDynamicHandle>, Without<BPxRigidStaticHandle>)
    >,
    result: &mut Vec<(Entity, BPxShape, Option<GlobalTransform>)>,
    level: u32,
) {
    if let Ok((entity, bpactor, children, shape_cfg, gtransform)) = query.get(entity) {
        // if we find BPxActor which is not the current one (level > 0), don't add its shapes
        if level > 0 && bpactor.is_some() { return; }

        if let Some(shape_cfg) = shape_cfg {
            result.push((entity, shape_cfg.clone(), gtransform.copied()));
        }

        if let Some(children) = children {
            for child in children.iter().copied() {
                find_nested_shapes(child, query, result, level + 1);
            }
        }
    }
}

fn find_and_attach_nested_shapes<T: RigidActor<Shape = crate::PxShape>>(
    commands: &mut Commands,
    entity: Entity,
    actor: &mut T,
    physics: &mut BPxPhysics,
    geometries: &Res<Assets<BPxGeometry>>,
    materials: &mut ResMut<Assets<BPxMaterial>>,
    query: &Query<
        (Entity, Option<&BPxActor>, Option<&Children>, Option<&BPxShape>, Option<&GlobalTransform>),
        (Without<BPxShapeHandle>, Without<BPxRigidDynamicHandle>, Without<BPxRigidStaticHandle>)
    >,
    actor_transform: &GlobalTransform,
    default_material: &mut ResMut<BPxDefaultMaterial>,
)
{
    let mut found_shapes = vec![];
    find_nested_shapes(entity, query, &mut found_shapes, 0);

    for (entity, shape_cfg, gtransform) in found_shapes {
        let BPxShape { geometry, material } = shape_cfg;
        let geometry = geometries.get(&geometry).expect("geometry not found for BPxGeometry");
        let mut material = materials.get_mut(&material);

        if material.is_none() {
            // fetch default material if it exists, create if it doesn't
            if default_material.is_none() {
                let material = materials.add(physics.create_material(0.5, 0.5, 0.6, ()).unwrap().into());
                ***default_material = Some(material);
            }

            material = materials.get_mut(&default_material.as_ref().as_ref().unwrap());
        }

        let material = material.unwrap(); // we create default material above, so we guarantee it exists
        let mut shape_handle = BPxShapeHandle::create_shape(physics, geometry, material);

        if let Some(gtransform) = gtransform {
            let relative_transform = actor_transform.affine().inverse() * gtransform.affine();

            unsafe {
                PxShape_setLocalPose_mut(
                    shape_handle.as_mut_ptr(),
                    Transform::from_matrix(relative_transform.into()).to_physx().as_ptr(),
                );
            }
        }

        actor.attach_shape(&mut shape_handle);

        commands.entity(entity)
            .insert(shape_handle);
    }
}

pub fn create_dynamic_actors(
    mut commands: Commands,
    mut physics: ResMut<BPxPhysics>,
    mut scene: ResMut<BPxScene>,
    query: Query<
        (Entity, Option<&BPxActor>, Option<&Children>, Option<&BPxShape>, Option<&GlobalTransform>),
        (Without<BPxShapeHandle>, Without<BPxRigidDynamicHandle>, Without<BPxRigidStaticHandle>)
    >,
    new_actors: Query<
        (Entity, &BPxActor, &GlobalTransform, Option<&BPxVelocity>),
        (Without<BPxRigidDynamicHandle>, Without<BPxRigidStaticHandle>)
    >,
    geometries: Res<Assets<BPxGeometry>>,
    mut materials: ResMut<Assets<BPxMaterial>>,
    mut default_material: ResMut<BPxDefaultMaterial>,
) {
    for (entity, actor_cfg, transform, velocity) in new_actors.iter() {
        match actor_cfg {
            BPxActor::Dynamic { density } => {
                let mut actor : Owner<PxRigidDynamic> = physics.create_dynamic(&transform.to_physx(), ()).unwrap();

                find_and_attach_nested_shapes(
                    &mut commands,
                    entity,
                    actor.as_mut(),
                    physics.as_mut(),
                    &geometries,
                    &mut materials,
                    &query,
                    &transform,
                    &mut default_material,
                );

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
                    .insert(BPxRigidDynamicHandle::new(actor));
            }

            BPxActor::Static => {
                let mut actor : Owner<PxRigidStatic> = physics.create_static(transform.to_physx(), ()).unwrap();

                find_and_attach_nested_shapes(
                    &mut commands,
                    entity,
                    actor.as_mut(),
                    physics.as_mut(),
                    &geometries,
                    &mut materials,
                    &query,
                    &transform,
                    &mut default_material,
                );

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

            BPxActor::StaticPlane { normal, offset, material } => {
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
