use crate::components::RigidDynamicHandle;
use crate::prelude::{Scene, *};
use bevy::prelude::*;
use physx::traits::Class;
use physx_sys::{PxRigidBodyExt_updateMassAndInertia_1, PxRigidBodyExt_setMassAndUpdateInertia_1};

#[derive(Component, Debug, Reflect, Clone, Copy)]
pub enum MassProperties {
    Density {
        density: f32,
        center: Vec3,
    },
    Mass {
        mass: f32,
        center: Vec3,
    },
}

impl MassProperties {
    pub fn density(density: f32) -> Self {
        Self::Density { density, center: Vec3::ZERO }
    }

    pub fn mass(mass: f32) -> Self {
        Self::Mass { mass, center: Vec3::ZERO }
    }

    pub fn density_with_center(density: f32, center: Vec3) -> Self {
        Self::Density { density, center }
    }

    pub fn mass_with_center(mass: f32, center: Vec3) -> Self {
        Self::Mass { mass, center }
    }
}

pub struct MassPropertiesPlugin;

impl Plugin for MassPropertiesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MassProperties>();
        app.add_system(
            mass_properties_sync
                .in_base_set(PhysicsSet::Sync)
                .in_schedule(PhysicsSchedule)
                .after(crate::systems::sync_transform_static)
                .after(crate::systems::sync_transform_dynamic)
                .after(crate::systems::sync_transform_nested_shapes),
        );
    }
}

pub fn mass_properties_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<(&mut RigidDynamicHandle, &MassProperties), Changed<MassProperties>>
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (mut actor, mass_props) in actors.iter_mut() {
        let mut actor_handle = actor.get_mut(&mut scene);

        match mass_props {
            MassProperties::Density { density, center } => unsafe {
                PxRigidBodyExt_updateMassAndInertia_1(
                    actor_handle.as_mut_ptr(),
                    *density,
                    center.to_physx_sys().as_ptr(),
                    false
                );
            }

            MassProperties::Mass { mass, center } => unsafe {
                PxRigidBodyExt_setMassAndUpdateInertia_1(
                    actor_handle.as_mut_ptr(),
                    *mass,
                    center.to_physx_sys().as_ptr(),
                    false
                );
            }
        }
    }
}
