//! Set the shape's contact offset and rest offset for collisions.
use bevy::prelude::*;
use physx::traits::Class;
use physx_sys::{PxShape_setContactOffset_mut, PxShape_setRestOffset_mut};

use crate::components::ShapeHandle;
use crate::prelude::{Scene, *};

#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
/// Set the shape's contact offset and rest offset for collisions.
pub struct ShapeOffsets {
    /// Sets the contact offset.
    ///
    /// Shapes whose distance is less than the sum of their contactOffset values will
    /// generate contacts. The contact offset must be positive and greater than the rest
    /// offset. Having a contactOffset greater than than the restOffset allows the collision
    /// detection system to predictively enforce the contact constraint even when the objects
    /// are slightly separated. This prevents jitter that would occur if the constraint were
    /// enforced only when shapes were within the rest distance.
    ///
    /// Default: 0.02f * PxTolerancesScale::length
    pub contact_offset: f32,
    /// Sets the rest offset.
    ///
    /// Two shapes will come to rest at a distance equal to the sum of their restOffset
    /// values. If the restOffset is 0, they should converge to touching exactly.
    /// Having a restOffset greater than zero is useful to have objects slide smoothly,
    /// so that they do not get hung up on irregularities of each others’ surfaces.
    ///
    /// Default: 0.0f
    pub rest_offset: f32,
}

pub struct ShapeOffsetsPlugin;

impl Plugin for ShapeOffsetsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShapeOffsets>();
        app.add_systems(PhysicsSchedule, shape_offsets_sync.in_set(PhysicsSet::Sync));
    }
}

pub fn shape_offsets_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<
        (Option<&mut ShapeHandle>, Ref<ShapeOffsets>),
        Or<(Added<ShapeHandle>, Changed<ShapeOffsets>)>,
    >,
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (handle, offsets) in actors.iter_mut() {
        if let Some(mut handle) = handle {
            let mut handle = handle.get_mut(&mut scene);

            unsafe {
                PxShape_setContactOffset_mut(handle.as_mut_ptr(), offsets.contact_offset);
                PxShape_setRestOffset_mut(handle.as_mut_ptr(), offsets.rest_offset);
            };
        } else if !offsets.is_added() {
            bevy::log::warn!("ShapeOffsets component exists, but it's not a shape");
        };
    }
}
