use std::collections::HashMap;
use std::ptr::{null, null_mut};
use bevy::prelude::*;
use physx::prelude::*;
use physx::scene::Scene;
use physx::traits::Class;
use physx_sys::{
    PxFilterData,
    PxArticulationJointReducedCoordinate_setChildPose_mut,
    PxArticulationJointReducedCoordinate_setFrictionCoefficient_mut,
    PxArticulationJointReducedCoordinate_setMaxJointVelocity_mut,
    PxArticulationJointReducedCoordinate_setParentPose_mut,
    PxArticulationLink_getInboundJoint,
    PxArticulationReducedCoordinate_createLink_mut,
    PxScene_addActor_mut,
    PxScene_addArticulation_mut,
    PxShape_getLocalPose,
    PxShape_setLocalPose_mut,
    PxShape_setQueryFilterData_mut,
    PxShape_setSimulationFilterData_mut,
};

use crate::components::ArticulationJoint;

use super::prelude as bpx;
use super::{prelude::*, PxArticulationReducedCoordinate, PxRigidDynamic, PxRigidStatic};
use super::components::{ArticulationLinkHandle, ArticulationRootHandle, RigidDynamicHandle, RigidStaticHandle};
use super::resources::DefaultMaterial;

type ActorsQuery<'world, 'state, 'a> = Query<'world, 'state,
    (Entity, &'a bpx::RigidBody, &'a GlobalTransform, Option<&'a ArticulationJoint>),
    (Without<RigidDynamicHandle>, Without<RigidStaticHandle>, Without<ArticulationLinkHandle>)
>;

type ShapesQuery<'world, 'state, 'a> = Query<'world, 'state,
    (Entity, Option<&'a bpx::RigidBody>, Option<&'a Children>, Option<&'a bpx::Shape>, Option<&'a GlobalTransform>),
    (Without<ShapeHandle>, Without<RigidDynamicHandle>, Without<RigidStaticHandle>)
>;

pub fn scene_simulate(
    mut scene: ResMut<bpx::Scene>,
    time: Res<PhysicsTime>,
) {
    let mut scene = scene.get_mut();
    scene.simulate(time.delta_seconds, None, None);
    scene.fetch_results(true).unwrap();
}

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
        let bpx::Shape { geometry, material, query_filter_data, simulation_filter_data } = shape_cfg;
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
        let mut shape_component = ShapeHandle::create_shape(physics, geometry, material, entity);
        let custom_transform = shape_component.custom_xform;
        // SAFETY: scene locking is done by the caller
        let shape_handle = unsafe { shape_component.get_mut_unsafe() };

        unsafe {
            PxShape_setLocalPose_mut(
                shape_handle.as_mut_ptr(),
                (relative_transform * custom_transform).to_physx().as_ptr(),
            );

            if query_filter_data != default() {
                let pxfilterdata : PxFilterData = query_filter_data.into();
                PxShape_setQueryFilterData_mut(shape_handle.as_mut_ptr(), &pxfilterdata as *const _);
            }

            if simulation_filter_data != default() {
                let pxfilterdata : PxFilterData = simulation_filter_data.into();
                PxShape_setSimulationFilterData_mut(shape_handle.as_mut_ptr(), &pxfilterdata as *const _);
            }
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

pub fn sync_transform_dynamic(
    mut scene: ResMut<bpx::Scene>,
    global_transforms: Query<&GlobalTransform>,
    mut actors: Query<(&mut RigidDynamicHandle, &mut Transform, &GlobalTransform, Option<&Parent>)>,
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
