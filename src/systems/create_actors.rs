use std::collections::HashMap;
use std::ptr::{null, null_mut};

use bevy::prelude::*;
use physx::prelude::*;
use physx::traits::Class;
use physx_sys::{
    PxScene_addActor_mut,
    PxShape_setLocalPose_mut,
};

use crate::components::{
    ArticulationJoint,
    ArticulationLinkHandle,
    RigidDynamicHandle,
    RigidStaticHandle,
    ShapeHandle,
};
use crate::prelude::{self as bpx, *};
use crate::resources::DefaultMaterial;
use crate::types::*;

type ActorsQuery<'world, 'state, 'a> = Query<'world, 'state,
    (Entity, &'a bpx::RigidBody, &'a GlobalTransform, Option<&'a ArticulationJoint>),
    (Without<RigidDynamicHandle>, Without<RigidStaticHandle>, Without<ArticulationLinkHandle>)
>;

type ShapesQuery<'world, 'state, 'a> = Query<'world, 'state,
    (Entity, Option<&'a bpx::RigidBody>, Option<&'a Children>, Option<&'a bpx::Shape>, Option<&'a GlobalTransform>),
    (Without<ShapeHandle>, Without<RigidDynamicHandle>, Without<RigidStaticHandle>)
>;

fn find_and_attach_nested_shapes<T: RigidActor<Shape = PxShape>>(
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
    #[allow(unused)]
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
        bevy::log::warn!("articulations are not supported with PhysX 4");
    }
}
