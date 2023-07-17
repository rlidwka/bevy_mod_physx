// These are optional additional components that we can sync with PhysX engine.
//
// They are added as a convenience, users can potentially disable them or implement
// their own similar plugins.
//

mod damping;
pub use damping::{Damping, DampingPlugin};

mod debug_render;
pub use debug_render::{DebugRenderPlugin, DebugRenderSettings};

mod external_force;
pub use external_force::{ExternalForce, ExternalForcePlugin};

mod kinematic;
pub use kinematic::{Kinematic, KinematicPlugin};

mod mass_properties;
pub use mass_properties::{MassProperties, MassPropertiesPlugin};

mod max_velocity;
pub use max_velocity::{MaxVelocity, MaxVelocityPlugin};

mod name;
pub use name::{NameFormatter, NamePlugin};

mod shape_filter_data;
pub use shape_filter_data::{ShapeFilterData, ShapeFilterDataPlugin};

mod shape_offsets;
pub use shape_offsets::{ShapeOffsets, ShapeOffsetsPlugin};

mod sleep_control;
pub use sleep_control::{SleepControl, SleepControlPlugin};

mod sleep_marker;
pub use sleep_marker::{SleepMarkerPlugin, Sleeping};
pub(crate) use sleep_marker::WakeSleepCallback;

mod velocity;
pub use velocity::{Velocity, VelocityPlugin};
