//! Re-exports of commonly imported symbols from this crate.
//!
//! Please note that this prelude conflicts with Bevy's prelude
//! (e.g. [Material] vs [bevy::prelude::Material], [Scene] vs [bevy::prelude::Scene]).
//! Suggestions on how to resolve these conflicts are welcome.

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

pub use crate::plugins::articulation::{
    ArticulationJointDriveTarget,
    ArticulationJointDriveVelocity,
    ArticulationJointDrives,
    ArticulationJointPosition,
    ArticulationRoot,
};

pub use crate::plugins::damping::Damping;
pub use crate::plugins::debug_render::DebugRenderSettings;
pub use crate::plugins::external_force::ExternalForce;
pub use crate::plugins::kinematic::Kinematic;
pub use crate::plugins::name::NameFormatter;
pub use crate::plugins::mass_properties::MassProperties;
pub use crate::plugins::shape_offsets::ShapeOffsets;
pub use crate::plugins::shape_filter_data::ShapeFilterData;
pub use crate::plugins::sleep::{SleepControl, Sleeping};
pub use crate::plugins::velocity::{MaxVelocity, Velocity};

pub use crate::resources::{Physics, Scene};

pub use crate::utils::events::AppExtensions;
pub use crate::utils::physx_extras::{ActorMapExtras, ConvexMeshExtras, HeightFieldExtras, TriangleMeshExtras};
pub use crate::utils::raycast::{RaycastHit, SceneQueryExt};
pub use crate::utils::type_bridge::*;
