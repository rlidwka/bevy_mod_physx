// These are optional additional components that we can sync with PhysX engine.
//
// They are added as a convenience, users can potentially disable them or implement
// their own similar plugins.
//

mod articulation;
pub use articulation::{
    ArticulationJointDriveTargets,
    ArticulationJointDrives,
    ArticulationPlugin,
    ArticulationRoot,
};

mod damping;
pub use damping::{Damping, DampingPlugin};

mod external_force;
pub use external_force::{ExternalForce, ExternalForcePlugin};

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

mod velocity;
pub use velocity::{Velocity, VelocityPlugin};
