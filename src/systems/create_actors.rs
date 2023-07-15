use bevy::prelude::*;
use physx::prelude::*;
use physx::traits::Class;
use physx_sys::{
    PxArticulationJointReducedCoordinate_setChildPose_mut,
    PxArticulationJointReducedCoordinate_setFrictionCoefficient_mut,
    PxArticulationJointReducedCoordinate_setMaxJointVelocity_mut,
    PxArticulationJointReducedCoordinate_setParentPose_mut,
    PxArticulationLink_getInboundJoint,
    PxArticulationReducedCoordinate_createLink_mut,
    PxScene_addActor_mut,
    PxScene_addArticulation_mut,
    PxShape_setLocalPose_mut,
};
use std::collections::HashMap;
use std::ptr::{null, null_mut};

use crate::components::{ArticulationJoint, ShapeHandle};

use crate::prelude as bpx;
use crate::{prelude::*, PxArticulationReducedCoordinate, PxRigidDynamic, PxRigidStatic};
use crate::components::{ArticulationLinkHandle, ArticulationRootHandle, RigidDynamicHandle, RigidStaticHandle};
use crate::resources::DefaultMaterial;

type ActorsQuery<'world, 'state, 'a> = Query<'world, 'state,
    (Entity, &'a bpx::RigidBody, &'a GlobalTransform, Option<&'a ArticulationJoint>),
    (Without<RigidDynamicHandle>, Without<RigidStaticHandle>, Without<ArticulationLinkHandle>)
>;

type ShapesQuery<'world, 'state, 'a> = Query<'world, 'state,
    (Entity, Option<&'a bpx::RigidBody>, Option<&'a Children>, Option<&'a bpx::Shape>, Option<&'a GlobalTransform>),
    (Without<ShapeHandle>, Without<RigidDynamicHandle>, Without<RigidStaticHandle>)
>;

fn find_and_attach_nested_shapes<T: RigidActor<Shape = crate::PxShape>>(
    commands: &mut Commands,
    entity: Entity,
    actor: &mut T,
    physics: &mut bpx::Physics,
    geometries: &mut ResMut<Assets<bpx::Geometry>>,
    materials: &mut ResMut<Assets<bpx::Material>>,
    query: &ShapesQuery,
    actor_transform: &GlobalTransform,
    default_material: &Handle<bpx::Material>,
) {
    let mut found_shapes = vec![];

    let Ok((entity, _bpactor, children, shape_cfg, gtransform)) = query.get(entity) else { return; };

    if let Some(shape_cfg) = shape_cfg {
        // single shape, matches root
        found_shapes.push((entity, shape_cfg.clone(), gtransform.copied()));
    } else if let Some(children) = children {
        // children, possibly multiple shapes
        for child in children.iter().copied() {
            let Ok((_entity, bpactor, _children, shape_cfg, gtransform)) = query.get(child) else { continue; };

            // if we find Actor which is not the current one (level > 0), don't add its shapes
            if bpactor.is_some() { continue; }

            if let Some(shape_cfg) = shape_cfg {
                found_shapes.push((child, shape_cfg.clone(), gtransform.copied()));
            }
        }
    }

    for (entity, shape_cfg, gtransform) in found_shapes {
        let bpx::Shape {
            geometry,
            material,
            flags,
        } = shape_cfg;

        let geometry = geometries.get_mut(&geometry).expect("geometry not found");
        let mut material = materials.get_mut(&material);

        let relative_transform = gtransform.map(|gtransform| {
            let xform = actor_transform.affine().inverse() * gtransform.affine();
            Transform::from_matrix(xform.into())
        }).unwrap_or_default();

        if material.is_none() {
            material = materials.get_mut(default_material);
        }

        let material = material.expect("default material not found");
        let mut shape_component = ShapeHandle::create_shape(physics, geometry, material, flags, entity);
        let custom_transform = shape_component.custom_xform;
        // SAFETY: scene locking is done by the caller
        let shape_handle = unsafe { shape_component.get_mut_unsafe() };

        unsafe {
            PxShape_setLocalPose_mut(
                shape_handle.as_mut_ptr(),
                (relative_transform * custom_transform).to_physx().as_ptr(),
            );
        }

        actor.attach_shape(shape_handle);

        commands.entity(entity)
            .insert(shape_component);
    }
}

pub fn create_rigid_actors(
    mut commands: Commands,
    mut physics: ResMut<bpx::Physics>,
    mut scene: ResMut<bpx::Scene>,
    query: ShapesQuery,
    new_actors: ActorsQuery,
    mut geometries: ResMut<Assets<bpx::Geometry>>,
    mut materials: ResMut<Assets<bpx::Material>>,
    default_material: Res<DefaultMaterial>,
) {
    struct ArticulationTreeNode<'a> {
        entity: Entity,
        transform: GlobalTransform,
        ptr: *mut physx_sys::PxArticulationLink,
        joint: Option<&'a ArticulationJoint>,
        children: Vec<usize>,
    }

    let mut articulation_link_tree = vec![];
    let mut articulation_entity_mapping = HashMap::new();

    for (entity, actor_cfg, actor_transform, inbound_joint) in new_actors.iter() {
        let send_sleep_notifies = scene.send_sleep_notifies;
        let mut scene = scene.get_mut();

        match actor_cfg {
            bpx::RigidBody::Dynamic => {
                let mut actor : Owner<PxRigidDynamic> = physics.create_dynamic(&actor_transform.to_physx(), entity).unwrap();

                find_and_attach_nested_shapes(
                    &mut commands,
                    entity,
                    actor.as_mut(),
                    physics.as_mut(),
                    &mut geometries,
                    &mut materials,
                    &query,
                    actor_transform,
                    &default_material,
                );

                if send_sleep_notifies {
                    actor.set_actor_flag(ActorFlag::SendSleepNotifies, true);
                }

                // unsafe raw function call is required to avoid consuming actor
                unsafe {
                    PxScene_addActor_mut(scene.as_mut_ptr(), actor.as_mut_ptr(), null());
                }

                commands.entity(entity)
                    .insert(RigidDynamicHandle::new(actor, *actor_transform));
            }

            bpx::RigidBody::Static => {
                let mut actor : Owner<PxRigidStatic> = physics.create_static(actor_transform.to_physx(), entity).unwrap();

                find_and_attach_nested_shapes(
                    &mut commands,
                    entity,
                    actor.as_mut(),
                    physics.as_mut(),
                    &mut geometries,
                    &mut materials,
                    &query,
                    actor_transform,
                    &default_material,
                );

                // unsafe raw function call is required to avoid consuming actor
                unsafe {
                    PxScene_addActor_mut(scene.as_mut_ptr(), actor.as_mut_ptr(), null());
                }

                commands.entity(entity)
                    .insert(RigidStaticHandle::new(actor, *actor_transform));
            }

            bpx::RigidBody::ArticulationLink => {
                articulation_entity_mapping.insert(entity, articulation_link_tree.len());

                articulation_link_tree.push(ArticulationTreeNode {
                    entity,
                    transform: *actor_transform,
                    joint: inbound_joint,
                    ptr: null_mut(),
                    children: vec![],
                });
            }
        }
    }

    if !articulation_link_tree.is_empty() {
        // the code below reconstruct articulation tree in order to ensure proper initialization order,
        // so that every child will be created after its parent
        let mut bases = vec![];

        for idx in 0..articulation_link_tree.len() {
            let current = &articulation_link_tree[idx];

            if let Some(ArticulationJoint { parent, .. }) = current.joint {
                let Some(parent_id) = articulation_entity_mapping.get(parent).copied() else {
                    bevy::log::warn!("Broken articulation hierarchy: cannot find parent {:?} for {:?}.", parent, current.entity);
                    bevy::log::warn!("Note that all links in any one articulation must be added at the same time (in a single bevy tick).");
                    continue;
                };

                articulation_link_tree[parent_id].children.push(idx);
            } else {
                // articulation root
                bases.push(idx);
            }
        }

        fn traverse_dfs(current: usize, articulations: &[ArticulationTreeNode], indexes: &mut Vec<usize>) {
            indexes.push(current);
            for i in articulations[current].children.iter().copied() {
                traverse_dfs(i, articulations, indexes);
            }
        }

        for base_idx in bases {
            let mut indexes = vec![];
            let base_entity = articulation_link_tree[base_idx].entity;

            traverse_dfs(base_idx, &articulation_link_tree, &mut indexes);

            let mut articulation: Owner<PxArticulationReducedCoordinate> =
                physics.create_articulation_reduced_coordinate(base_entity).unwrap();

            for i in indexes {
                let parent = articulation_link_tree[i].joint.map(|joint| {
                    let idx = articulation_entity_mapping.get(&joint.parent).unwrap();
                    assert!(!articulation_link_tree[*idx].ptr.is_null());
                    articulation_link_tree[*idx].ptr
                });

                let link_transform = articulation_link_tree[i].transform;
                let link_entity = articulation_link_tree[i].entity;

                articulation_link_tree[i].ptr = unsafe {
                    PxArticulationReducedCoordinate_createLink_mut(
                        articulation.as_mut_ptr(),
                        parent.unwrap_or(null_mut()),
                        link_transform.to_physx_sys().as_ptr(),
                    )
                };

                let mut actor: Owner<crate::PxArticulationLink> = unsafe {
                    ArticulationLink::from_raw(articulation_link_tree[i].ptr, link_entity)
                }.unwrap();

                find_and_attach_nested_shapes(
                    &mut commands,
                    link_entity,
                    actor.as_mut(),
                    physics.as_mut(),
                    &mut geometries,
                    &mut materials,
                    &query,
                    &link_transform,
                    &default_material,
                );

                if let Some(joint_cfg) = articulation_link_tree[i].joint {
                    let joint = unsafe { PxArticulationLink_getInboundJoint(articulation_link_tree[i].ptr) };
                    assert!(!joint.is_null());

                    // SAFETY: ArticulationJointReducedCoordinate is repr(transparent) wrapper
                    let joint = unsafe { &mut *(joint as *mut ArticulationJointReducedCoordinate) };

                    joint.set_joint_type(joint_cfg.joint_type);

                    fn set_motion(joint: &mut ArticulationJointReducedCoordinate, axis: ArticulationAxis, value: ArticulationJointMotion) {
                        match value {
                            ArticulationJointMotion::Locked => { /* nothing to do, it's the default */ }
                            ArticulationJointMotion::Free => {
                                joint.set_motion(axis, ArticulationMotion::Free);
                            }
                            ArticulationJointMotion::Limited { min, max } => {
                                joint.set_motion(axis, ArticulationMotion::Limited);
                                joint.set_limit(axis, min, max);
                            }
                        }
                    }

                    set_motion(joint, ArticulationAxis::Twist, joint_cfg.motion_twist);
                    set_motion(joint, ArticulationAxis::Swing1, joint_cfg.motion_swing1);
                    set_motion(joint, ArticulationAxis::Swing2, joint_cfg.motion_swing2);
                    set_motion(joint, ArticulationAxis::X, joint_cfg.motion_x);
                    set_motion(joint, ArticulationAxis::Y, joint_cfg.motion_y);
                    set_motion(joint, ArticulationAxis::Z, joint_cfg.motion_z);

                    unsafe {
                        PxArticulationJointReducedCoordinate_setParentPose_mut(joint.as_mut_ptr(), joint_cfg.parent_pose.to_physx_sys().as_ptr());
                        PxArticulationJointReducedCoordinate_setChildPose_mut(joint.as_mut_ptr(), joint_cfg.child_pose.to_physx_sys().as_ptr());
                        PxArticulationJointReducedCoordinate_setMaxJointVelocity_mut(joint.as_mut_ptr(), joint_cfg.max_joint_velocity);
                        PxArticulationJointReducedCoordinate_setFrictionCoefficient_mut(joint.as_mut_ptr(), joint_cfg.friction_coefficient);
                    }
                }

                commands.entity(link_entity)
                    .insert(ArticulationLinkHandle::new(actor, link_transform));
            }

            // unsafe raw function call is required to avoid consuming articulation
            unsafe {
                PxScene_addArticulation_mut(scene.get_mut().as_mut_ptr(), articulation.as_mut_ptr());
            }

            commands.entity(base_entity)
                .insert(ArticulationRootHandle::new(articulation));
        }
    }
}
