//! Re-exports of commonly imported symbols from this crate.
//!
//! Please note that this prelude conflicts with Bevy's prelude
//! (e.g. [Material] vs [bevy::prelude::Material], [Scene] vs [bevy::prelude::Scene]).
//! Suggestions on how to resolve these conflicts are welcome.

pub use crate::utils::type_bridge::*;

pub use crate::{
    FoundationDescriptor,
    PhysicsCore,
    PhysicsPlugins,
    PhysicsSchedule,
    PhysicsSet,
    PhysicsTime,
    SceneDescriptor,
    TimestepMode,
};

pub use crate::assets::{Geometry, Material};

pub use crate::components::{
    ArticulationJoint,
    ArticulationJointMotion,
    RigidBody,
    Shape,
};

pub use crate::events::AppExtensions;

pub use crate::plugins::{
    ArticulationJointDriveTargets,
    ArticulationJointDrives,
    ArticulationRoot,
    Damping,
    DebugRenderSettings,
    ExternalForce,
    Kinematic,
    NameFormatter,
    MassProperties,
    MaxVelocity,
    ShapeOffsets,
    ShapeFilterData,
    SleepControl,
    Sleeping,
    Velocity,
};

pub use crate::raycast::{RaycastHit, SceneQueryExt};

pub use crate::resources::{Physics, Scene};
