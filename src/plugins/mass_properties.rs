use bevy::prelude::*;
use physx::traits::Class;
use physx_sys::{PxRigidBodyExt_setMassAndUpdateInertia_1, PxRigidBodyExt_updateMassAndInertia_1};

use crate::components::{ArticulationLinkHandle, RigidDynamicHandle};
use crate::prelude::{Scene, *};

#[derive(Component, Debug, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
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

impl Default for MassProperties {
    fn default() -> Self {
        Self::Density { density: 1., center: Vec3::ZERO }
    }
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
        app.add_systems(
            PhysicsSchedule,
            mass_properties_sync
                .in_set(PhysicsSet::Sync)
                .after(crate::systems::sync_transform_static)
                .after(crate::systems::sync_transform_dynamic)
                .after(crate::systems::sync_transform_nested_shapes),
        );
    }
}

pub fn mass_properties_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<
        (
            Option<&mut RigidDynamicHandle>,
            Option<&mut ArticulationLinkHandle>,
            Ref<MassProperties>,
        ),
        Or<(
            Added<RigidDynamicHandle>,
            Added<ArticulationLinkHandle>,
            Changed<MassProperties>,
        )>,
    >,
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (dynamic, articulation, mass_props) in actors.iter_mut() {
        let actor_handle = if let Some(mut actor) = dynamic {
            actor.get_mut(&mut scene).as_mut_ptr()
        } else if let Some(mut actor) = articulation {
            actor.get_mut(&mut scene).as_mut_ptr()
        } else {
            if !mass_props.is_added() {
                bevy::log::warn!("MassProperties component exists, but it's neither a rigid dynamic nor articulation link");
            }
            continue;
        };

        match *mass_props {
            MassProperties::Density { density, center } => unsafe {
                PxRigidBodyExt_updateMassAndInertia_1(
                    actor_handle,
                    density,
                    center.to_physx_sys().as_ptr(),
                    false
                );
            }

            MassProperties::Mass { mass, center } => unsafe {
                PxRigidBodyExt_setMassAndUpdateInertia_1(
                    actor_handle,
                    mass,
                    center.to_physx_sys().as_ptr(),
                    false
                );
            }
        }
    }
}
