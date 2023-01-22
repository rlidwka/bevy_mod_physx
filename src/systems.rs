use std::ptr::null;
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
    PxVehicleWheelsSimData_setWheelData_mut, PxRigidBodyExt_setMassAndUpdateInertia_mut_1
};


use super::{prelude::*, PxRigidDynamic, PxRigidStatic};
use super::assets::{BPxGeometry, BPxMaterial};
use super::components::{
    BPxActor, BPxMassProperties, BPxRigidDynamicHandle, BPxRigidStaticHandle, BPxShape, BPxShapeHandle,
    BPxVehicle, BPxVehicleWheel, BPxVelocity
};
use super::resources::{BPxDefaultMaterial, BPxPhysics, BPxScene, BPxTimeSync};

type ActorsQuery<'world, 'state, 'a> = Query<'world, 'state,
    (Entity, &'a BPxActor, &'a GlobalTransform, Option<&'a BPxMassProperties>, Option<&'a BPxVehicle>, Option<&'a BPxVelocity>),
    (Without<BPxRigidDynamicHandle>, Without<BPxRigidStaticHandle>)
>;

type ShapesQuery<'world, 'state, 'a> = Query<'world, 'state,
    (Entity, Option<&'a BPxActor>, Option<&'a Children>, Option<&'a BPxShape>, Option<&'a BPxVehicleWheel>, Option<&'a GlobalTransform>),
    (Without<BPxShapeHandle>, Without<BPxRigidDynamicHandle>, Without<BPxRigidStaticHandle>)
>;

pub fn scene_simulate(time: Res<Time>, mut scene: ResMut<BPxScene>, mut timesync: ResMut<BPxTimeSync>) {
    timesync.advance_bevy_time(&time);

    if let Some(delta) = timesync.check_advance_physx_time() {
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
) -> Vec<(i32, BPxVehicleWheel, Transform)> {
    let mut found_shapes = vec![];
    find_nested_shapes(entity, query, &mut found_shapes, 0);

    let mut shape_index = 0;
    let mut wheels = vec![];

    for (entity, shape_cfg, wheel_cfg, gtransform) in found_shapes {
        let BPxShape { geometry, material } = shape_cfg;
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

            material = materials.get_mut(&default_material.as_ref().as_ref().unwrap());
        }

        let material = material.unwrap(); // we create default material above, so we guarantee it exists
        let mut shape_handle = BPxShapeHandle::create_shape(physics, geometry, material);

        unsafe {
            PxShape_setLocalPose_mut(
                shape_handle.as_mut_ptr(),
                relative_transform.to_physx().as_ptr(),
            );
        }

        shape_index += 1;
        actor.attach_shape(&mut shape_handle);

        if let Some(wheel_cfg) = wheel_cfg {
            wheels.push((shape_index - 1, wheel_cfg, relative_transform));
        }

        commands.entity(entity)
            .insert(shape_handle);
    }

    wheels
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
    for (entity, actor_cfg, transform, mass_props, vehicle_cfg, velocity) in new_actors.iter() {
        match actor_cfg {
            BPxActor::Dynamic => {
                let mut actor : Owner<PxRigidDynamic> = physics.create_dynamic(&transform.to_physx(), ()).unwrap();

                let wheels = find_and_attach_nested_shapes(
                    &mut commands,
                    entity,
                    actor.as_mut(),
                    physics.as_mut(),
                    &mut geometries,
                    &mut materials,
                    &query,
                    &transform,
                    &mut default_material,
                );

                if let Some(_) = vehicle_cfg {
                    let wheel_count = wheels.len() as u32;
                    let wheel_sim_data = unsafe { PxVehicleWheelsSimData_allocate_mut(wheel_count).as_mut().unwrap() };
                    let mut suspension_spring_masses = vec![0f32; wheel_count as usize];

                    unsafe {
                        let wheel_offsets = wheels.iter().map(|w| w.2.translation.to_physx_sys()).collect::<Vec<_>>();
                        phys_PxVehicleComputeSprungMasses(
                            wheel_count,
                            wheel_offsets.as_ptr(),
                            PxVec3::new(0., 0., 0.).as_ptr(),
                            actor.get_mass(),
                            1,
                            suspension_spring_masses.as_mut_ptr()
                        );
                    }

                    for (wheel_idx, (shape_idx, wheel_cfg, transform)) in wheels.into_iter().enumerate() {
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
                                transform.translation.to_physx_sys().as_ptr()
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
                            PxVehicleWheelsSimData_setWheelShapeMapping_mut(wheel_sim_data, wheel_idx, shape_idx);
                        }
                    }

                    let vehicle = unsafe { PxVehicleNoDrive_allocate_mut(wheel_count) };
                    unsafe { PxVehicleNoDrive_setup_mut(vehicle, physics.as_mut_ptr(), actor.as_mut_ptr(), wheel_sim_data); }
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
                let mut actor : Owner<PxRigidStatic> = physics.create_static(transform.to_physx(), ()).unwrap();

                find_and_attach_nested_shapes(
                    &mut commands,
                    entity,
                    actor.as_mut(),
                    physics.as_mut(),
                    &mut geometries,
                    &mut materials,
                    &query,
                    &transform,
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
