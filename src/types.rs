//! Monomorphized PhysX types used by Bevy plugin.
//!
use bevy::prelude::Entity;

use crate::callbacks;

pub type PxMaterial = physx::material::PxMaterial<()>;
pub type PxShape = physx::shape::PxShape<Entity, PxMaterial>;
pub type PxArticulationLink = physx::articulation_link::PxArticulationLink<Entity, PxShape>;
pub type PxRigidStatic = physx::rigid_static::PxRigidStatic<Entity, PxShape>;
pub type PxRigidDynamic = physx::rigid_dynamic::PxRigidDynamic<Entity, PxShape>;
pub type PxArticulationReducedCoordinate =
    physx::articulation_reduced_coordinate::PxArticulationReducedCoordinate<Entity, PxArticulationLink>;

pub type PxScene = physx::scene::PxScene<
    (),
    PxArticulationLink,
    PxRigidStatic,
    PxRigidDynamic,
    PxArticulationReducedCoordinate,
    callbacks::OnCollision,
    callbacks::OnTrigger,
    callbacks::OnConstraintBreak,
    callbacks::OnWakeSleep,
    callbacks::OnAdvance,
>;
