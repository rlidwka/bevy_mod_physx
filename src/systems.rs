use std::ptr::null;
use bevy::prelude::*;
use physx::prelude::*;
use physx::scene::Scene;
use physx::traits::Class;
use physx_sys::{
    PxFilterData,
    PxRigidBodyExt_setMassAndUpdateInertia_1,
    PxRigidBodyExt_updateMassAndInertia_1,
    PxScene_addActor_mut,
    PxShape_getLocalPose,
    PxShape_setLocalPose_mut,
    PxShape_setQueryFilterData_mut,
    PxShape_setSimulationFilterData_mut,
};

use super::prelude as bpx;
use super::{prelude::*, PxRigidDynamic, PxRigidStatic};
use super::components::{RigidDynamicHandle, RigidStaticHandle};
use super::resources::DefaultMaterial;

type ActorsQuery<'world, 'state, 'a> = Query<'world, 'state,
    (Entity, &'a bpx::RigidBody, &'a GlobalTransform, Option<&'a MassProperties>),
    (Without<RigidDynamicHandle>, Without<RigidStaticHandle>)
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

fn find_nested_shapes(
    entity: Entity,
    query: &ShapesQuery,
    result: &mut Vec<(Entity, bpx::Shape, Option<GlobalTransform>)>,
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
    physics: &mut bpx::Physics,
    geometries: &mut ResMut<Assets<bpx::Geometry>>,
    materials: &mut ResMut<Assets<bpx::Material>>,
    query: &ShapesQuery,
    actor_transform: &GlobalTransform,
    default_material: &Handle<bpx::Material>,
) {
    let mut found_shapes = vec![];
    find_nested_shapes(entity, query, &mut found_shapes, 0);

    for (entity, shape_cfg, gtransform) in found_shapes {
        let bpx::Shape { geometry, material, query_filter_data, simulation_filter_data } = shape_cfg;
        let geometry = geometries.get_mut(&geometry).expect("geometry not found for BPxGeometry");
        let mut material = materials.get_mut(&material);

        let relative_transform = gtransform.map(|gtransform| {
            let xform = actor_transform.affine().inverse() * gtransform.affine();
            Transform::from_matrix(xform.into())
        }).unwrap_or_default();

        if material.is_none() {
            material = materials.get_mut(default_material);
        }

        let material = material.expect("default material not found");
        let (mut shape_handle, transform) = ShapeHandle::create_shape(physics, geometry, material, entity);

        unsafe {
            PxShape_setLocalPose_mut(
                shape_handle.as_mut_ptr(),
                (relative_transform * transform).to_physx().as_ptr(),
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

        actor.attach_shape(&mut shape_handle);

        commands.entity(entity)
            .insert(shape_handle);
    }
}

pub fn create_dynamic_actors(
    mut commands: Commands,
    mut physics: ResMut<bpx::Physics>,
    mut scene: ResMut<bpx::Scene>,
    query: ShapesQuery,
    mut new_actors: ActorsQuery,
    mut geometries: ResMut<Assets<bpx::Geometry>>,
    mut materials: ResMut<Assets<bpx::Material>>,
    default_material: Res<DefaultMaterial>,
) {
    for (entity, actor_cfg, actor_transform, mass_props) in new_actors.iter_mut() {
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

                match mass_props {
                    Some(MassProperties::Density { density, center }) => unsafe {
                        PxRigidBodyExt_updateMassAndInertia_1(
                            actor.as_mut_ptr(),
                            *density,
                            center.to_physx_sys().as_ptr(),
                            false
                        );
                    }
                    Some(MassProperties::Mass { mass, center }) => unsafe {
                        PxRigidBodyExt_setMassAndUpdateInertia_1(
                            actor.as_mut_ptr(),
                            *mass,
                            center.to_physx_sys().as_ptr(),
                            false
                        );
                    }
                    None => {}
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

                if mass_props.is_some() {
                    bevy::log::warn!("ignoring BPxMassProperties component from a static actor");
                }

                // unsafe raw function call is required to avoid consuming actor
                unsafe {
                    PxScene_addActor_mut(scene.as_mut_ptr(), actor.as_mut_ptr(), null());
                }

                commands.entity(entity)
                    .insert(RigidStaticHandle::new(actor, *actor_transform));
            }
        }
    }
}

pub fn apply_user_changes(
    mut scene: ResMut<bpx::Scene>,
    mut changed_dynamic: Query<(&mut RigidDynamicHandle, &GlobalTransform), Changed<GlobalTransform>>,
    mut changed_static: Query<(&mut RigidStaticHandle, &GlobalTransform), Changed<GlobalTransform>>,
) {
    for (mut handle, xform) in changed_dynamic.iter_mut() {
        if xform != &handle.cached_transform {
            handle.cached_transform = *xform;
            handle.get_mut(&mut scene).set_global_pose(&xform.to_physx(), true);
        }
    }

    for (mut handle, xform) in changed_static.iter_mut() {
        if xform != &handle.cached_transform {
            handle.cached_transform = *xform;
            handle.get_mut(&mut scene).set_global_pose(&xform.to_physx(), true);
        }
    }
}

pub fn writeback_actors(
    scene: Res<bpx::Scene>,
    global_transforms: Query<&GlobalTransform>,
    parents: Query<&Parent>,
    mut writeback_transform: Query<&mut Transform>,
    mut actors: Query<(Entity, &mut RigidDynamicHandle, Option<&Parent>)>
) {
    for (actor_entity, mut actor, parent) in actors.iter_mut() {
        let actor_handle = actor.get(&scene);
        let xform = actor_handle.get_global_pose();
        let mut actor_xform = xform.to_bevy();

        let next_transform = if let Some(parent_transform) = parent.and_then(|p| global_transforms.get(**p).ok()) {
            let (_scale, inv_rotation, inv_translation) =
                parent_transform.affine().inverse().to_scale_rotation_translation();

            actor_xform.rotation = inv_rotation * actor_xform.rotation;
            actor_xform.translation = inv_rotation * actor_xform.translation + inv_translation;

            parent_transform.mul_transform(actor_xform)
        } else {
            actor_xform.into()
        };

        if let Ok(mut transform) = writeback_transform.get_mut(actor_entity) {
            // avoid triggering bevy's change tracking if no change
            if actor_xform != *transform { *transform = actor_xform; }
        }

        // this is actor transform from the previous frame
        let actor_xform = Transform::from(global_transforms.get(actor_entity).copied().unwrap_or(GlobalTransform::IDENTITY));

        for shape in actor_handle.get_shapes() {
            let shape_entity = *shape.get_user_data();
            if shape_entity == actor_entity {
                // we already updated actor entity above,
                // and in this case local transform will always be IDENTITY
                continue;
            }

            let shape_local_xform = unsafe { PxShape_getLocalPose(shape.as_ptr()) }.to_bevy();
            let mut shape_xform = actor_xform * shape_local_xform;

            if let Some(parent_transform) = parents.get(shape_entity).ok().and_then(|p| global_transforms.get(**p).ok()) {
                let (_scale, inv_rotation, inv_translation) =
                    parent_transform.affine().inverse().to_scale_rotation_translation();

                shape_xform.rotation = inv_rotation * shape_xform.rotation;
                shape_xform.translation = inv_rotation * shape_xform.translation + inv_translation;
            }

            if let Ok(mut transform) = writeback_transform.get_mut(shape_entity) {
                // avoid triggering bevy's change tracking if no change
                if shape_xform != *transform { *transform = shape_xform; }
            }
        }

        drop(actor_handle);
        actor.cached_transform = next_transform;
    }
}
