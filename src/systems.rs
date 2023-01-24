use std::collections::HashMap;
use std::ptr::{null, null_mut};
use bevy::prelude::*;
use physx::prelude::*;
use physx::scene::Scene;
use physx::traits::Class;
use physx_sys::{
    PxScene_addActor_mut, PxRigidBodyExt_updateMassAndInertia_mut_1, PxShape_setLocalPose_mut,
    PxVehicleWheelsSimData_allocate_mut, phys_PxVehicleComputeSprungMasses, PxVehicleNoDrive_allocate_mut,
    PxVehicleNoDrive_setup_mut, PxVehicleWheelsSimData_setWheelShapeMapping_mut,
    PxVehicleWheelsSimData_setTireForceAppPointOffset_mut, PxVehicleWheelsSimData_setSuspForceAppPointOffset_mut,
    PxVehicleWheelsSimData_setWheelCentreOffset_mut, PxVehicleWheelsSimData_setSuspTravelDirection_mut,
    PxVehicleWheelsSimData_setSuspensionData_mut, PxVehicleWheelsSimData_setTireData_mut,
    PxVehicleWheelsSimData_setWheelData_mut, PxRigidBodyExt_setMassAndUpdateInertia_mut_1, PxScene_getGravity,
    PxVehicleWheels, phys_PxVehicleUpdates, phys_PxVehicleSuspensionRaycasts, PxVehicleNoDrive_setDriveTorque_mut, PxShape_getLocalPose, PxShape_setQueryFilterData_mut, PxFilterData, PxShape_setSimulationFilterData_mut, PxVehicleNoDrive, PxVehicleNoDrive_setBrakeTorque_mut, PxVehicleNoDrive_setSteerAngle_mut,
};

use super::{prelude::*, PxRigidDynamic, PxRigidStatic};
use super::assets::{BPxGeometry, BPxMaterial};
use super::components::{
    BPxActor, BPxMassProperties, BPxRigidDynamicHandle, BPxRigidStaticHandle, BPxShape, BPxShapeHandle,
    BPxVehicleNoDrive, BPxVehicleWheel, BPxVelocity, BPxVehicleHandle
};
use super::resources::{BPxDefaultMaterial, BPxPhysics, BPxScene, BPxTimeSync, BPxVehicleRaycastBuffer, BPxVehicleFrictionPairs};

type ActorsQuery<'world, 'state, 'a> = Query<'world, 'state,
    (Entity, &'a BPxActor, &'a GlobalTransform, Option<&'a BPxMassProperties>, Option<&'a BPxVehicleNoDrive>, Option<&'a BPxVelocity>),
    (Without<BPxRigidDynamicHandle>, Without<BPxRigidStaticHandle>)
>;

type ShapesQuery<'world, 'state, 'a> = Query<'world, 'state,
    (Entity, Option<&'a BPxActor>, Option<&'a Children>, Option<&'a BPxShape>, Option<&'a BPxVehicleWheel>, Option<&'a GlobalTransform>),
    (Without<BPxShapeHandle>, Without<BPxRigidDynamicHandle>, Without<BPxRigidStaticHandle>)
>;

pub fn scene_simulate(
    time: Res<Time>,
    mut scene: ResMut<BPxScene>,
    mut timesync: ResMut<BPxTimeSync>,
    mut raycastbuf: ResMut<BPxVehicleRaycastBuffer>,
    friction_pairs: Res<BPxVehicleFrictionPairs>,
    mut vehicles: Query<&mut BPxVehicleHandle>,
) {
    timesync.advance_bevy_time(&time);

    if let Some(delta) = timesync.check_advance_physx_time() {
        let mut wheel_count = 0;
        let mut vehicles = vehicles.iter_mut().map(|v| {
            wheel_count += v.wheels;
            v.ptr
        }).collect::<Vec<_>>();

        if !vehicles.is_empty() {
            raycastbuf.alloc(&mut scene, wheel_count);

            let gravity = unsafe { PxScene_getGravity(scene.as_ptr()) };

            unsafe {
                phys_PxVehicleSuspensionRaycasts(
                    raycastbuf.get_batch_query(),
                    vehicles.len() as u32,
                    vehicles.as_mut_ptr() as *mut *mut PxVehicleWheels,
                    wheel_count as u32,
                    raycastbuf.get_query_results(),
                    vec![true; vehicles.len()].as_ptr(),
                );

                phys_PxVehicleUpdates(
                    delta,
                    gravity.as_ptr(),
                    **friction_pairs,
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
    result: &mut Vec<(Entity, BPxShape, Option<BPxVehicleWheel>, Option<GlobalTransform>)>,
    level: u32,
) {
    if let Ok((entity, bpactor, children, shape_cfg, wheel_cfg, gtransform)) = query.get(entity) {
        // if we find BPxActor which is not the current one (level > 0), don't add its shapes
        if level > 0 && bpactor.is_some() { return; }

        if let Some(shape_cfg) = shape_cfg {
            result.push((entity, shape_cfg.clone(), wheel_cfg.cloned(), gtransform.copied()));
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
    geometries: &mut ResMut<Assets<BPxGeometry>>,
    materials: &mut ResMut<Assets<BPxMaterial>>,
    query: &ShapesQuery,
    actor_transform: &GlobalTransform,
    default_material: &mut ResMut<BPxDefaultMaterial>,
) {
    let mut found_shapes = vec![];
    find_nested_shapes(entity, query, &mut found_shapes, 0);

    for (entity, shape_cfg, _, gtransform) in found_shapes {
        let BPxShape { geometry, material, query_filter_data, simulation_filter_data } = shape_cfg;
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
        let mut shape_handle = BPxShapeHandle::create_shape(physics, geometry, material, entity);

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
    mut physics: ResMut<BPxPhysics>,
    mut scene: ResMut<BPxScene>,
    query: ShapesQuery,
    new_actors: ActorsQuery,
    mut geometries: ResMut<Assets<BPxGeometry>>,
    mut materials: ResMut<Assets<BPxMaterial>>,
    mut default_material: ResMut<BPxDefaultMaterial>,
) {
    for (entity, actor_cfg, actor_transform, mass_props, vehicle_cfg, velocity) in new_actors.iter() {
        match actor_cfg {
            BPxActor::Dynamic => {
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

                if let Some(vehicle_cfg) = vehicle_cfg {
                    let center_of_mass = match mass_props {
                        Some(BPxMassProperties::Density { density: _, center }) => *center,
                        Some(BPxMassProperties::Mass { mass: _, center }) => *center,
                        None => Vec3::ZERO,
                    };

                    let mut shape_mapping = HashMap::new();
                    for (idx, shape) in actor.get_shapes().into_iter().enumerate() {
                        shape_mapping.insert(*shape.get_user_data(), idx);
                    }

                    let wheels = vehicle_cfg.get_wheels().iter().map(|wheel| {
                        let (_, _, _, _, wheel_cfg, gtransform) = query.get(*wheel).ok()?;
                        let relative_transform = gtransform.map(|gtransform| {
                            let xform = actor_transform.affine().inverse() * gtransform.affine();
                            Transform::from_matrix(xform.into())
                        }).unwrap_or_default();

                        Some((shape_mapping.get(wheel)?, wheel_cfg?, relative_transform))
                    }).collect::<Vec<_>>();

                    // this is needed to correctly calculate wheel offsets in the very first frame
                    actor.set_c_mass_local_pose(&Transform::from_translation(center_of_mass).to_physx());

                    let wheel_count = wheels.len() as u32;
                    let wheel_sim_data = unsafe { PxVehicleWheelsSimData_allocate_mut(wheel_count).as_mut().unwrap() };
                    let mut suspension_spring_masses = vec![0f32; wheel_count as usize];

                    unsafe {
                        let wheel_offsets = wheels.iter().map(|w| {
                            w.and_then(|w| Some(w.2.translation.to_physx_sys())).unwrap_or_else(|| Vec3::ZERO.to_physx_sys())
                        }).collect::<Vec<_>>();

                        let gravity = PxScene_getGravity(scene.as_ptr()).to_bevy().abs();
                        let max_gravity_element = gravity.max_element();

                        phys_PxVehicleComputeSprungMasses(
                            wheel_count,
                            wheel_offsets.as_ptr(),
                            center_of_mass.to_physx_sys().as_ptr(),
                            actor.get_mass(),
                            if max_gravity_element == gravity.x {
                                0 // X
                            } else if max_gravity_element == gravity.y {
                                1 // Y
                            } else {
                                2 // Z
                            },
                            suspension_spring_masses.as_mut_ptr()
                        );
                    }

                    for (wheel_idx, wheel_params) in wheels.into_iter().enumerate() {
                        let Some((shape_idx, wheel_cfg, transform)) = wheel_params else { continue; };
                        let wheel_idx = wheel_idx as u32;

                        wheel_cfg.wheel_data.to_physx();
                        wheel_cfg.tire_data.to_physx();

                        unsafe {
                            PxVehicleWheelsSimData_setWheelData_mut(
                                wheel_sim_data, wheel_idx,
                                &wheel_cfg.wheel_data.to_physx() as *const _
                            );

                            PxVehicleWheelsSimData_setTireData_mut(
                                wheel_sim_data, wheel_idx,
                                &wheel_cfg.tire_data.to_physx() as *const _
                            );

                            let mut suspension_data = wheel_cfg.suspension_data.to_physx();
                            suspension_data.mSprungMass = suspension_spring_masses[wheel_idx as usize];

                            PxVehicleWheelsSimData_setSuspensionData_mut(
                                wheel_sim_data, wheel_idx,
                                &suspension_data as *const _
                            );

                            PxVehicleWheelsSimData_setSuspTravelDirection_mut(
                                wheel_sim_data,
                                wheel_idx,
                                wheel_cfg.susp_travel_direction.to_physx_sys().as_ptr()
                            );

                            PxVehicleWheelsSimData_setWheelCentreOffset_mut(
                                wheel_sim_data, wheel_idx,
                                (transform.translation - center_of_mass).to_physx_sys().as_ptr()
                            );

                            PxVehicleWheelsSimData_setSuspForceAppPointOffset_mut(
                                wheel_sim_data, wheel_idx,
                                wheel_cfg.susp_force_app_point_offset.to_physx_sys().as_ptr()
                            );

                            PxVehicleWheelsSimData_setTireForceAppPointOffset_mut(
                                wheel_sim_data, wheel_idx,
                                wheel_cfg.tire_force_app_point_offset.to_physx_sys().as_ptr()
                            );

                            //PxVehicleWheelsSimData_setSceneQueryFilterData_mut(wheel_sim_data, wheel_idx, sq_filter_data);
                            PxVehicleWheelsSimData_setWheelShapeMapping_mut(wheel_sim_data, wheel_idx, *shape_idx as i32);
                        }
                    }

                    let vehicle = unsafe { PxVehicleNoDrive_allocate_mut(wheel_count) };
                    unsafe { PxVehicleNoDrive_setup_mut(vehicle, physics.as_mut_ptr(), actor.as_mut_ptr(), wheel_sim_data); }

                    commands.entity(entity)
                        .insert(BPxVehicleHandle { ptr: vehicle as *mut _, wheels: wheel_count as usize });
                }

                match mass_props {
                    Some(BPxMassProperties::Density { density, center }) => unsafe {
                        PxRigidBodyExt_updateMassAndInertia_mut_1(
                            actor.as_mut_ptr(),
                            *density,
                            center.to_physx_sys().as_ptr(),
                            false
                        );
                    }
                    Some(BPxMassProperties::Mass { mass, center }) => unsafe {
                        PxRigidBodyExt_setMassAndUpdateInertia_mut_1(
                            actor.as_mut_ptr(),
                            *mass,
                            center.to_physx_sys().as_ptr(),
                            false
                        );
                    }
                    None => {}
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
                    .insert(BPxRigidStaticHandle::new(actor));
            }
        }
    }
}

pub fn writeback_actors(
    global_transforms: Query<&GlobalTransform>,
    parents: Query<&Parent>,
    mut writeback_transform: Query<&mut Transform>,
    mut actors: Query<(Entity, &BPxRigidDynamicHandle, Option<&Parent>, Option<&mut BPxVelocity>)>
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
            let newvel = BPxVelocity::new(
                actor.get_linear_velocity().to_bevy(),
                actor.get_angular_velocity().to_bevy(),
            );

            // avoid triggering bevy's change tracking if no change
            if newvel != *velocity { *velocity = newvel; }
        }
    }
}

pub fn vehicle_no_drive_update(
    mut query: Query<(&BPxVehicleNoDrive, &mut BPxVehicleHandle), Or<(Changed<BPxVehicleNoDrive>, Added<BPxVehicleHandle>)>>
) {
    for (vehicle, handle) in query.iter_mut() {
        for (idx, _entity) in vehicle.get_wheels().iter().enumerate() {
            unsafe {
                PxVehicleNoDrive_setDriveTorque_mut(handle.ptr as *mut PxVehicleNoDrive, idx as u32, vehicle.get_drive_torque(idx));
                PxVehicleNoDrive_setBrakeTorque_mut(handle.ptr as *mut PxVehicleNoDrive, idx as u32, vehicle.get_brake_torque(idx));
                PxVehicleNoDrive_setSteerAngle_mut(handle.ptr as *mut PxVehicleNoDrive, idx as u32, vehicle.get_steer_angle(idx));
            }
        }
    }
}
