//! Re-exports of commonly imported symbols from this crate.
//!
//! Please note that this prelude conflicts with Bevy's prelude
//! (e.g. [Material] vs [bevy::prelude::Material], [Scene] vs [bevy::prelude::Scene]).
//! Suggestions on how to resolve these conflicts are welcome.

pub use crate::{
    PhysicsCore,
    PhysicsPlugins,
    PhysicsSchedule,
    PhysicsSet,
    PhysicsTime,
    TimestepMode,
};

pub use crate::core::articulation::{
    ArticulationJoint,
    ArticulationJointMotion,
    ArticulationLinkHandle,
    ArticulationRootHandle,
};
pub use crate::core::foundation::{FoundationDescriptor, Physics};
pub use crate::core::geometry::Geometry;
pub use crate::core::material::Material;
pub use crate::core::rigid_dynamic::RigidDynamicHandle;
pub use crate::core::rigid_static::RigidStaticHandle;
pub use crate::core::scene::{Scene, SceneDescriptor};
pub use crate::core::shape::{Shape, ShapeHandle};
pub use crate::core::RigidBody;

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
pub use crate::plugins::mass_properties::MassProperties;
pub use crate::plugins::name::NameFormatter;
pub use crate::plugins::shape_filter_data::ShapeFilterData;
pub use crate::plugins::shape_offsets::ShapeOffsets;
pub use crate::plugins::sleep::{SleepControl, Sleeping};
pub use crate::plugins::velocity::{MaxVelocity, Velocity};
pub use crate::plugins::lock_flags::RigidDynamicLockFlags;

pub use crate::utils::events::AppExtensions;
pub use crate::utils::raycast::{RaycastHit, SceneQueryExt};
pub use crate::utils::type_bridge::*;
