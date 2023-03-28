#[doc(hidden)]
pub use super::type_bridge::*;

#[doc(hidden)]
pub use super::{
    PhysXPlugin,
    PhysicsSchedule,
    PhysicsSet,
    PhysicsTime,
    FoundationDescriptor,
    SceneDescriptor,
};

#[doc(hidden)]
pub use super::assets::{Geometry, Material};

#[doc(hidden)]
pub use super::components::{RigidBody, Shape, ShapeHandle, MassProperties, Velocity, Vehicle, VehicleHandle};

#[doc(hidden)]
pub use super::resources::{Physics, Scene, Cooking, VehicleSimulation};

#[doc(hidden)]
pub use super::render::PhysXDebugRenderPlugin;
