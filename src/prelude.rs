#[doc(hidden)]
pub use super::utils::type_bridge::*;

#[doc(hidden)]
pub use super::{
    FoundationDescriptor,
    PhysXPlugin,
    PhysicsSchedule,
    PhysicsSet,
    PhysicsTime,
    SceneDescriptor,
    TimestepMode,
};

#[doc(hidden)]
pub use super::assets::{Geometry, Material};

#[doc(hidden)]
pub use super::components::{
    ArticulationJoint,
    ArticulationJointMotion,
    RigidBody,
    Shape,
};

#[doc(hidden)]
pub use super::events::AppExtensions;

#[doc(hidden)]
pub use super::plugins::{
    Damping,
    ExternalForce,
    NameFormatter,
    MassProperties,
    MaxVelocity,
    ShapeFilterData,
    ShapeOffsets,
    Velocity,
};

#[doc(hidden)]
pub use super::raycast::{RaycastHit, SceneQueryExt};

#[doc(hidden)]
pub use super::resources::{Physics, Scene};

#[doc(hidden)]
pub use super::render::PhysXDebugRenderPlugin;
