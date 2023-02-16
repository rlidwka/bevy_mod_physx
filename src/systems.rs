use std::ptr::{null, null_mut};
use bevy::prelude::*;
use physx::prelude::*;
use physx::scene::Scene;
use physx::traits::Class;
use physx_sys::{
    PxScene_addActor_mut, PxRigidBodyExt_updateMassAndInertia_mut_1, PxShape_setLocalPose_mut,
    PxRigidBodyExt_setMassAndUpdateInertia_mut_1, PxScene_getGravity,
    PxVehicleWheels, phys_PxVehicleUpdates,
    PxShape_getLocalPose, PxShape_setQueryFilterData_mut, PxFilterData, PxShape_setSimulationFilterData_mut, phys_PxVehicleSuspensionSweeps,
};

use super::prelude as bpx;
use super::{prelude::*, PxRigidDynamic, PxRigidStatic};
use super::components::{RigidDynamicHandle, RigidStaticHandle};
use super::resources::{DefaultMaterial, VehicleSceneQueryData, VehicleFrictionPairs};

type ActorsQuery<'world, 'state, 'a> = Query<'world, 'state,
    (Entity, &'a bpx::RigidBody, &'a GlobalTransform, Option<&'a MassProperties>, Option<&'a Velocity>, Option<&'a mut Vehicle>),
    (Without<RigidDynamicHandle>, Without<RigidStaticHandle>, Without<VehicleHandle>)
>;

type ShapesQuery<'world, 'state, 'a> = Query<'world, 'state,
    (Entity, Option<&'a bpx::RigidBody>, Option<&'a Children>, Option<&'a bpx::Shape>, Option<&'a GlobalTransform>),
    (Without<ShapeHandle>, Without<RigidDynamicHandle>, Without<RigidStaticHandle>)
>;

pub fn scene_simulate(
    mut scene: ResMut<bpx::Scene>,
    simtime: Res<SimTime>,
    mut scene_query: ResMut<VehicleSceneQueryData>,
    friction_pairs: Res<VehicleFrictionPairs>,
    mut vehicles_query: Query<&mut VehicleHandle>,
) {
    for delta in simtime.ticks() {
        let mut wheel_count = 0;
        let mut vehicles: Vec<*mut PxVehicleWheels> = vec![];

        for mut vehicle in vehicles_query.iter_mut() {
            match vehicle.as_mut() {
                VehicleHandle::NoDrive(vehicle) => {
                    wheel_count += vehicle.wheels_sim_data().get_nb_wheels() as usize;
                    vehicles.push(vehicle.as_mut_ptr());
                }
                VehicleHandle::Drive4W(vehicle) => {
                    wheel_count += vehicle.wheels_sim_data().get_nb_wheels() as usize;
                    vehicles.push(vehicle.as_mut_ptr());
                }
                VehicleHandle::DriveNW(vehicle) => {
                    wheel_count += vehicle.wheels_sim_data().get_nb_wheels() as usize;
                    vehicles.push(vehicle.as_mut_ptr());
                }
                VehicleHandle::DriveTank(vehicle) => {
                    wheel_count += vehicle.wheels_sim_data().get_nb_wheels() as usize;
                    vehicles.push(vehicle.as_mut_ptr());
                }
            }
        }

        if !vehicles.is_empty() {
            scene_query.alloc(&mut scene, wheel_count);

            let gravity = unsafe { PxScene_getGravity(scene.as_ptr()) };

            unsafe {
                /*phys_PxVehicleSuspensionRaycasts(
                    scene_query.get_batch_query(),
                    vehicles.len() as u32,
                    vehicles.as_mut_ptr() as *mut *mut PxVehicleWheels,
                    wheel_count as u32,
                    scene_query.get_raycast_query_buffer(),
                    vec![true; vehicles.len()].as_ptr(),
                );*/

                phys_PxVehicleSuspensionSweeps(
                    scene_query.get_batch_query(),
                    vehicles.len() as u32,
                    vehicles.as_mut_ptr() as *mut *mut PxVehicleWheels,
                    wheel_count as u32,
                    scene_query.get_sweep_query_buffer(),
                    1,
                    null_mut(),
                    1.,
                    1.01,
                );

                phys_PxVehicleUpdates(
                    delta,
                    gravity.as_ptr(),
                    friction_pairs.as_ptr(),
                    vehicles.len() as u32,
                    vehicles.as_mut_ptr() as *mut *mut PxVehicleWheels,
                    null_mut(),
                    null_mut(),
                );
            }
        }

        scene.simulate(delta, None, None);
        scene.fetch_results(true).unwrap();
    }
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
    default_material: &mut ResMut<DefaultMaterial>,
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
            // fetch default material if it exists, create if it doesn't
            if default_material.is_none() {
                let material = materials.add(physics.create_material(0.5, 0.5, 0.6, ()).unwrap().into());
                ***default_material = Some(material);
            }

            material = materials.get_mut(default_material.as_ref().as_ref().unwrap());
        }

        let material = material.unwrap(); // we create default material above, so we guarantee it exists
        let mut shape_handle = ShapeHandle::create_shape(physics, geometry, material, entity);

        unsafe {
            PxShape_setLocalPose_mut(
                shape_handle.as_mut_ptr(),
                relative_transform.to_physx().as_ptr(),
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
    mut default_material: ResMut<DefaultMaterial>,
) {
    for (entity, actor_cfg, actor_transform, mass_props, velocity, vehicle) in new_actors.iter_mut() {
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
                    &mut default_material,
                );

                match mass_props {
                    Some(MassProperties::Density { density, center }) => unsafe {
                        PxRigidBodyExt_updateMassAndInertia_mut_1(
                            actor.as_mut_ptr(),
                            *density,
                            center.to_physx_sys().as_ptr(),
                            false
                        );
                    }
                    Some(MassProperties::Mass { mass, center }) => unsafe {
                        PxRigidBodyExt_setMassAndUpdateInertia_mut_1(
                            actor.as_mut_ptr(),
                            *mass,
                            center.to_physx_sys().as_ptr(),
                            false
                        );
                    }
                    None => {}
                }

                if let Some(mut vehicle) = vehicle {
                    commands.entity(entity)
                        .insert(VehicleHandle::new(&mut vehicle, &mut physics, &mut actor));
                }

                if let Some(Velocity { linvel, angvel }) = velocity {
                    actor.set_linear_velocity(&linvel.to_physx(), false);
                    actor.set_angular_velocity(&angvel.to_physx(), false);
                }

                // unsafe raw function call is required to avoid consuming actor
                unsafe {
                    PxScene_addActor_mut(scene.as_mut_ptr(), actor.as_mut_ptr(), null());
                }

                commands.entity(entity)
                    .insert(RigidDynamicHandle::new(actor));
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
                    &mut default_material,
                );

                if mass_props.is_some() {
                    bevy::log::warn!("ignoring BPxMassProperties component from a static actor");
                }

                if velocity.is_some() {
                    bevy::log::warn!("ignoring BPxVelocity component from a static actor");
                }

                // unsafe raw function call is required to avoid consuming actor
                unsafe {
                    PxScene_addActor_mut(scene.as_mut_ptr(), actor.as_mut_ptr(), null());
                }

                commands.entity(entity)
                    .insert(RigidStaticHandle::new(actor));
            }
        }
    }
}

pub fn writeback_actors(
    global_transforms: Query<&GlobalTransform>,
    parents: Query<&Parent>,
    mut writeback_transform: Query<&mut Transform>,
    mut actors: Query<(Entity, &RigidDynamicHandle, Option<&Parent>, Option<&mut Velocity>)>
) {
    for (actor_entity, actor, parent, velocity) in actors.iter_mut() {
        let xform = actor.get_global_pose();
        let mut actor_xform = xform.to_bevy();

        if let Some(parent_transform) = parent.and_then(|p| global_transforms.get(**p).ok()) {
            let (_scale, inv_rotation, inv_translation) =
                parent_transform.affine().inverse().to_scale_rotation_translation();

            actor_xform.rotation = inv_rotation * actor_xform.rotation;
            actor_xform.translation = inv_rotation * actor_xform.translation + inv_translation;
        }

        if let Ok(mut transform) = writeback_transform.get_mut(actor_entity) {
            // avoid triggering bevy's change tracking if no change
            if actor_xform != *transform { *transform = actor_xform; }
        }

        // this is actor transform from the previous frame
        let actor_xform = Transform::from(global_transforms.get(actor_entity).copied().unwrap_or(GlobalTransform::IDENTITY));

        for shape in actor.get_shapes() {
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

        if let Some(mut velocity) = velocity {
            let newvel = Velocity::new(
                actor.get_linear_velocity().to_bevy(),
                actor.get_angular_velocity().to_bevy(),
            );

            // avoid triggering bevy's change tracking if no change
            if newvel != *velocity { *velocity = newvel; }
        }
    }
}
