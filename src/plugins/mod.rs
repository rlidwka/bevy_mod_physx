//! Collection of plugins that sync additional components with PhysX engine.
//!
//! These are added as a convenience, users can potentially disable them or implement
//! their own similar plugins.
//!
pub mod articulation;
pub mod damping;
#[cfg(feature = "debug-render")]
pub mod debug_render;
pub mod external_force;
pub mod kinematic;
pub mod mass_properties;
pub mod name;
pub mod shape_filter_data;
pub mod shape_offsets;
pub mod sleep;
pub mod velocity;
pub mod lock_flags;
