//! Extension traits for physx-rs crate.
//!
//! These are corresponding pull requests:
//! - <https://github.com/EmbarkStudios/physx-rs/pull/195>
//! - <https://github.com/EmbarkStudios/physx-rs/pull/206>

mod actor_map;
pub use actor_map::*;

mod convex_mesh;
pub use convex_mesh::*;

mod height_field;
pub use height_field::*;

mod triangle_mesh;
pub use triangle_mesh::*;
