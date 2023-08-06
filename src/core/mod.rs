//! Basic physics parts belonging to [PhysicsCore](crate::PhysicsCore) plugin.
pub mod articulation;
pub mod foundation;
pub mod geometry;
pub mod material;
pub mod rigid_dynamic;
pub mod rigid_static;
pub mod scene;
pub mod shape;
pub mod systems;

use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
/// Defines a type of rigid body you want to create.
pub enum RigidBody {
    /// Dynamic rigid body simulation object (PxRigidDynamic).
    Dynamic,
    /// Static rigid body simulation object (PxRigidStatic).
    Static,
    /// Tree structure of bodies connected by joints (PxArticulationReducedCoordinate).
    ArticulationLink,
}
